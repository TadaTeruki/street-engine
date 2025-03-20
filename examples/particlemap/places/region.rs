use worley_particle::{map::ParticleMap, Particle, ParticleParameters};

use super::{PlaceMap, PlaceNode, PlaceNodeAttributes, PlaceNodeEstimator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionAttributes {
    pub habitablity_rate: f64,
}

impl PlaceNodeAttributes for RegionAttributes {
    fn alpha(&self) -> f64 {
        self.habitablity_rate
    }
}

pub enum RegionPlaceNodeEstimator<'a> {
    FlatnessMap(&'a ParticleMap<f64>),
    RegionMap(&'a PlaceMap<RegionAttributes>),
}

impl<'a> RegionPlaceNodeEstimator<'a> {
    pub fn get_params(&self) -> ParticleParameters {
        match self {
            Self::FlatnessMap(flatness_map) => *flatness_map.params(),
            Self::RegionMap(region_map) => *region_map.map.params(),
        }
    }

    pub fn get_score(&self, particle: Particle) -> Option<f64> {
        match self {
            Self::FlatnessMap(flatness_map) => flatness_map.get(&particle).copied(),
            Self::RegionMap(region_map) => region_map
                .map
                .get(&particle)
                .map(|node| node.attributes.habitablity_rate),
        }
    }
}

impl<'a> PlaceNodeEstimator<RegionAttributes> for RegionPlaceNodeEstimator<'a> {
    fn estimate(&self, place_particle: Particle) -> Option<PlaceNode<RegionAttributes>> {
        let score_params = self.get_params();
        let score_particles = Particle::from_inside_particle(score_params, place_particle);

        let len_score_particles = score_particles.len();

        let sum_score = score_particles
            .iter()
            .filter_map(|score_particle| self.get_score(*score_particle))
            .sum::<f64>();

        let habitablity_rate = sum_score / len_score_particles as f64;

        match self {
            Self::FlatnessMap(_) | Self::RegionMap(_) => {
                let mut max_score = 0.0;
                let mut core_particle = None;

                for score_particle in score_particles {
                    if let Some(score) = self.get_score(score_particle) {
                        if core_particle.is_none() || score > max_score {
                            max_score = score;
                            core_particle = Some(score_particle);
                        }
                    }
                }

                Some(PlaceNode {
                    core: core_particle?.site(),
                    attributes: RegionAttributes { habitablity_rate },
                })
            }
        }
    }
}

pub fn create_region_map_from_flatness(
    params: ParticleParameters,
    flatness_map: &ParticleMap<f64>,
    color: [f64; 3],
) -> PlaceMap<RegionAttributes> {
    let region_place_map = PlaceMap::new(
        params,
        RegionPlaceNodeEstimator::FlatnessMap(flatness_map),
        &flatness_map,
        color,
    );

    region_place_map
}

pub fn create_region_map_from_region(
    params: ParticleParameters,
    region_map: &PlaceMap<RegionAttributes>,
    color: [f64; 3],
) -> PlaceMap<RegionAttributes> {
    let region_place_map = PlaceMap::new(
        params,
        RegionPlaceNodeEstimator::RegionMap(region_map),
        &region_map.map,
        color,
    );

    region_place_map
}
