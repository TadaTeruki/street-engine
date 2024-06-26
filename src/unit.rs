use std::ops::Add;

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

/// A unit of measurement for length.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Length(f64);

impl Length {
    pub fn new(length: f64) -> Self {
        Self(length)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Eq for Length {}

impl PartialOrd for Length {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Length {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}
