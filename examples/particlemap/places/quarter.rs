use worley_particle::{
    map::{network::ParticleNetwork, IDWStrategy, ParticleMap},
    Particle, ParticleParameters,
};

use super::{
    region::RegionAttributes, PlaceMap, PlaceNode, PlaceNodeAttributes, PlaceNodeEstimator,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuarterAttributes {
    pub evaluation: f64,
}

impl PlaceNodeAttributes for QuarterAttributes {
    fn alpha(&self) -> f64 {
        self.evaluation
    }
}

pub struct QuarterPlaceNodeEstimator<'a> {
    pub region_maps: &'a Vec<PlaceMap<RegionAttributes>>,
    pub networks: Vec<ParticleNetwork>,
}

fn dot(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

fn dot_linearized(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dot = dot(a, b);
    let angle = dot.acos();
    1.0 - angle / std::f64::consts::PI
}

fn normalize(a: (f64, f64)) -> (f64, f64) {
    let norm = (a.0 * a.0 + a.1 * a.1).sqrt();
    (a.0 / norm, a.1 / norm)
}

fn calculate_dot_evaluation(
    from: (f64, f64),
    to: (f64, f64),
    to_neighbor: (f64, f64),
    range_cos: f64,
) -> f64 {
    let vec_from = normalize((to.0 - from.0, to.1 - from.1));

    let neighbor = normalize((to.0 - to_neighbor.0, to.1 - to_neighbor.1));
    (dot_linearized(vec_from, neighbor) - (1.0 - range_cos)).max(0.0) / range_cos
}

impl<'a> PlaceNodeEstimator<QuarterAttributes> for QuarterPlaceNodeEstimator<'a> {
    fn estimate(&self, place_particle: Particle) -> Option<PlaceNode<QuarterAttributes>> {
        let region_map = self.region_maps.last()?;

        let site = place_particle.site();
        let region_idw_weights = region_map.map.calculate_idw_weights(
            site.0,
            site.1,
            &IDWStrategy::default_from_params(region_map.map.params()),
        )?;

        let mut dot_evaluation = 0.0;
        let mut mean_neighbors_len = 0.0;

        for (region_particle, weight) in region_idw_weights.iter() {
            let region_node = if let Some(node) = region_map.map.get(region_particle) {
                node
            } else {
                continue;
            };

            let neighbors = region_particle
                .calculate_voronoi()
                .neighbors
                .iter()
                .filter_map(|neighbor| Some(region_map.map.get(neighbor)?))
                .collect::<Vec<_>>();

            mean_neighbors_len += neighbors.len() as f64 * weight;

            let unit_dot_evaluation = neighbors
                .iter()
                .map(|neighbor| {
                    calculate_dot_evaluation(site, region_node.core, neighbor.core, 0.1)
                        * neighbor.attributes.habitablity_rate
                })
                .sum::<f64>()
                / neighbors.len() as f64;

            dot_evaluation +=
                unit_dot_evaluation * region_node.attributes.habitablity_rate * weight;
        }

        let evaluation = dot_evaluation * mean_neighbors_len;

        Some(PlaceNode {
            core: site,
            attributes: QuarterAttributes { evaluation },
        })
    }
}

pub fn create_quarter_place_map(
    params: ParticleParameters,
    flatness_map: &ParticleMap<f64>,
    region_maps: &Vec<PlaceMap<RegionAttributes>>,
    color: [f64; 3],
) -> PlaceMap<QuarterAttributes> {
    let networks = region_maps
        .iter()
        .map(|region_map| ParticleNetwork::new(&region_map.map))
        .collect::<Vec<_>>();

    let quarter_place_map = PlaceMap::new(
        params,
        QuarterPlaceNodeEstimator {
            region_maps,
            networks,
        },
        &flatness_map,
        color,
    );

    quarter_place_map
}
