use std::path::Path;
use webmerc::GlobalMercator;
use crate::bounds::Bounds;
use crate::TILE_SIZE;

pub struct Dataset {
    lats: Vec<f64>,
    lons: Vec<f64>,
    file: netcdf::File,
}

impl Dataset {
    pub fn new(path: &Path, lat_name: &str, lon_name: &str) -> anyhow::Result<Self> {
        let file = netcdf::open(path)?;
        let mercator = GlobalMercator::new(TILE_SIZE as u64);

        let lats = &file.variable(lat_name).expect("No latitude variable in dataset");
        let lats = lats.values_arr::<f64, _>(..)?;

        let lons = &file.variable(lon_name).expect("No longitude variable in dataset");
        let lons = lons.values_arr::<f64, _>(..)?;

        // Convert from WGS74 to Web Mercator (m)
        let lats_m = lats.iter().map(|lat| {
            mercator.lat_lon_to_meters(lons[0], *lat).1
        }).collect::<Vec<f64>>();

        let lons_m = lons.iter().map(|lon| {
            mercator.lat_lon_to_meters(*lon, lats[0]).0
        }).collect::<Vec<f64>>();

        Ok(Self {
            lats: lats_m,
            lons: lons_m,
            file,
        })
    }

    pub fn get_bounds(&self) -> Bounds {
        let min_x = self.lons.first().unwrap();
        let min_y = self.lats.first().unwrap();
        let max_x = self.lons.last().unwrap();
        let max_y = self.lats.last().unwrap();

        Bounds::new(*min_x, *min_y, *max_x, *max_y)
    }

    fn get_dim_index(&self, dim: &Vec<f64>, val: f64) -> usize {
        dim.iter().enumerate().min_by(|(_, a), (_, b)| {
            (*a - val).abs().partial_cmp(&(*b - val).abs()).unwrap()
        }).unwrap().0
    }

    pub fn get_values(&self, var_name: &str, bounds: Bounds) -> anyhow::Result<ndarray::ArrayD<f64>> {
        let data = self.file.variable(var_name).unwrap();

        // Get start and end indices for lat and lon
        let start_lat_i = self.get_dim_index(&self.lats, bounds.min_y);
        let end_lat_i = self.get_dim_index(&self.lats, bounds.max_y);
        let start_lon_i = self.get_dim_index(&self.lons, bounds.min_x);
        let end_lon_i = self.get_dim_index(&self.lons, bounds.max_x);

        let result = data.values_arr::<f64, _>((start_lat_i..end_lat_i, start_lon_i..end_lon_i))?;

        Ok(result)
    }
}
