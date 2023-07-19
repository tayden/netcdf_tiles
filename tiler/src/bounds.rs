#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Bounds {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn min_x(&self) -> f64 {
        self.min_x
    }

    pub fn min_y(&self) -> f64 {
        self.min_y
    }

    pub fn max_x(&self) -> f64 {
        self.max_x
    }

    pub fn max_y(&self) -> f64 {
        self.max_y
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        // If the two bounds don't intersect, return None
        if self.min_x > other.max_x
            || self.max_x < other.min_x
            || self.min_y > other.max_y
            || self.max_y < other.min_y
        {
            return None;
        }
        // Else return the intersection
        let min_x = self.min_x.max(other.min_x);
        let min_y = self.min_y.max(other.min_y);
        let max_x = self.max_x.min(other.max_x);
        let max_y = self.max_y.min(other.max_y);
        Some(Self::new(min_x, min_y, max_x, max_y))
    }

    pub fn get_pixel_lengths(&self, num_x_pixels: usize, num_y_pixels: usize) -> (f64, f64) {
        let x_delta = (self.max_x - self.min_x) / num_x_pixels as f64;
        let y_delta = (self.max_y - self.min_y) / num_y_pixels as f64;
        (x_delta, y_delta)
    }

    pub fn xy_scale(&self, x_scale: f64, y_scale: f64) -> Self {
        Self::new(
            self.min_x * x_scale,
            self.min_y * y_scale,
            self.max_x * x_scale,
            self.max_y * y_scale,
        )
    }

    pub fn xy_shift(&self, x_shift: f64, y_shift: f64) -> Self {
        Self::new(
            self.min_x + x_shift,
            self.min_y + y_shift,
            self.max_x + x_shift,
            self.max_y + y_shift,
        )
    }

    pub fn xy_len(&self) -> (usize, usize) {
        let x_len = (self.max_x - self.min_x).round() as usize;
        let y_len = (self.max_y - self.min_y).round() as usize;

        (x_len, y_len)
    }
}

impl From<(f64, f64, f64, f64)> for Bounds {
    fn from((min_x, min_y, max_x, max_y): (f64, f64, f64, f64)) -> Self {
        Self::new(min_x, min_y, max_x, max_y)
    }
}

impl std::ops::Sub for Bounds {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.min_x - other.min_x,
            self.min_y - other.min_y,
            self.max_x - other.max_x,
            self.max_y - other.max_y,
        )
    }
}


#[cfg(test)]
mod bounds_tests {
    use super::*;

    #[test]
    fn test_intersect(){
        let bounds1 = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let bounds2 = Bounds::new(5.0, 5.0, 15.0, 15.0);
        let bounds12 = bounds1.intersect(&bounds2).unwrap();
        assert_eq!(bounds12.min_x, 5.0);
        assert_eq!(bounds12.min_y, 5.0);
        assert_eq!(bounds12.max_x, 10.0);
        assert_eq!(bounds12.max_y, 10.0);
    }
}