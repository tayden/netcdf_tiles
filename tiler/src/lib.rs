use std::path::Path;
use globalmaptile::GlobalMercator;
use bounds::Bounds;
use dataset::Dataset;

mod bounds;
mod dataset;

pub const TILE_SIZE: usize = 256;


pub fn get_tile(dset_path: &Path, tx: i32, ty: i32, zoom: u32, var_name: &str, lat_name: &str, lon_name: &str) -> anyhow::Result<Option<Vec<f64>>> {
    let mercator = GlobalMercator::new(TILE_SIZE as u32);

    let dset = Dataset::new(dset_path, lat_name, lon_name)?;
    let dset_bounds = dset.get_bounds();
    let tile_bounds: Bounds = mercator.tile_bounds(tx, ty, zoom).into();

    // Get the meter distance between result pixels
    let (tile_x_delta, tile_y_delta) = tile_bounds.get_pixel_lengths(TILE_SIZE, TILE_SIZE);

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

    // Get the pixel indices of the result tile
    let pixel_bounds = intersect_bounds.xy_shift(-tile_bounds.min_x, -tile_bounds.min_y).xy_scale(1.0 / tile_x_delta, 1.0 / tile_y_delta);

    let (x_len, y_len) = pixel_bounds.xy_len();
    let x_len = x_len as usize;
    let y_len = y_len as usize;

    // Read the intersection data
    // TODO: Try to decimate the read data here by passing a stride to get_value
    let values = dset.get_values(var_name, intersect_bounds)?;

    // Get decimation factors
    let dec_y: usize = values.shape()[0] / y_len;
    let dec_x: usize  = values.shape()[1] / x_len;

    if dec_y == 0 || dec_x == 0 {
        // TODO: Bilinear upsampling?
        // println!("Too much zoom");
        return Ok(None);
    }

    // Decimate the data
    for y in 0..y_len {
        for x in 0..x_len {
            let val = values[[y * dec_y, x * dec_x]];
            result[(y_len - 1 - y) * TILE_SIZE + x] = val;
        }
    }

    Ok(Some(result))
}
