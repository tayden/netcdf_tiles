use rocket::response::status::NoContent;
use std::io::Cursor;
use std::path::Path;

// TODO: Needs to be able to configure with env variables

const BASE_PATH: &str = "./testfiles";

#[macro_use]
extern crate rocket;

#[derive(Responder)]
#[response(status = 200, content_type = "image/png")]
struct ImageTile(Vec<u8>);

// Responds with image tile if there is one, otherwise 204
#[get("/<var>/<year>/<month>/<day>/<x>/<y>/<z>?<max_value>&<lat_dim>&<lon_dim>&<gradient>")]
fn index(
    var: &str,
    year: u16,
    month: u8,
    day: u8,
    x: u32,
    y: u32,
    z: u32,
    max_value: Option<f64>,
    lat_dim: Option<&str>,
    lon_dim: Option<&str>,
    gradient: Option<&str>,
) -> Result<ImageTile, NoContent> {
    // Handle optional query params
    let max_value = max_value.unwrap_or(10.0);
    let lat_name = lat_dim.unwrap_or("latitude");
    let lon_name = lon_dim.unwrap_or("longitude");
    // TODO: This is gross and should be handled better
    let gradient = match gradient {
        Some("turbo") => colorous::TURBO,
        Some("viridis") => colorous::VIRIDIS,
        Some("inferno") => colorous::INFERNO,
        Some("magma") => colorous::MAGMA,
        Some("plasma") => colorous::PLASMA,
        Some("cividis") => colorous::CIVIDIS,
        Some("warm") => colorous::WARM,
        Some("cool") => colorous::COOL,
        Some("cubehelix") => colorous::CUBEHELIX,
        Some("rainbow") => colorous::RAINBOW,
        Some("sinebow") => colorous::SINEBOW,
        Some("greens") => colorous::GREENS,
        Some("bluegreen") => colorous::BLUE_GREEN,
        _ => colorous::VIRIDIS,
    };

    let dset_path = format!(
        "{}/{}/{:02}/{:02}/polymer_mosaic_output.nc",
        BASE_PATH, year, month, day
    );
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
            let x = i % tiler::TILE_SIZE;
            let y = i / tiler::TILE_SIZE;

            let c = gradient.eval_continuous((*v / max_value).min(1.0).max(0.0));
            imgbuf.put_pixel(x as u32, y as u32, image::Rgba([c.r, c.g, c.b, 255]));
        }
    });

    // Write bytes
    let mut bytes = Cursor::new(Vec::new());
    imgbuf
        .write_to(
            &mut bytes,
            image::ImageOutputFormat::from(image::ImageFormat::Png),
        )
        .unwrap();

    let bytes = bytes.into_inner();
    Ok(ImageTile(bytes))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
