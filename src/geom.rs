#[derive(Debug, Copy, Clone)]
pub struct Site2D {
    pub x: f64,
    pub y: f64,
}

impl Site2D {
    pub fn new(x: f64, y: f64) -> Site2D {
        Site2D { x, y }
    }

    pub fn distance(&self, other: &Site2D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}
