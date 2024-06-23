use crate::core::geometry::site::Site;

/// Handles for bezier curves.
///
/// This is a wrapper around the bezier-rs crate.
#[derive(Debug, Copy, Clone)]
pub enum PathBezierHandle {
    Linear,
    Quadratic(Site),
    Cubic(Site, Site),
}

impl PartialEq for PathBezierHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PathBezierHandle::Linear, PathBezierHandle::Linear) => true,
            (PathBezierHandle::Quadratic(handle0), PathBezierHandle::Quadratic(handle1)) => {
                handle0 == handle1
            }
            (
                PathBezierHandle::Cubic(handle0, handle1),
                PathBezierHandle::Cubic(handle2, handle3),
            ) => {
                (handle0, handle1) == (handle2, handle3) || (handle0, handle1) == (handle3, handle2)
            }
            _ => false,
        }
    }
}

impl Eq for PathBezierHandle {}
