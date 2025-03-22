// use worley_particle::{map::ParticleMap, ParticleParameters};

// use super::{section::SectionAttributes, PlaceMap};

// pub struct UsageProvider<'a> {
//     usage: Vec<(String, ParticleMap<f64>)>,
//     params: ParticleParameters,
//     sections: &'a Vec<PlaceMap<SectionAttributes>>,
// }

// impl<'a> UsageProvider<'a> {
//     pub fn new(sections: &'a Vec<PlaceMap<SectionAttributes>>, params: ParticleParameters) -> Self {
//         UsageProvider {
//             usage: Vec::new(),
//             params,
//             sections,
//         }
//     }

//     pub fn push_usage<D: FnOnce(f64, f64, &Vec<PlaceMap<SectionAttributes>>) -> f64>(&mut self, name: String, descriptor: D) {
//         // let map = ParticleMap
//     }
// }
