/// A unit of measurement for elevation.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Elevation(f64);

impl Elevation {
    pub fn new(elevation: f64) -> Self {
        Self(elevation)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Eq for Elevation {}

impl PartialOrd for Elevation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Elevation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}
