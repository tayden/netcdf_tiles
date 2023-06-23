use std::path::Path;
use globalmaptile::GlobalMercator;

pub const TILE_SIZE: usize = 256;

struct Dataset {
    lats: Vec<f64>,
    lons: Vec<f64>,
    file: netcdf::File,
}

impl Dataset {
    fn new(path: &Path, lat_name: &str, lon_name: &str) -> anyhow::Result<Self> {
        let file = netcdf::open(path)?;
        let mercator = GlobalMercator::new(TILE_SIZE as u32);

        let lats = &file.variable(lat_name).unwrap();
        let lats = lats.values_arr::<f64, _>(..)?;

        let lons = &file.variable(lon_name).unwrap();
        let lons = lons.values_arr::<f64, _>(..)?;


        let lats_m = lats.iter().map(|lat| {
            mercator.lat_lon_to_meters(*lat, lons[0]).1
        }).collect::<Vec<f64>>();

        let lons_m = lons.iter().map(|lon| {
            mercator.lat_lon_to_meters(lats[0], *lon).0
        }).collect::<Vec<f64>>();

        Ok(Self {
            lats: lats_m,
            lons: lons_m,
            file,
        })
    }

    fn get_bounds(&self) -> anyhow::Result<[f64; 4]> {
        let (min_lat, max_lat) = (self.lats.first().unwrap(), self.lats.last().unwrap());
        let (min_lon, max_lon) = (self.lons.first().unwrap(), self.lons.last().unwrap());
        Ok([*min_lon, *min_lat, *max_lon, *max_lat])
    }

    fn get_dim_index(&self, dim: &Vec<f64>, val: f64) -> usize {
        dim.iter().enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a - val).abs().partial_cmp(&(*b - val).abs()).unwrap()
            }).unwrap().0
    }

    fn get_value(& self, var_name: &str, start_lon: f64, end_lon: f64, start_lat: f64, end_lat: f64) -> anyhow::Result<ndarray::ArrayD<f64>> {
        let data = self.file.variable(var_name).unwrap();

        // Get start and end indices for lat and lon
        let start_lat_i = self.get_dim_index(&self.lats, start_lat);
        let end_lat_i = self.get_dim_index(&self.lats, end_lat);
        let start_lon_i = self.get_dim_index(&self.lons, start_lon);
        let end_lon_i = self.get_dim_index(&self.lons, end_lon);

        let result = data.values_arr::<f64, _>((start_lat_i..end_lat_i, start_lon_i..end_lon_i))?;

        Ok(result)
    }
}


pub fn get_tile(dset_path: &Path, tx: i32, ty: i32, zoom: u32, var_name: &str, lat_name: &str, lon_name: &str) -> anyhow::Result<Vec<f64>> {
    let mercator = GlobalMercator::new(TILE_SIZE as u32);

    let dset = Dataset::new(dset_path, lat_name, lon_name)?;
    let dset_bounds = dset.get_bounds()?;
    let tile_bounds = mercator.tile_bounds(tx, ty, zoom);
    // println!("dset bounds: {:?}", dset_bounds);
    // println!("tile_bounds: {:?}", tile_bounds);

    // Get the meter distance between result pixels
    let tile_x_delta = (tile_bounds.2 - tile_bounds.0) / (TILE_SIZE as f64);
    let tile_y_delta = (tile_bounds.3 - tile_bounds.1) / (TILE_SIZE as f64);

    // Create result array and image
    let mut result = vec![0.0; TILE_SIZE * TILE_SIZE];

    // Check if bounds intersect
    if dset_bounds[0] > tile_bounds.2 || dset_bounds[2] < tile_bounds.0 || dset_bounds[1] > tile_bounds.3 || dset_bounds[3] < tile_bounds.1 {
        // TODO: Return None to allow sending No Content 204 response
        println!("No intersection");
        return Ok(result)
    }

    // Get intersection bounds
    let start_lon = if dset_bounds[0] > tile_bounds.0 { dset_bounds[0] } else { tile_bounds.0 };
    let end_lon = if dset_bounds[2] < tile_bounds.2 { dset_bounds[2] } else { tile_bounds.2 };
    let start_lat = if dset_bounds[1] > tile_bounds.1 { dset_bounds[1] } else { tile_bounds.1 };
    let end_lat = if dset_bounds[3] < tile_bounds.3 { dset_bounds[3] } else { tile_bounds.3 };

    // TODO: Try to decimate the read data here by passing a stride to get_value
    // Read the intersection data
    let value = dset.get_value(var_name, start_lon, end_lon, start_lat, end_lat)?;
    // println!("value: {:?}", value);

    // Get the pixel indices of the result tile
    let start_x = ((start_lon - tile_bounds.0) / tile_x_delta) as usize;
    let end_x = ((end_lon - tile_bounds.0) / tile_x_delta) as usize;
    let start_y = ((start_lat - tile_bounds.1) / tile_y_delta) as usize;
    let end_y = ((end_lat - tile_bounds.1) / tile_y_delta) as usize;
    // println!("start_x: {:?}, end_x: {:?}, start_y: {:?}, end_y: {:?}", start_x, end_x, start_y, end_y);

    let y_len = end_y - start_y;
    let x_len = end_x - start_x;
    // println!("y_len: {:?}, x_len: {:?}", y_len, x_len);

    // Get decimation factors
    let dec_y: usize = value.shape()[0] / y_len;
    let dec_x: usize = value.shape()[1] / x_len;
    // println!("dec_y: {:?}, dec_x: {:?}", dec_y, dec_x);

    if dec_y == 0 || dec_x == 0 {
        // TODO: Return None to allow sending No Content 204 response
        // TODO: Bilinear upsampling?
        println!("Too much zoom");
        return Ok(result);
    }

    // Decimate the data
    for y in 0..y_len {
        for x in 0..x_len {
            let val = value[[y * dec_y, x * dec_x]];
            result[(y_len - 1 - y) * TILE_SIZE + x] = val;
        }
    }
    // println!("result: {:?}", result);

    Ok(result)
}