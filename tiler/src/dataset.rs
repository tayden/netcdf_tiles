use crate::bounds::Bounds;
use std::path::Path;

pub struct Dataset {
    lats: Vec<f64>,
    lons: Vec<f64>,
    file: netcdf::File,
    inv_y: bool,
    inv_x: bool,
}

impl Dataset {
    pub fn new(path: &Path, lat_name: &str, lon_name: &str) -> anyhow::Result<Self> {
        let file = netcdf::open(path)?;

        let lats = &file.variable(lat_name).expect("No latitude variable in dataset");
        let lats = lats.values_arr::<f64, _>(..)?;
        let lats = lats.into_raw_vec();

        let lons = &file.variable(lon_name).expect("No longitude variable in dataset");
        let lons = lons.values_arr::<f64, _>(..)?;
        let lons = lons.into_raw_vec();

        let inv_y = lats[0] > lats[1];
        let inv_x = lons[0] > lons[1];

        Ok(Self {
            lats,
            lons,
            file,
            inv_y,
            inv_x,
        })
    }

    pub fn lats(&self) -> &[f64] {
        &self.lats
    }

    pub fn lons(&self) -> &[f64] {
        &self.lons
    }

    pub fn get_bounds(&self) -> Bounds {
        let (min_x, max_x) = match self.inv_x {
            true => (self.lons.last().unwrap(), self.lons.first().unwrap()),
            false => (self.lons.first().unwrap(), self.lons.last().unwrap()),
        };
        let (min_y, max_y) = match self.inv_y {
            true => (self.lats.last().unwrap(), self.lats.first().unwrap()),
            false => (self.lats.first().unwrap(), self.lats.last().unwrap()),
        };

        Bounds::new(*min_x, *min_y, *max_x, *max_y)
    }

    fn get_dim_index(&self, dim: &[f64], val: f64) -> usize {
        let mut closest_i = 0;
        let mut closest_dv = f64::abs(val - dim[0]);
        for (i, v) in dim.iter().enumerate() {
            let dv = f64::abs(*v - val);
            if dv < closest_dv {
                closest_i = i;
                closest_dv = dv;
            } else if (dv > closest_dv) && (i > 0) {
                return i - 1;
            }
        }
        closest_i
    }

    pub fn get_values(
        &self,
        var_name: &str,
        bounds: Bounds,
    ) -> anyhow::Result<ndarray::ArrayD<f64>> {
        let data = self.file.variable(var_name).unwrap();

        // Get start and end indices for lat and lon
        let (start_lat_i, end_lat_i) = if self.inv_y {
            (
                self.get_dim_index(&self.lats, bounds.max_y),
                self.get_dim_index(&self.lats, bounds.min_y),
            )
        } else {
            (
                self.get_dim_index(&self.lats, bounds.min_y),
                self.get_dim_index(&self.lats, bounds.max_y),
            )
        };
        let (start_lon_i, end_lon_i) = if self.inv_x {
            (
                self.get_dim_index(&self.lons, bounds.max_x),
                self.get_dim_index(&self.lons, bounds.min_x),
            )
        } else {
            (
                self.get_dim_index(&self.lons, bounds.min_x),
                self.get_dim_index(&self.lons, bounds.max_x),
            )
        };

        let mut result = data.values_arr::<f64, _>((start_lat_i..end_lat_i, start_lon_i..end_lon_i))?;

        if self.inv_y {
            result.invert_axis(ndarray::Axis(0));
        }

        if self.inv_x {
            result.invert_axis(ndarray::Axis(1));
        }

        Ok(result)
    }
}


#[cfg(test)]
mod dataset_test {
    use super::*;
    use std::path::Path;

    // #[test]
    // fn test_dset_bounds() {
    //     let dset_path = Path::new("../testfiles/6_bin8_data/2023/07/01/mosaic_bin8_output.nc");
    //     let dset = Dataset::new(dset_path, "lat", "lon").unwrap();
    //     let bounds = dset.get_bounds();
    //     // Bounds from ERDDAP metadata
    //     assert_relative_eq!(bounds.min_x, -139.00062881782247);
    //     assert_relative_eq!(bounds.min_y, 47.00098814229249);
    //     assert_relative_eq!(bounds.max_x, -121.50242544017246);
    //     assert_relative_eq!(bounds.max_y, 59.5000898311175);
    // }
    //
    // #[test]
    // fn test_get_dim_index() {
    //     let dset_path = Path::new("../testfiles/6_bin8_data/2023/07/01/mosaic_bin8_output.nc");
    //     let dset = Dataset::new(dset_path, "lat", "lon").unwrap();
    //     let bounds = dset.get_bounds();
    //     let lats = dset.lats();
    //     let lons = dset.lons();
    //
    //     let min_yi = dset.get_dim_index(lats, bounds.min_y);
    //     assert_eq!(min_yi, lats.len() - 1);
    //     let max_yi = dset.get_dim_index(lats, bounds.max_y);
    //     assert_eq!(max_yi, 0);
    //
    //     let min_xi = dset.get_dim_index(lons, bounds.min_x);
    //     assert_eq!(min_xi, 0);
    //     let max_xi = dset.get_dim_index(lons, bounds.max_x);
    //     assert_eq!(max_xi, lons.len() - 1);
    //
    //     // From ERDDAP dset metadata
    //     let lat_res = 0.0026949335249730503;
    //     let lon_res = 0.0026949335249730503;
    //
    //     let min_yi = dset.get_dim_index(lats, bounds.min_y + lat_res);
    //     assert_eq!(min_yi, lats.len() - 2);
    //     let max_yi = dset.get_dim_index(lats, bounds.max_y - lat_res);
    //     assert_eq!(max_yi, 1);
    //
    //     let min_xi = dset.get_dim_index(lons, bounds.min_x + lon_res);
    //     assert_eq!(min_xi, 1);
    //     let max_xi = dset.get_dim_index(lons, bounds.max_x - lon_res);
    //     assert_eq!(max_xi, lons.len() - 2);
    //
    //     let min_yi = dset.get_dim_index(lats, bounds.min_y + 10.0 * lat_res);
    //     assert_eq!(min_yi, lats.len() - 11);
    //     let max_yi = dset.get_dim_index(lats, bounds.max_y - 10.0 * lat_res);
    //     assert_eq!(max_yi, 10);
    //
    //     let min_xi = dset.get_dim_index(lons, bounds.min_x + 10.0 * lon_res);
    //     assert_eq!(min_xi, 10);
    //     let max_xi = dset.get_dim_index(lons, bounds.max_x - 10.0 * lon_res);
    //     assert_eq!(max_xi, lons.len() - 11);
    // }
}