use super::PlaceMap;

pub trait UsageDescriptor: FnOnce(Vec<PlaceMap<f64>>) -> f64 {}

pub struct UsageProvider {
    usage: Vec<PlaceMap<f64>>,
}

// impl UsageProvider {
// }