#[derive(Debug, Copy, Clone)]
pub struct Site2D {
    pub x: f64,
    pub y: f64,
}

impl PartialEq for Site2D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Site2D {}

impl Site2D {
    pub fn new(x: f64, y: f64) -> Site2D {
        Site2D { x, y }
    }

    pub fn distance(&self, other: &Site2D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// calculate the intersection of two line segments
    pub fn get_intersection(
        &self,
        end0: &Site2D,
        start1: &Site2D,
        end1: &Site2D,
    ) -> Option<Site2D> {
        let x0 = self.x;
        let y0 = self.y;
        let x1 = end0.x;
        let y1 = end0.y;
        let x2 = start1.x;
        let y2 = start1.y;
        let x3 = end1.x;
        let y3 = end1.y;

        let a1 = y1 - y0;
        let b1 = x0 - x1;
        let c1 = x1 * y0 - x0 * y1;
        let r3 = a1 * x2 + b1 * y2 + c1;
        let r4 = a1 * x3 + b1 * y3 + c1;
        if r3 * r4 > 0.0 {
            return None;
        }

        let a2 = y3 - y2;
        let b2 = x2 - x3;
        let c2 = x3 * y2 - x2 * y3;
        let r1 = a2 * x0 + b2 * y0 + c2;
        let r2 = a2 * x1 + b2 * y1 + c2;
        if r1 * r2 > 0.0 {
            return None;
        }

        let denom = a1 * b2 - a2 * b1;
        if denom == 0.0 {
            return None;
        }
        let x = (b1 * c2 - b2 * c1) / denom;
        let y = (a2 * c1 - a1 * c2) / denom;
        Some(Site2D::new(x, y))
    }

    /// calculate the nearest point on the line segment with the distance to the point
    pub fn get_nearest_point_on_line_segment(
        &self,
        start: &Site2D,
        end: &Site2D,
    ) -> Option<Site2D> {
        let x0 = self.x;
        let y0 = self.y;
        let x1 = start.x;
        let y1 = start.y;
        let x2 = end.x;
        let y2 = end.y;

        let a = (x0 - x1, y0 - y1);
        let b = (x2 - x1, y2 - y1);
        let dot = a.0 * b.0 + a.1 * b.1;
        let mag_b2 = b.0 * b.0 + b.1 * b.1;
        let distance = dot / mag_b2;
        if !(0.0..=1.0).contains(&distance) {
            return None;
        }
        let proj = (x1 + b.0 * distance, y1 + b.1 * distance);
        Some(Site2D::new(proj.0, proj.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let site1 = Site2D::new(0.0, 0.0);
        let site2 = Site2D::new(3.0, 4.0);
        assert_eq!(site1.distance(&site2), 5.0);
    }

    #[test]
    fn test_distance_between_line_segment() {
        let site = Site2D::new(1.0, 3.0);
        let start = Site2D::new(1.0, 1.0);
        let end = Site2D::new(3.0, 3.0);

        let nearest = site
            .get_nearest_point_on_line_segment(&start, &end)
            .unwrap();
        assert_eq!(nearest, Site2D::new(2.0, 2.0));
    }
    #[test]
    fn test_get_intersection() {
        let start0 = Site2D::new(1.0, 1.0);
        let end0 = Site2D::new(3.0, 3.0);
        let start1 = Site2D::new(1.0, 3.0);
        let end1 = Site2D::new(3.0, 1.0);
        let start2 = Site2D::new(-1.0, 0.0);
        let end2 = Site2D::new(0.0, -1.0);
        assert_eq!(
            start0.get_intersection(&end0, &start1, &end1),
            Some(Site2D::new(2.0, 2.0))
        );
        assert_eq!(
            start1.get_intersection(&end1, &start0, &end0),
            Some(Site2D::new(2.0, 2.0))
        );
        assert_eq!(start0.get_intersection(&end0, &start2, &end2), None);
        assert_eq!(start2.get_intersection(&end2, &start0, &end0), None);
    }
}
