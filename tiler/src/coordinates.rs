use std::f64::consts::PI;
use crate::bounds::Bounds;
use crate::TILE_SIZE;


pub struct TileCoord {
    x: u32,
    y: u32,
    zoom: u8,
}

impl TileCoord {
    pub fn new(x: u32, y: u32, zoom: u8) -> Self {
        Self { x, y, zoom }
    }
    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    pub fn zoom(&self) -> u8 {
        self.zoom
    }
}

pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
}

pub struct LatLng {
    lat: f64,
    lng: f64,
}

impl LatLng {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lng(&self) -> f64 {
        self.lng
    }
}

pub fn from_lat_lng_to_point(lat_lng: &LatLng) -> Point {
    let mercator = -f64::ln(f64::tan((0.25 + lat_lng.lat() / 360.0) * PI));
    Point::new(
        (TILE_SIZE as f64) * (lat_lng.lng() / 360.0 + 0.5),
        (TILE_SIZE as f64) / 2.0 * (1.0 + mercator / PI),
    )
}

pub fn from_point_to_lat_lng(point: &Point) -> LatLng {
    let lng = (point.x / TILE_SIZE as f64 - 0.5) * 360.0;
    let mercator = ((point.y * 2.0 / (TILE_SIZE as f64)) - 1.0) * PI;
    let lat = ((f64::atan(f64::exp(-mercator)) / PI) - 0.25) * 360.0;
    LatLng::new(lat, lng)
}

pub fn from_lat_lng_to_tile_coord(lat_lng: &LatLng, zoom: u8) -> TileCoord {
    let scale = f64::powi(2.0, zoom as i32);
    let point = from_lat_lng_to_point(lat_lng);

    TileCoord::new(
        f64::floor(point.x * scale / (TILE_SIZE as f64)) as u32,
        f64::floor(point.y * scale / (TILE_SIZE as f64)) as u32,
        zoom,
    )
}

pub fn from_tile_coord_to_lat_lng_bounds(tile_coord: &TileCoord) -> Bounds {
    let scale = f64::powi(2.0, tile_coord.zoom() as i32);
    let min_point = Point::new(
        tile_coord.x() as f64 * (TILE_SIZE as f64) / scale,
        tile_coord.y() as f64 * (TILE_SIZE as f64) / scale,
    );
    let max_point = Point::new(
        (tile_coord.x() as f64 + 1.0) * (TILE_SIZE as f64) / scale,
        (tile_coord.y() as f64 + 1.0) * (TILE_SIZE as f64) / scale,
    );

    let min_lat_lng = from_point_to_lat_lng(&min_point);
    let max_lat_lng = from_point_to_lat_lng(&max_point);

    let min_lng = min_lat_lng.lng();
    let max_lng = max_lat_lng.lng();
    // These get flipped because the y axis is flipped between img and lat coordinates
    let max_lat = min_lat_lng.lat();
    let min_lat = max_lat_lng.lat();

    Bounds::new(min_lng, min_lat, max_lng, max_lat)
}


#[cfg(test)]
mod coordinate_transforms_tests {
    use super::*;

    #[test]
    fn test_from_lat_lng_to_point() {
        let lat_lng = LatLng::new(0.0, 0.0);
        let point = from_lat_lng_to_point(&lat_lng);
        assert_eq!(point.x(), 128.0);
        assert_eq!(point.y(), 128.0);

        let lat_lng = LatLng::new(0.0, 180.0);
        let point = from_lat_lng_to_point(&lat_lng);
        assert_eq!(point.x(), 256.0);
        assert_eq!(point.y(), 128.0);

        let lat_lng = LatLng::new(0.0, -180.0);
        let point = from_lat_lng_to_point(&lat_lng);
        assert_eq!(point.x(), 0.0);
        assert_eq!(point.y(), 128.0);

        let lat_lng = LatLng::new(85.051129, 0.0);
        let point = from_lat_lng_to_point(&lat_lng);
        assert_eq!(point.x(), 128.0);
        assert_abs_diff_eq!(point.y(), 0.0, epsilon=0.001);

        let lat_lng = LatLng::new(-85.051129, 0.0);
        let point = from_lat_lng_to_point(&lat_lng);
        assert_eq!(point.x(), 128.0);
        assert_abs_diff_eq!(point.y(), 256.0, epsilon=0.001);
    }

    #[test]
    fn test_from_point_to_lat_lng() {
        let point = Point::new(128.0, 128.0);
        let lat_lng = from_point_to_lat_lng(&point);
        assert_eq!(lat_lng.lat(), 0.0);
        assert_eq!(lat_lng.lng(), 0.0);

        let point = Point::new(256.0, 128.0);
        let lat_lng = from_point_to_lat_lng(&point);
        assert_eq!(lat_lng.lat(), 0.0);
        assert_eq!(lat_lng.lng(), 180.0);

        let point = Point::new(0.0, 128.0);
        let lat_lng = from_point_to_lat_lng(&point);
        assert_eq!(lat_lng.lat(), 0.0);
        assert_eq!(lat_lng.lng(), -180.0);

        let point = Point::new(128.0, 0.0);
        let lat_lng = from_point_to_lat_lng(&point);
        assert_relative_eq!(lat_lng.lat(), 85.051129, epsilon=0.001);
        assert_abs_diff_eq!(lat_lng.lng(),  0.0 , epsilon=0.001);

        let point = Point::new(128.0, 256.0);
        let lat_lng = from_point_to_lat_lng(&point);
        assert_relative_eq!(lat_lng.lat(), -85.051129, epsilon=0.001);
        assert_abs_diff_eq!(lat_lng.lng(),  0.0 , epsilon=0.001);
    }

    #[test]
    fn test_from_lat_lng_to_tile_coord() {
        let lat_lng = LatLng::new(0.0, 0.0);
        let tile_coord = from_lat_lng_to_tile_coord(&lat_lng, 0);
        assert_eq!(tile_coord.x(), 0);
        assert_eq!(tile_coord.y(), 0);
        assert_eq!(tile_coord.zoom(), 0);

        let lat_lng = LatLng::new(80.0, 120.0);
        let tile_coord = from_lat_lng_to_tile_coord(&lat_lng, 0);
        assert_eq!(tile_coord.x(), 0);
        assert_eq!(tile_coord.y(), 0);
        assert_eq!(tile_coord.zoom(), 0);

        let lat_lng = LatLng::new(80.0, -120.0);
        let tile_coord = from_lat_lng_to_tile_coord(&lat_lng, 1);
        assert_eq!(tile_coord.x(), 0);
        assert_eq!(tile_coord.y(), 0);
        assert_eq!(tile_coord.zoom(), 1);

        let lat_lng = LatLng::new(41.0, -134.0);
        let tile_coord = from_lat_lng_to_tile_coord(&lat_lng, 3);
        assert_eq!(tile_coord.x(), 1);
        assert_eq!(tile_coord.y(), 2);
        assert_eq!(tile_coord.zoom(), 3);

        let lat_lng = LatLng::new(6.5, 29.7);
        let tile_coord = from_lat_lng_to_tile_coord(&lat_lng, 9);
        assert_eq!(tile_coord.x(), 298);
        assert_eq!(tile_coord.y(), 246);
        assert_eq!(tile_coord.zoom(), 9);
    }

    #[test]
    fn test_from_tile_coord_to_lat_lng_bounds() {
        let tile_coord = TileCoord::new(0, 0, 0);
        let bounds = from_tile_coord_to_lat_lng_bounds(&tile_coord);
        assert_eq!(bounds.min_x(), -180.0);
        assert_relative_eq!(bounds.min_y(), -85.051129, epsilon=0.0001);
        assert_eq!(bounds.max_x(), 180.0);
        assert_relative_eq!(bounds.max_y(), 85.051129, epsilon=0.0001);

        let tile_coord = TileCoord::new(298, 246, 9);
        let bounds = from_tile_coord_to_lat_lng_bounds(&tile_coord);
        assert_relative_eq!(bounds.min_x(), 29.531253, epsilon=0.0001);
        assert_relative_eq!(bounds.min_y(), 6.315302, epsilon=0.0001);
        assert_relative_eq!(bounds.max_x(), 30.234373, epsilon=0.0001);
        assert_relative_eq!(bounds.max_y(), 7.013666, epsilon=0.0001);
    }
}
