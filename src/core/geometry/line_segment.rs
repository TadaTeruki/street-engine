use super::site::Site;

/// Representation of a line segment.
#[derive(Debug, Clone)]
pub struct LineSegment(pub Site, pub Site);

impl LineSegment {
    /// Create a line segment from two sites.
    pub fn new(start: Site, end: Site) -> Self {
        Self(start, end)
    }

    /// Calculate the intersection of two line segments.
    /// If the intersection is outside the line segments or not exist, return None.
    pub fn get_intersection(&self, other: &Self) -> Option<Site> {
        let (x0, y0) = (self.0.x, self.0.y);
        let (x1, y1) = (self.1.x, self.1.y);
        let (x2, y2) = (other.0.x, other.0.y);
        let (x3, y3) = (other.1.x, other.1.y);

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
    pub fn get_projection(&self, site: &Site) -> Option<Site> {
        let (x0, y0) = (site.x, site.y);
        let (x1, y1) = (self.0.x, self.0.y);
        let (x2, y2) = (self.1.x, self.1.y);

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

impl PartialEq for LineSegment {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_intersection() {
        // Parallel lines (no intersection)
        let line0 = LineSegment::new(Site::new(0.0, 0.0), Site::new(2.0, 2.0));
        let line1 = LineSegment::new(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        assert_eq!(line0.get_intersection(&line1), None);

        // Collinear overlapping lines
        // This is expected to return None, as the intersection is not a point.
        let line0 = LineSegment::new(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        let line1 = LineSegment::new(Site::new(2.0, 2.0), Site::new(4.0, 4.0));
        assert_eq!(line0.get_intersection(&line1), None);

        // Intersecting at an end point
        let line0 = LineSegment::new(Site::new(0.0, 0.0), Site::new(2.0, 0.0));
        let line1 = LineSegment::new(Site::new(2.0, 0.0), Site::new(2.0, 2.0));
        assert_eq!(line0.get_intersection(&line1), Some(Site::new(2.0, 0.0)));

        // Vertical and horizontal lines intersecting
        let line0 = LineSegment::new(Site::new(0.0, 1.0), Site::new(4.0, 1.0));
        let line1 = LineSegment::new(Site::new(2.0, 0.0), Site::new(2.0, 3.0));
        assert_eq!(line0.get_intersection(&line1), Some(Site::new(2.0, 1.0)));

        // Collinear lines that barely touch by their edges
        let line0 = LineSegment::new(Site::new(0.0, 0.0), Site::new(1.0, 1.0));
        let line1 = LineSegment::new(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        assert_eq!(line0.get_intersection(&line1), None);

        // Lines with no intersection (completely separate)
        let line0 = LineSegment::new(Site::new(0.0, 0.0), Site::new(1.0, 1.0));
        let line1 = LineSegment::new(Site::new(2.0, 2.0), Site::new(3.0, 3.0));
        assert_eq!(line0.get_intersection(&line1), None);

        // Edge case: Zero-length line segment
        // This is expected to return None.
        let line0 = LineSegment::new(Site::new(1.0, 1.0), Site::new(1.0, 1.0));
        let line1 = LineSegment::new(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        assert_eq!(line0.get_intersection(&line1), None);

        // Intersecting at a point
        let line0 = LineSegment::new(Site::new(1.0, 3.0), Site::new(3.0, 4.0));
        let line1 = LineSegment::new(Site::new(1.0, 4.0), Site::new(2.0, 2.0));
        assert_eq!(line0.get_intersection(&line1), Some(Site::new(1.4, 3.2)));
    }

    #[test]
    fn test_get_projection() {
        let line = LineSegment::new(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(line.get_projection(&site), Some(Site::new(2.0, 2.0)));

        let line = LineSegment::new(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(line.get_projection(&site), Some(Site::new(2.0, 2.0)));

        let line = LineSegment::new(Site::new(1.0, 1.0), Site::new(1.0, 2.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(line.get_projection(&site), None);
    }
}
