use std::{collections::HashMap, fmt::Debug};

use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use rayon::iter::{ParallelBridge, ParallelIterator};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::{map::ParticleMap, Particle, ParticleParameters};

pub mod collection;
mod quarter;
mod region;

pub trait PlaceNodeAttributes: Debug + Clone + Copy + PartialEq + Send + Sync {
    fn alpha(&self) -> f64;
}

pub trait PlaceNodeEstimator<T: PlaceNodeAttributes>: Sync {
    fn estimate(&self, particle: Particle) -> Option<PlaceNode<T>>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceNode<T: PlaceNodeAttributes> {
    pub core: (f64, f64),
    pub attributes: T,
}

pub struct PlaceMap<T: PlaceNodeAttributes> {
    map: ParticleMap<PlaceNode<T>>,
    color: [f64; 3],
}

impl<T: PlaceNodeAttributes> PlaceMap<T> {
    pub fn new<E: PlaceNodeEstimator<T>, U: Debug + Clone + Copy + PartialEq + Sync>(
        place_particle_param: ParticleParameters,
        place_node_estimator: E,
        base_map: &ParticleMap<U>,
        color: [f64; 3],
    ) -> Self {
        let place_hashmap = base_map
            .iter()
            .par_bridge()
            .filter_map(|(base_particle, _)| {
                let (x, y) = base_particle.site();
                let place_particle = Particle::from(x, y, place_particle_param);
                Some((
                    place_particle,
                    place_node_estimator.estimate(place_particle)?,
                ))
            })
            .collect::<HashMap<_, _>>();

        let particle_map = ParticleMap::new(place_particle_param, place_hashmap);

        Self {
            map: particle_map,
            color,
        }
    }
}

impl<T: PlaceNodeAttributes> Layer for PlaceMap<T> {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let area_width = drawing_area.width();
        let area_height = drawing_area.height();

        let rect = focus_range.to_rect(area_width as f64, area_height as f64);

        for (_, node) in self.map.iter() {
            let x = rect.map_coord_x(node.core.0, 0.0, area_width as f64);
            let y = rect.map_coord_y(node.core.1, 0.0, area_height as f64);

            cr.arc(
                x,
                y,
                self.map.params().scale * 30.0 + 3.0,
                0.0,
                2.0 * std::f64::consts::PI,
            );

            // cr.set_source_rgba(self.color[0], self.color[1], self.color[2], 0.5);
            // cr.set_line_width(1.0);
            // cr.stroke_preserve().expect("Failed to draw edge");

            cr.set_source_rgba(
                self.color[0],
                self.color[1],
                self.color[2],
                node.attributes.alpha(),
            );

            cr.fill().expect("Failed to draw place");
        }
    }
}
