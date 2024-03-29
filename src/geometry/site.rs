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
    fn new(x: f64, y: f64) -> Self {
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

    /// Calculate the intersection of two line segments.
    fn get_intersection(&self, end0: &Self, start1: &Self, end1: &Self) -> Option<Self> {
        let (x0, y0) = (self.x, self.y);
        let (x1, y1) = (end0.x, end0.y);
        let (x2, y2) = (start1.x, start1.y);
        let (x3, y3) = (end1.x, end1.y);

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
        Some(Site::new(x, y))
    }

    /// Calculate the perpendicular projection of the site on the line segment.
    /// If the projection is outside the line segment, return None.
    fn get_projection_on_line_segment(&self, start: &Self, end: &Self) -> Option<Self> {
        let (x0, y0) = (self.x, self.y);
        let (x1, y1) = (start.x, start.y);
        let (x2, y2) = (end.x, end.y);

        let a = (x0 - x1, y0 - y1);
        let b = (x2 - x1, y2 - y1);
        let dot = a.0 * b.0 + a.1 * b.1;
        let mag_b2 = b.0 * b.0 + b.1 * b.1;
        let distance = dot / mag_b2;

        if !(0.0..=1.0).contains(&distance) {
            return None;
        }
        let proj = (x1 + b.0 * distance, y1 + b.1 * distance);
        Some(Site::new(proj.0, proj.1))
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
    fn test_get_projection_on_line_segment() {
        let site = Site::new(1.0, 3.0);
        let start = Site::new(1.0, 1.0);
        let end = Site::new(3.0, 3.0);

        let projection = site.get_projection_on_line_segment(&start, &end);
        assert_eq!(projection, Some(Site::new(2.0, 2.0)));

        let site = Site::new(1.0, 3.0);
        let start = Site::new(1.0, 1.0);
        let end = Site::new(2.0, 2.0);

        let projection = site.get_projection_on_line_segment(&start, &end);
        assert_eq!(projection, Some(Site::new(2.0, 2.0)));

        let site = Site::new(1.0, 3.0);
        let start = Site::new(1.0, 1.0);
        let end = Site::new(1.0, 2.0);

        let projection = site.get_projection_on_line_segment(&start, &end);
        assert_eq!(projection, None);
    }
    #[test]
    fn test_get_intersection() {
        let start0 = Site::new(1.0, 1.0);
        let end0 = Site::new(3.0, 3.0);
        let start1 = Site::new(1.0, 3.0);
        let end1 = Site::new(3.0, 1.0);
        let start2 = Site::new(-1.0, 0.0);
        let end2 = Site::new(0.0, -1.0);
        assert_eq!(
            start0.get_intersection(&end0, &start1, &end1),
            Some(Site::new(2.0, 2.0))
        );
        assert_eq!(
            start1.get_intersection(&end1, &start0, &end0),
            Some(Site::new(2.0, 2.0))
        );
        assert_eq!(start0.get_intersection(&end0, &start2, &end2), None);
        assert_eq!(start2.get_intersection(&end2, &start0, &end0), None);
    }
}
