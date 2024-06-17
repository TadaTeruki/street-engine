use crate::unit::Elevation;

/// The type of a node in the transport network.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportNodeType {
    /// A node on the land.
    Land,

    /// A node which creates a bridge, overpass or tunnel.
    /// The elevation is relative to the land.
    ///
    /// If the associated elevation is positive, it is a bridge or overpass.
    /// If the associated elevation is negative, it is a tunnel.
    Bridge(Elevation),
}

impl TransportNodeType {
    /// Create a new land node.
    pub fn as_land() -> Self {
        Self::Land
    }

    /// Create a new bridge node.
    pub fn as_bridge(elevation: Elevation) -> Self {
        Self::Bridge(elevation)
    }

    /// Get the absolute elevation of the node.
    pub fn absoulte_elevation(&self, land_elevation: Elevation) -> Elevation {
        match self {
            Self::Land => Elevation::new(0.0 + land_elevation.value()),
            Self::Bridge(elevation) => Elevation::new(elevation.value() + land_elevation.value()),
        }
    }
}
