use crate::core::geometry::site::Site;

/// Handles for bezier curves.
///
/// This is a wrapper around the bezier-rs crate.
#[derive(Clone)]
pub enum PathHandle {
    Linear,
    Quadratic(Site),
    Cubic(Site, Site),
}

impl PartialEq for PathHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PathHandle::Linear, PathHandle::Linear) => true,
            (PathHandle::Quadratic(handle0), PathHandle::Quadratic(handle1)) => handle0 == handle1,
            (PathHandle::Cubic(handle0, handle1), PathHandle::Cubic(handle2, handle3)) => {
                (handle0, handle1) == (handle2, handle3) || (handle0, handle1) == (handle3, handle2)
            }
            _ => false,
        }
    }
}

impl Eq for PathHandle {}
