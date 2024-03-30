/// Representation of a 2D site.
#[derive(Debug, Copy, Clone)]
pub struct Site {
    pub x: f64,
    pub y: f64,
}

impl PartialEq for Site {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Site {}

impl Site {
    /// Create a site from x and y coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate the euclidean distance to the other site.
    fn distance(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Calculate the site moved by the angle and distance.
    fn extend(&self, angle: f64, distance: f64) -> Self {
        let x = self.x + angle.cos() * distance;
        let y = self.y + angle.sin() * distance;
        Self::new(x, y)
    }

    /// Calculate the angle to the other site.
    fn get_angle(&self, other: &Self) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        dy.atan2(dx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let site1 = Site::new(0.0, 0.0);
        let site2 = Site::new(3.0, 4.0);
        assert_eq!(site1.distance(&site2), 5.0);
    }
}
