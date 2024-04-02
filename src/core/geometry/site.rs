use rstar::{PointDistance, RTreeObject, AABB};

use super::angle::Angle;

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

impl PartialOrd for Site {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Site {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ordering = self.x.total_cmp(&other.x);
        if ordering == std::cmp::Ordering::Equal {
            self.y.total_cmp(&other.y)
        } else {
            ordering
        }
    }
}

impl RTreeObject for Site {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for Site {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        ((self.x - point[0]).powi(2) + (self.y - point[1]).powi(2)).sqrt()
    }
}

impl Site {
    /// Create a site from x and y coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate the euclidean distance to the other site.
    pub fn distance(&self, other: &Self) -> f64 {
        self.distance_2(other).sqrt()
    }

    /// Calculate the squared euclidean distance to the other site.
    pub fn distance_2(&self, other: &Self) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    /// Calculate the site moved by the angle and distance.
    pub fn extend(&self, angle: Angle, distance: f64) -> Self {
        let x = self.x + angle.radian().cos() * distance;
        let y = self.y + angle.radian().sin() * distance;
        Self::new(x, y)
    }

    /// Calculate the angle to the other site.
    pub fn get_angle(&self, other: &Self) -> Angle {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        if dx == 0.0 && dy == 0.0 {
            return Angle::new(0.0);
        }
        Angle::new(dy.atan2(dx))
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

    #[test]
    fn test_extend() {
        let site = Site::new(0.0, 0.0);
        let angle = Angle::new(std::f64::consts::PI / 4.0);
        let distance = 1.0;
        let extended = site.extend(angle, distance);
        let expected = Site::new(1.0, 1.0);
        assert!(extended.distance(&expected) - distance < 1e-9);
    }

    #[test]
    fn test_get_angle() {
        let site0 = Site::new(0.0, 0.0);
        let site1 = Site::new(1.0, 0.0);
        let site2 = Site::new(1.0, 1.0);
        let site3 = Site::new(0.0, 1.0);
        let site4 = Site::new(-1.0, 1.0);
        let site5 = Site::new(-1.0, 0.0);
        let site6 = Site::new(-1.0, -1.0);
        let site7 = Site::new(0.0, -1.0);
        let site8 = Site::new(1.0, -1.0);

        assert_eq!(site0.get_angle(&site1).radian(), 0.0);
        assert_eq!(site0.get_angle(&site2).radian(), std::f64::consts::PI / 4.0);
        assert_eq!(site0.get_angle(&site3).radian(), std::f64::consts::PI / 2.0);
        assert_eq!(
            site0.get_angle(&site4).radian(),
            3.0 * std::f64::consts::PI / 4.0
        );
        assert_eq!(site0.get_angle(&site5).radian(), std::f64::consts::PI);
        assert_eq!(
            site0.get_angle(&site6).radian(),
            -3.0 * std::f64::consts::PI / 4.0
        );
        assert_eq!(
            site0.get_angle(&site7).radian(),
            -std::f64::consts::PI / 2.0
        );
        assert_eq!(
            site0.get_angle(&site8).radian(),
            -std::f64::consts::PI / 4.0
        );
    }
}
