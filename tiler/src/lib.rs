use dataset::Dataset;
use std::path::Path;
use crate::coordinates::{TileCoord, from_tile_coord_to_lat_lng_bounds};

mod bounds;
mod dataset;
mod coordinates;

#[cfg(test)]
#[macro_use]
extern crate approx;

pub const TILE_SIZE: usize = 256;

pub fn get_tile(
    dset_path: &Path,
    tx: u32,
    ty: u32,
    zoom: u32,
    var_name: &str,
    lat_name: &str,
    lon_name: &str,
) -> anyhow::Result<Option<Vec<f64>>> {
    let dset = Dataset::new(dset_path, lat_name, lon_name)?;
    let dset_bounds = dset.get_bounds();
    let tile_bounds = from_tile_coord_to_lat_lng_bounds(&TileCoord::new(tx, ty, zoom as u8));

    // Create result array and image
    let mut result = vec![0.0; TILE_SIZE * TILE_SIZE];

    // Get bounds intersection
    let intersect_bounds = dset_bounds.intersect(&tile_bounds);
    let intersect_bounds = match intersect_bounds {
        Some(bounds) => bounds,
        None => {
            // println!("No intersection");
            return Ok(None);
        }
    };

    // Read the intersection data
    // TODO: Try to decimate the read data here by passing a stride to get_value
    let values = dset.get_values(var_name, intersect_bounds)?;

    // Get the meter distance between result pixels
    let (tile_x_delta, tile_y_delta) = tile_bounds.get_pixel_lengths(TILE_SIZE, TILE_SIZE);

    // Get the pixel indices of the result tile
    let pixel_bounds = intersect_bounds.xy_shift(-tile_bounds.min_x, -tile_bounds.min_y).xy_scale(1.0 / tile_x_delta, 1.0 / tile_y_delta);
    let (px_origin, py_origin) = (pixel_bounds.min_x as usize, pixel_bounds.min_y as usize);
    let (x_len, y_len) = pixel_bounds.xy_len();

    // Get decimation factors
    let dec_y: f64 = (values.shape()[0] as f64) / y_len as f64;
    let dec_x: f64 = (values.shape()[1] as f64) / x_len as f64;

    println!("v_shape: {:?}", values.shape());
    println!("dec_y: {}, dec_x: {}", dec_y, dec_x);

    // if dec_y == 0 || dec_x == 0 {
    //     // TODO: Nearest upsampling?
    //     // println!("Too much zoom");
    //     return Ok(None);
    // }

    // Decimate the data
    let mut yif = 0.0;
    let mut xif = 0.0;
    for yi in 0..y_len {
        for xi in 0..x_len {
            let val = values[[yif as usize, xif as usize]];
            let yi = (TILE_SIZE - 1) - (py_origin + yi);
            let xi = px_origin + xi;
            result[yi * TILE_SIZE + xi] = val;
            xif += dec_x;
        }
        yif += dec_y;
        xif = 0.0;
    }

    Ok(Some(result))
}
