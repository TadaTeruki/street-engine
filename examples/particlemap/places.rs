use std::collections::HashMap;

use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::{map::ParticleMap, Particle, ParticleParameters};

pub trait PlaceNodeEstimator {
    fn estimate(&self, particle: Particle) -> Option<PlaceNode>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceNode {
    pub core: (f64, f64),
    pub evaluation: f64,
}

pub struct PlaceMap {
    map: ParticleMap<Option<PlaceNode>>,
    color: [u8; 4],
}

impl PlaceMap {
    pub fn new<E: PlaceNodeEstimator>(
        place_particle_param: ParticleParameters,
        color: [u8; 4],
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

        Self {
            map: particle_map,
            color,
        }
    }
}

impl Layer for PlaceMap {
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

            cr.set_source_rgba(
                self.color[0] as f64 / 255.0,
                self.color[1] as f64 / 255.0,
                self.color[2] as f64 / 255.0,
                self.color[3] as f64 / 255.0,
            );
            cr.arc(x, y, 5.0, 0.0, 2.0 * std::f64::consts::PI);

            cr.fill().expect("Failed to draw place");
        }
    }
}
