use std::io::Cursor;
use std::path::Path;

const BASE_PATH: &str = "/home/taylor/CLionProjects/netcdf_tiles/testfiles";

#[macro_use]
extern crate rocket;

#[derive(Responder)]
#[response(status = 200, content_type = "image/png")]
struct ImageTile(Vec<u8>);

#[get("/<var>/<year>/<month>/<day>/<x>/<y>/<z>")]
fn index(var: &str, year: u16, month: u8, day: u8, x: i32, y: i32, z: u32) -> ImageTile {
    // Variables to be defined via REST API
    let lat_name = "latitude";
    let lon_name = "longitude";
    // This path would normally be looked up by date

    let dset_path = format!("{}/{}/{:02}/{:02}/polymer_mosaic_output.nc", BASE_PATH, year, month, day);
    println!("dset_path: {}", dset_path);
    let dset_path = Path::new(&dset_path[..]);

    // Get tile
    let data = tiler::get_tile(dset_path, x, y, z, var, lat_name, lon_name).unwrap();
    println!("data: {:?}", data);
    // let gradient = colorous::VIRIDIS;
    // let max_value = 40.0;
    // let colors = data.iter().map(|v| {
    //     match v {
    //         x if *x < 0.0001 => None,
    //         _ => Some(gradient.eval_continuous(v / max_value)),
    //     }
    // }).collect::<Vec<Option<colorous::Color>>>();
    // println!("colors: {:?}", colors);
    // let mut imgbuf = image::RgbaImage::new(tiler::TILE_SIZE as u32, tiler::TILE_SIZE as u32);
    // // Convert to floating point image
    // for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    //     if let Some(val) = colors[(y * tiler::TILE_SIZE as u32 + x) as usize] {
    //         *pixel = image::Rgba([val.r, val.g, val.b, 255]);
    //         println!("{} {} {}", val.r, val.g, val.b);
    //     }
    // }
    //
    // // Write bytes
    let mut bytes: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    // imgbuf.write_to(&mut bytes, image::ImageOutputFormat::from(image::ImageFormat::Png)).unwrap();
    //
    let bytes = bytes.into_inner();
    ImageTile(bytes)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}