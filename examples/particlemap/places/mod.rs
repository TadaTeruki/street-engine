use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use rayon::iter::{ParallelBridge, ParallelIterator};
use vislayers::geometry::FocusRange;
use worley_particle::{map::ParticleMap, Particle, ParticleParameters};

pub mod collection;
mod usage;
mod region;
mod section;

pub trait PlaceNodeAttributes: Debug + Clone + Copy + PartialEq + Send + Sync {
    fn alpha(&self) -> f64;
}

impl PlaceNodeAttributes for f64 {
    fn alpha(&self) -> f64 {
        return *self
    }
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
}

impl<T: PlaceNodeAttributes> PlaceMap<T> {
    pub fn new<E: PlaceNodeEstimator<T>, U: Debug + Clone + Copy + PartialEq + Sync>(
        place_particle_param: ParticleParameters,
        place_node_estimator: E,
        base_map: &ParticleMap<U>,
    ) -> Self {
        let places = base_map
            .iter()
            .par_bridge()
            .map(|(base_particle, _)| {
                let site = base_particle.site();
                Particle::from(site.0, site.1, place_particle_param)
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let place_hashmap = places
            .iter()
            .filter_map(|particle| {
                let node = place_node_estimator.estimate(*particle)?;
                Some((*particle, node))
            })
            .collect::<HashMap<_, _>>();

        let particle_map = ParticleMap::new(place_particle_param, place_hashmap);

        Self { map: particle_map }
    }
}

impl<T: PlaceNodeAttributes> PlaceMap<T> {
    fn draw(
        &self,
        drawing_area: &DrawingArea,
        cr: &Context,
        focus_range: &FocusRange,
        color: [f64; 3],
    ) {
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

            cr.set_source_rgba(color[0], color[1], color[2], 0.2);
            cr.set_line_width(1.0);
            cr.stroke_preserve().expect("Failed to draw edge");

            cr.set_source_rgba(color[0], color[1], color[2], node.attributes.alpha());

            cr.fill().expect("Failed to draw place");
        }
    }
}
