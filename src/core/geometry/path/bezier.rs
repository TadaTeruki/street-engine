use bezier_rs::{Bezier, TValue};
use rstar::PointDistance;

use crate::core::geometry::{angle::Angle, site::Site};

use super::handle::PathBezierHandle;

/// Representation of a bezier curve.
///
/// This is a wrapper around the bezier-rs crate.
#[derive(Debug, Clone, PartialEq)]
pub struct PathBezier {
    curve: Bezier,
}

/// Position on a bezier curve.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PathBezierPosition(f64);

impl PathBezierPosition {
    /// Create a new position on a bezier curve.
    pub fn new(position: f64) -> Self {
        Self(position)
    }
}

impl Eq for PathBezier {}

impl PathBezier {
    /// Create a new linear bezier curve.
    ///
    /// This function is a shortcut for `Path::new` with `PathBezierHandle::Linear`.
    fn new_linear(start: Site, end: Site) -> Self {
        Self::new(start, end, PathBezierHandle::Linear)
    }

    /// Create a new quadratic bezier curve.
    ///
    /// This function is a shortcut for `Path::new` with `PathBezierHandle::Quadratic`.
    pub fn new_quadratic(start: Site, end: Site, handle: Site) -> Self {
        Self::new(start, end, PathBezierHandle::Quadratic(handle))
    }

    /// Create a new cubic bezier curve.
    ///
    /// This function is a shortcut for `Path::new` with `PathBezierHandle::Cubic`.
    fn new_cubic(start: Site, end: Site, handle0: Site, handle1: Site) -> Self {
        Self::new(start, end, PathBezierHandle::Cubic(handle0, handle1))
    }

    /// Create a new bezier curve.
    pub fn new(start: Site, end: Site, handles: PathBezierHandle) -> Self {
        match handles {
            PathBezierHandle::Linear => Self {
                curve: Bezier::from_linear_coordinates(start.x, start.y, end.x, end.y),
            },
            PathBezierHandle::Quadratic(handle) => Self {
                curve: Bezier::from_quadratic_coordinates(
                    start.x, start.y, handle.x, handle.y, end.x, end.y,
                ),
            },
            PathBezierHandle::Cubic(handle0, handle1) => Self {
                curve: Bezier::from_cubic_coordinates(
                    start.x, start.y, handle0.x, handle0.y, handle1.x, handle1.y, end.x, end.y,
                ),
            },
        }
    }

    /// Create a new bezier curve from two 2D vectors.
    pub fn from_2d_vectors(
        site_start: Site,
        vector_start: Option<(Angle, f64)>,
        site_end: Site,
        vector_end: Option<(Angle, f64)>,
    ) -> Self {
        let site_start_term =
            vector_start.map(|(angle, distance)| site_start.extend(angle, distance));

        let site_end_term = vector_end.map(|(angle, distance)| site_end.extend(angle, distance));

        if let (Some(site_start_term), Some(site_end_term)) = (site_start_term, site_end_term) {
            // if both are Some
            if site_start.distance_2(&site_start_term) > site_start.distance_2(&site_end_term)
                || site_end.distance_2(&site_end_term) > site_end.distance_2(&site_start_term)
            {
                // if the path is too short, use linear
                Self::new_linear(site_start, site_end)
            } else {
                Self::new_cubic(site_start, site_end, site_start_term, site_end_term)
            }
        } else if let Some(site_start_term) = site_start_term {
            // if one is Some
            Self::new_quadratic(site_start, site_end, site_start_term)
        } else if let Some(site_end_term) = site_end_term {
            // if one is Some
            Self::new_quadratic(site_start, site_end, site_end_term)
        } else {
            // if both are None
            Self::new_linear(site_start, site_end)
        }
    }

    /// Get the angle at the start and end of the bezier curve.
    pub fn get_angle(&self) -> (Angle, Angle) {
        let handles = self.get_handle();
        let start = Site::new(self.curve.start.x, self.curve.start.y);
        let end = Site::new(self.curve.end.x, self.curve.end.y);
        match handles {
            PathBezierHandle::Linear => (start.get_angle(&end), end.get_angle(&start)),
            PathBezierHandle::Quadratic(handle) => {
                (start.get_angle(&handle), end.get_angle(&handle))
            }
            PathBezierHandle::Cubic(handle0, handle1) => {
                (start.get_angle(&handle0), end.get_angle(&handle1))
            }
        }
    }

    /// Get the handle of the bezier curve.
    pub fn get_handle(&self) -> PathBezierHandle {
        let points = self
            .curve
            .get_points()
            .map(|point| Site::new(point.x, point.y))
            .collect::<Vec<Site>>();
        match points.len() {
            2 => PathBezierHandle::Linear,
            3 => PathBezierHandle::Quadratic(points[1]),
            4 => PathBezierHandle::Cubic(points[1], points[2]),
            _ => unreachable!(),
        }
    }

    /// Get the intersections with another bezier curve.
    ///
    /// Return the intersection points and the parametric position on the curve.
    pub fn get_intersections(&self, other: &Self) -> Vec<(Site, PathBezierPosition)> {
        let intersections = self.curve.intersections(&other.curve, None, None);
        intersections
            .iter()
            .map(|t| (self.curve.evaluate(TValue::Parametric(*t)), *t))
            .map(|(point, t)| (Site::new(point.x, point.y), PathBezierPosition::new(t)))
            .collect::<Vec<(Site, PathBezierPosition)>>()
    }

    /// Split the bezier curve at a given position.
    pub fn split(&self, position: PathBezierPosition) -> (Self, Self) {
        let curve = self.curve.split(TValue::Parametric(position.0));
        (Self { curve: curve[0] }, Self { curve: curve[1] })
    }

    /// Get the projection of a site on the bezier curve.
    ///
    /// Return the projection point and the parametric position on the curve.
    pub fn get_projection(&self, site: &Site) -> Option<(Site, PathBezierPosition)> {
        let projection_ts = self.curve.normals_to_point(glam::DVec2 {
            x: site.x,
            y: site.y,
        });

        projection_ts
            .iter()
            .map(|t| (self.curve.evaluate(TValue::Parametric(*t)), *t))
            .map(|(point, t)| (Site::new(point.x, point.y), PathBezierPosition::new(t)))
            .min_by(|a, b| site.distance_2(&a.0).total_cmp(&site.distance_2(&b.0)))
    }

    /// Get the distance from a site to the bezier curve.
    pub fn get_distance(&self, site: &Site) -> f64 {
        if let Some(projection) = self.get_projection(site) {
            site.distance(&projection.0)
        } else {
            site.distance(&Site::new(self.curve.start.x, self.curve.start.y))
                .min(site.distance(&Site::new(self.curve.end.x, self.curve.end.y)))
        }
    }

    /// Get the bounds of the bezier curve.
    pub fn get_bounds(&self) -> (Site, Site) {
        let bounds = self.curve.bounding_box();
        (
            Site::new(bounds[0].x, bounds[0].y),
            Site::new(bounds[1].x, bounds[1].y),
        )
    }

    /// Get the length of the bezier curve.
    pub fn get_length(&self) -> f64 {
        self.curve.length(Some(10))
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
            line0.get_intersections(&line1).get(0).map(|(site, _)| site),
            Some(Site::new(2.0, 0.0)).as_ref()
        );

        // Vertical and horizontal lines intersecting
        let line0 = PathBezier::new_linear(Site::new(0.0, 1.0), Site::new(4.0, 1.0));
        let line1 = PathBezier::new_linear(Site::new(2.0, 0.0), Site::new(2.0, 3.0));
        assert_eq!(
            line0.get_intersections(&line1).get(0).map(|(site, _)| site),
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
            line0.get_intersections(&line1).get(0).map(|(site, _)| site),
            Some(Site::new(1.4, 3.2)).as_ref()
        );
    }

    #[test]
    fn test_get_projection() {
        let line = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(
            line.get_projection(&site).map(|(site, _)| site),
            Some(Site::new(2.0, 2.0))
        );

        let line = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(2.0, 2.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(
            line.get_projection(&site).map(|(site, _)| site),
            Some(Site::new(2.0, 2.0))
        );

        let line = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(1.0, 2.0));
        let site = Site::new(1.0, 3.0);
        assert_eq!(line.get_projection(&site), None);
    }

    #[test]
    fn test_bounds() {
        // linear
        let line = PathBezier::new_linear(Site::new(1.0, 1.0), Site::new(3.0, 3.0));
        assert_eq!(
            line.get_bounds(),
            (Site::new(1.0, 1.0), Site::new(3.0, 3.0))
        );

        // curved
        let line = PathBezier::from_2d_vectors(
            Site::new(1.0, 1.0),
            Some((Angle::new(0.0), 1.0)),
            Site::new(3.0, 3.0),
            Some((Angle::new(0.0), 1.0)),
        );
        println!("{:?}", line.get_bounds());
        assert!(line.get_bounds().0.y < 1.0);
    }
}
