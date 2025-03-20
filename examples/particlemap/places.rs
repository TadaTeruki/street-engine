use std::{collections::HashMap, fmt::Debug};

use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::{map::ParticleMap, Particle, ParticleParameters};

pub trait PlaceNodeAttributes: Debug + Clone + Copy + PartialEq {
    fn color(&self) -> [f64; 4];
}

pub trait PlaceNodeEstimator<T: PlaceNodeAttributes> {
    fn estimate(&self, particle: Particle) -> Option<PlaceNode<T>>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceNode<T: PlaceNodeAttributes> {
    pub core: (f64, f64),
    pub attributes: T,
}

pub struct PlaceMap<T: PlaceNodeAttributes> {
    map: ParticleMap<Option<PlaceNode<T>>>,
}

impl<T: PlaceNodeAttributes> PlaceMap<T> {
    pub fn new<E: PlaceNodeEstimator<T>>(
        place_particle_param: ParticleParameters,
        place_node_estimator: E,
        base_map: &ParticleMap<f64>,
    ) -> Self {
        let mut place_hashmap = HashMap::new();

        base_map.iter().for_each(|(base_particle, _)| {
            let (x, y) = base_particle.site();
            let place_particle = Particle::from(x, y, place_particle_param);
            place_hashmap.insert(
                place_particle,
                place_node_estimator.estimate(place_particle),
            );
        });

        let particle_map = ParticleMap::new(place_particle_param, place_hashmap);

        Self { map: particle_map }
    }
}

impl<T: PlaceNodeAttributes> Layer for PlaceMap<T> {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let area_width = drawing_area.width();
        let area_height = drawing_area.height();

        let rect = focus_range.to_rect(area_width as f64, area_height as f64);

        for (_, node) in self.map.iter() {
            let node = if let Some(node) = node {
                node
            } else {
                continue;
            };

            let x = rect.map_coord_x(node.core.0, 0.0, area_width as f64);
            let y = rect.map_coord_y(node.core.1, 0.0, area_height as f64);

            let color = node.attributes.color();

            cr.set_source_rgba(color[0], color[1], color[2], color[3]);

            cr.arc(x, y, 5.0, 0.0, 2.0 * std::f64::consts::PI);

            cr.fill().expect("Failed to draw place");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuarterAttributes {
    pub flatness: f64,
}

impl PlaceNodeAttributes for QuarterAttributes {
    fn color(&self) -> [f64; 4] {
        [0.8, 0.5, 0.0, self.flatness]
    }
}

struct QuarterPlaceNodeEstimator<'a> {
    flatness_map: &'a ParticleMap<f64>,
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

pub struct PlaceMapCollection {
    quarter: PlaceMap<QuarterAttributes>,
}

impl PlaceMapCollection {
    pub fn new(elevation_map: &ParticleMap<f64>, flatness_map: &ParticleMap<f64>) -> Self {
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

        Self {
            quarter: quarter_place_map,
        }
    }
}

impl Layer for PlaceMapCollection {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        self.quarter.draw(drawing_area, cr, focus_range);
    }
}
