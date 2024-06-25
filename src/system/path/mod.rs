use crate::{
    core::{
        container::path_network::PathNetworkPathTrait,
        geometry::{
            path::{
                bezier::{PathBezier, PathBezierPosition},
                handle::PathBezierHandle,
            },
            site::Site,
        },
    },
    unit::Length,
};

pub mod factor;

/// A path in the transport network.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportPath {
    curve: PathBezier,
}

impl PathNetworkPathTrait for TransportPath {
    type Handle = PathBezierHandle;
    type Position = PathBezierPosition;

    /// Create a new path.
    fn new(start: Site, end: Site, handle: Self::Handle) -> Self {
        Self {
            curve: PathBezier::new(start, end, handle),
        }
    }

    fn get_distance(&self, site: &Site) -> f64 {
        self.curve.get_distance(site)
    }

    fn get_bounds(&self) -> (Site, Site) {
        self.curve.get_bounds()
    }
}

impl TransportPath {
    fn get_intersections(&self, other: &Self) -> Vec<(Site, PathBezierPosition)> {
        self.curve.get_intersections(&other.curve)
    }

    fn get_projection(&self, site: &Site) -> Option<(Site, PathBezierPosition)> {
        self.curve.get_projection(site)
    }

    fn split(&self, position: PathBezierPosition) -> (Self, Self)
    where
        Self: Sized,
    {
        let (curve0, curve1) = self.curve.split(position);
        (Self { curve: curve0 }, Self { curve: curve1 })
    }

    fn get_length(&self) -> Length {
        Length::new(self.curve.get_length())
    }
}
