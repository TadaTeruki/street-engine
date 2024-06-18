use bezier_rs::{Bezier, TValue};

use crate::core::geometry::site::Site;

use super::{handle::PathHandle, PathTrait};

/// Representation of a bezier curve.
///
/// This is a wrapper around the bezier-rs crate.
#[derive(Clone)]
pub struct PathBezier {
    curve: Bezier,
}

impl PathBezier {
    /// Create a new linear bezier curve.
    ///
    /// This function is a shortcut for `Path::new` with `PathHandle::Linear`.
    pub fn new_linear(start: Site, end: Site) -> Self {
        Self::new(start, end, PathHandle::Linear)
    }

    /// Create a new quadratic bezier curve.
    ///
    /// This function is a shortcut for `Path::new` with `PathHandle::Quadratic`.
    pub fn new_quadratic(start: Site, end: Site, handle: Site) -> Self {
        Self::new(start, end, PathHandle::Quadratic(handle))
    }

    /// Create a new cubic bezier curve.
    ///
    /// This function is a shortcut for `Path::new` with `PathHandle::Cubic`.
    pub fn new_cubic(start: Site, end: Site, handle0: Site, handle1: Site) -> Self {
        Self::new(start, end, PathHandle::Cubic(handle0, handle1))
    }
}

impl PathTrait for PathBezier {
    type Handle = PathHandle;

    fn new(start: Site, end: Site, handles: PathHandle) -> Self {
        match handles {
            PathHandle::Linear => Self {
                curve: Bezier::from_linear_coordinates(start.x, start.y, end.x, end.y),
            },
            PathHandle::Quadratic(handle) => Self {
                curve: Bezier::from_quadratic_coordinates(
                    start.x, start.y, handle.x, handle.y, end.x, end.y,
                ),
            },
            PathHandle::Cubic(handle0, handle1) => Self {
                curve: Bezier::from_cubic_coordinates(
                    start.x, start.y, handle0.x, handle0.y, handle1.x, handle1.y, end.x, end.y,
                ),
            },
        }
    }

    fn get_handle(&self) -> PathHandle {
        let points = self
            .curve
            .get_points()
            .map(|point| Site::new(point.x, point.y))
            .collect::<Vec<Site>>();
        match points.len() {
            2 => PathHandle::Linear,
            3 => PathHandle::Quadratic(points[1]),
            4 => PathHandle::Cubic(points[1], points[2]),
            _ => unreachable!(),
        }
    }

    fn get_intersections(&self, other: &Self) -> Vec<Site> {
        let intersections = self.curve.intersections(&other.curve, None, None);
        intersections
            .iter()
            .map(|t| self.curve.evaluate(TValue::Parametric(*t)))
            .map(|point| Site::new(point.x, point.y))
            .collect::<Vec<Site>>()
    }

    fn get_projection(&self, site: &Site) -> Option<Site> {
        let projection_t = self.curve.project(
            glam::DVec2 {
                x: site.x,
                y: site.y,
            },
            None,
        );
        let projection = self
            .curve
            .evaluate(bezier_rs::TValue::Parametric(projection_t));
        Some(Site::new(projection.x, projection.y))
    }

    fn get_distance(&self, site: &Site) -> f64 {
        if let Some(projection) = self.get_projection(site) {
            site.distance(&projection)
        } else {
            site.distance(&Site::new(self.curve.start.x, self.curve.start.y))
                .min(site.distance(&Site::new(self.curve.end.x, self.curve.end.y)))
        }
    }

    fn get_bounds(&self) -> (Site, Site) {
        let bounds = self.curve.bounding_box();
        (
            Site::new(bounds[0].x, bounds[0].y),
            Site::new(bounds[1].x, bounds[1].y),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_intersection() {
        // Parallel lines (no intersection)
        let line0 = PathBezier::new_linear(Site::new(0.0, 0.0), Site::new(2.0, 2.0));
        let line1 = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        assert_eq!(line0.get_intersections(&line1).get(0), None);

        // Collinear overlapping lines
        // This is expected to return None, as the intersection is not a point.
        let line0 = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        let line1 = PathBezier::new_linear(Site::new(2.0, 2.0), Site::new(4.0, 4.0));
        assert_eq!(line0.get_intersections(&line1).get(0), None);

        // Intersecting at an end point
        let line0 = PathBezier::new_linear(Site::new(0.0, 0.0), Site::new(2.0, 0.0));
        let line1 = PathBezier::new_linear(Site::new(2.0, 0.0), Site::new(2.0, 2.0));
        assert_eq!(
            line0.get_intersections(&line1).get(0),
            Some(Site::new(2.0, 0.0)).as_ref()
        );

        // Vertical and horizontal lines intersecting
        let line0 = PathBezier::new_linear(Site::new(0.0, 1.0), Site::new(4.0, 1.0));
        let line1 = PathBezier::new_linear(Site::new(2.0, 0.0), Site::new(2.0, 3.0));
        assert_eq!(
            line0.get_intersections(&line1).get(0),
            Some(Site::new(2.0, 1.0)).as_ref()
        );

        // Collinear lines that barely touch by their edges
        let line0 = PathBezier::new_linear(Site::new(0.0, 0.0), Site::new(1.0, 1.0));
        let line1 = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        assert_eq!(line0.get_intersections(&line1).get(0), None);

        // Lines with no intersection (completely separate)
        let line0 = PathBezier::new_linear(Site::new(0.0, 0.0), Site::new(1.0, 1.0));
        let line1 = PathBezier::new_linear(Site::new(2.0, 2.0), Site::new(3.0, 3.0));
        assert_eq!(line0.get_intersections(&line1).get(0), None);

        // Edge case: Zero-length line segment
        // This is expected to return None.
        let line0 = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(1.0, 1.0));
        let line1 = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        assert_eq!(line0.get_intersections(&line1).get(0), None);

        // Intersecting at a point
        let line0 = PathBezier::new_linear(Site::new(1.0, 3.0), Site::new(3.0, 4.0));
        let line1 = PathBezier::new_linear(Site::new(1.0, 4.0), Site::new(2.0, 2.0));
        assert_eq!(
            line0.get_intersections(&line1).get(0),
            Some(Site::new(1.4, 3.2)).as_ref()
        );
    }

    #[test]
    fn test_get_projection() {
        let line = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(line.get_projection(&site), Some(Site::new(2.0, 2.0)));

        let line = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(line.get_projection(&site), Some(Site::new(2.0, 2.0)));

        let line = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(1.0, 2.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(line.get_projection(&site), None);
    }
}
