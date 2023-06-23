use std::io::Cursor;
use std::path::Path;
use rocket::response::status::NoContent;

// TODO: Needs to be able to configure with env variables
const BASE_PATH: &str = "./testfiles";

#[macro_use]
extern crate rocket;

#[derive(Responder)]
#[response(status = 200, content_type = "image/png")]
struct ImageTile(Vec<u8>);

// Responds with image tile if there is one, otherwise 204
#[get("/<var>/<year>/<month>/<day>/<x>/<y>/<z>")]
fn index(var: &str, year: u16, month: u8, day: u8, x: i32, y: i32, z: u32) -> Result<ImageTile, NoContent> {
    // TODO: Allow override via query params?
    let lat_name = "latitude";
    let lon_name = "longitude";
    let max_value = 40.0;  // Scale values in image by this before converting to color
    let gradient = colorous::VIRIDIS;

    let dset_path = format!("{}/{}/{:02}/{:02}/polymer_mosaic_output.nc", BASE_PATH, year, month, day);
    let dset_path = Path::new(&dset_path[..]);

    // Get tile
    let data = match tiler::get_tile(dset_path, x, y, z, var, lat_name, lon_name) {
        Ok(Some(data)) => data,
        Ok(None) => return Err(NoContent),
        Err(e) => {
            println!("Error: {}", e);
            return Err(NoContent);
        }
    };

    // Convert to image
    let mut imgbuf = image::RgbaImage::new(tiler::TILE_SIZE as u32, tiler::TILE_SIZE as u32);

    data.iter().enumerate().for_each(|(i, v)| {
        if *v > 0.0 {
            let x = (i % tiler::TILE_SIZE) as usize;
            let y = (i / tiler::TILE_SIZE) as usize;

            let c = gradient.eval_continuous((*v / max_value).min(1.0).max(0.0));
            imgbuf.put_pixel(x as u32, y as u32, image::Rgba([c.r, c.g, c.b, 255]));
        }
    });

    // Write bytes
    let mut bytes: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    imgbuf.write_to(&mut bytes, image::ImageOutputFormat::from(image::ImageFormat::Png)).unwrap();

    let bytes = bytes.into_inner();
    Ok(ImageTile(bytes))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}