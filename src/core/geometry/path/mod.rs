use super::{angle::Angle, site::Site};

pub mod bezier;
pub mod handle;

/// Trait for paths.
///
/// `PH` is the type of the path handle.
pub trait PathTrait {
    type Handle: Clone + Eq;

    /// Create a new path with start, end sites and handles.
    fn new(start: Site, end: Site, handle: Self::Handle) -> Self;

    /// Create a new path from two 2D vectors.
    fn from_2d_vectors(
        site_start: Site,
        allow_with_distance_start: Option<(Angle, f64)>,
        site_end: Site,
        allow_with_distance_end: Option<(Angle, f64)>,
    ) -> Self;

    /// Get the handle of the path.
    fn get_handle(&self) -> Self::Handle;

    /// Calculate the intersection of two paths or return None.
    fn get_intersections(&self, other: &Self) -> Vec<Site>;

    /// Calculate the perpendicular projection of the site on the path.
    fn get_projection(&self, site: &Site) -> Option<Site>;

    /// Calculate the distance from the site to the path.
    fn get_distance(&self, site: &Site) -> f64;

    /// Calculate the bounds of the path and return the corner (min, max) sites.
    fn get_bounds(&self) -> (Site, Site);
}
