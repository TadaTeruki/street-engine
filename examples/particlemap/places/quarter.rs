use worley_particle::{map::ParticleMap, Particle, ParticleParameters};

use super::{PlaceMap, PlaceNode, PlaceNodeAttributes, PlaceNodeEstimator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuarterAttributes {
    pub flatness: f64,
}

impl PlaceNodeAttributes for QuarterAttributes {
    fn color(&self) -> [f64; 4] {
        [0.8, 0.5, 0.0, self.flatness]
    }
}

pub struct QuarterPlaceNodeEstimator<'a> {
    pub flatness_map: &'a ParticleMap<f64>,
}

impl<'a> PlaceNodeEstimator<QuarterAttributes> for QuarterPlaceNodeEstimator<'a> {
    fn estimate(&self, place_particle: Particle) -> Option<PlaceNode<QuarterAttributes>> {
        let flatness_params = self.flatness_map.params();
        let flatness_particles = Particle::from_inside_particle(*flatness_params, place_particle);
        let mut max_flatness = 0.0;
        let mut flatness_core_particle = None;

        for flatness_particle in flatness_particles {
            let flatness = if let Some(flatness) = self.flatness_map.get(&flatness_particle) {
                *flatness
            } else {
                continue;
            };

            if flatness_core_particle.is_none() || flatness > max_flatness {
                max_flatness = flatness;
                flatness_core_particle = Some(flatness_particle);
            }
        }

        Some(PlaceNode {
            core: flatness_core_particle?.site(),
            attributes: QuarterAttributes {
                flatness: max_flatness,
            },
        })
    }
}

pub fn create_quarter_place_map(
    elevation_map: &ParticleMap<f64>,
    flatness_map: &ParticleMap<f64>,
) -> PlaceMap<QuarterAttributes> {
    let place_map_base_params = ParticleParameters {
        scale: elevation_map.params().scale * 2.0,
        min_randomness: 0.8,
        max_randomness: 0.8,
        seed: 324,
        ..Default::default()
    };

    let quarter_place_map = PlaceMap::new(
        place_map_base_params,
        QuarterPlaceNodeEstimator {
            flatness_map: &flatness_map,
        },
        &flatness_map,
    );

    quarter_place_map
}
