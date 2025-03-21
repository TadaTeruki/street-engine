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
    RegionMap {
        child_region_map: &'a PlaceMap<RegionAttributes>,
        elevation_map: &'a ParticleMap<f64>,
    },
}

impl<'a> RegionPlaceNodeEstimator<'a> {
    pub fn get_params(&self) -> ParticleParameters {
        match self {
            Self::FlatnessMap(flatness_map) => *flatness_map.params(),
            Self::RegionMap {
                child_region_map, ..
            } => *child_region_map.map.params(),
        }
    }

    pub fn get_score(&self, particle: Particle) -> Option<f64> {
        match self {
            Self::FlatnessMap(flatness_map) => flatness_map.get(&particle).copied(),
            Self::RegionMap {
                child_region_map, ..
            } => {
                let child_region_node = child_region_map.map.get(&particle)?;
                Some(child_region_node.attributes.habitablity_rate)
            }
        }
    }

    pub fn get_core(&self, particle: Particle) -> Option<(f64, f64)> {
        match self {
            Self::FlatnessMap(_) => Some(particle.site()),
            Self::RegionMap {
                child_region_map, ..
            } => Some(child_region_map.map.get(&particle)?.core),
        }
    }
}

impl<'a> PlaceNodeEstimator<RegionAttributes> for RegionPlaceNodeEstimator<'a> {
    fn estimate(&self, place_particle: Particle) -> Option<PlaceNode<RegionAttributes>> {
        let score_params = self.get_params();

        let score_particles = match self {
            Self::FlatnessMap(_) => Particle::from_inside_particle(score_params, place_particle),
            Self::RegionMap { elevation_map, .. } => {
                // ignore ocean to handle coastal region
                Particle::from_inside_particle(score_params, place_particle)
                    .into_iter()
                    .filter(|particle| {
                        let site = particle.site();
                        elevation_map
                            .get(&Particle::from(site.0, site.1, *elevation_map.params()))
                            .is_some()
                    })
                    .collect::<Vec<_>>()
            }
        };

        let len_score_particles = score_particles.len();

        let sum_score = score_particles
            .iter()
            .filter_map(|score_particle| self.get_score(*score_particle))
            .sum::<f64>();

        let habitablity_rate = sum_score / len_score_particles as f64;

        match self {
            Self::FlatnessMap(_) | Self::RegionMap { .. } => {
                let mut score_centroid_x = 0.0;
                let mut score_centroid_y = 0.0;

                score_particles
                    .iter()
                    .filter_map(|score_particle| {
                        Some((
                            self.get_core(*score_particle)?,
                            self.get_score(*score_particle)?,
                        ))
                    })
                    .for_each(|(core_particle, score)| {
                        score_centroid_x += core_particle.0 * score;
                        score_centroid_y += core_particle.1 * score;
                    });

                let score_centroid = (score_centroid_x / sum_score, score_centroid_y / sum_score);

                // evaluation: score * (1 - distance to centroid/scale)
                let evaluate = |core_particle: (f64, f64), score: f64| {
                    let distance = ((core_particle.0 - score_centroid.0).powi(2)
                        + (core_particle.1 - score_centroid.1).powi(2))
                    .sqrt();
                    score * (1.0 - distance / score_params.scale)
                };

                let mut max_evaluation = 0.0;
                let mut core = None;

                score_particles
                    .iter()
                    .filter_map(|score_particle| {
                        Some((
                            self.get_core(*score_particle)?,
                            self.get_score(*score_particle)?,
                        ))
                    })
                    .for_each(|(core_particle, score)| {
                        let evaluation = evaluate(core_particle, score);
                        if core.is_none() || evaluation > max_evaluation {
                            max_evaluation = evaluation;
                            core = Some(core_particle);
                        }
                    });

                Some(PlaceNode {
                    core: core?,
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
    child_region_map: &PlaceMap<RegionAttributes>,
    elevation_map: &ParticleMap<f64>,
    color: [f64; 3],
) -> PlaceMap<RegionAttributes> {
    let region_place_map = PlaceMap::new(
        params,
        RegionPlaceNodeEstimator::RegionMap {
            child_region_map,
            elevation_map,
        },
        &child_region_map.map,
        color,
    );

    region_place_map
}
