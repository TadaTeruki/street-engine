use rstar::{PointDistance, RTreeObject};

use super::site::Site;

/// Representation of a rectangle.
#[derive(Debug, Copy, Clone)]
pub struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rect {
    /// Create a rectangle from x, y, width, and height.
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle from two sites.
    pub fn from_sites(start: &Site, end: &Site) -> Self {
        let x = start.x.min(end.x);
        let y = start.y.min(end.y);
        let width = start.x.max(end.x) - x;
        let height = start.y.max(end.y) - y;
        Self::new(x, y, width, height)
    }
}

impl RTreeObject for Rect {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_corners(
            [self.x, self.y],
            [self.x + self.width, self.y + self.height],
        )
    }
}

impl PointDistance for Rect {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let x = point[0].max(self.x).min(self.x + self.width);
        let y = point[1].max(self.y).min(self.y + self.height);
        (x - point[0]).powi(2) + (y - point[1]).powi(2)
    }
}

#[cfg(test)]
mod tests {
    use rstar::RTreeObject;

    use super::*;

    #[test]
    fn test_rstar() {
        let rect0 = Rect::new(0.0, 0.0, 1.0, 1.0);
        let rect1 = Rect::new(0.5, 0.5, 1.0, 1.0);
        let rect2 = Rect::new(2.0, 2.0, 1.0, 1.0);

        let rs = rstar::RTree::bulk_load(vec![rect0, rect1, rect2]);

        let query = [0.0, 0.0];
        let result = rs.locate_all_at_point(&query).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);

        let query = [0.5, 0.5];
        let result = rs.locate_all_at_point(&query).collect::<Vec<_>>();
        assert_eq!(result.len(), 2);

        let query = [1.5, 1.5];
        let result = rs.locate_all_at_point(&query).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);

        let query = [2.5, 2.5];
        let result = rs.locate_all_at_point(&query).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);

        let query = [5.0, 5.0];
        let result = rs.locate_all_at_point(&query).collect::<Vec<_>>();
        assert_eq!(result.len(), 0);

        let query = [0.0, 0.0];
        let result = rs.nearest_neighbor(&query).unwrap();
        assert_eq!(
            result.envelope(),
            rstar::AABB::from_corners([0.0, 0.0], [1.0, 1.0])
        );

        let query = [5.0, 5.0];
        let result = rs.nearest_neighbor(&query).unwrap();
        assert_eq!(
            result.envelope(),
            rstar::AABB::from_corners([2.0, 2.0], [3.0, 3.0])
        );
    }
}
