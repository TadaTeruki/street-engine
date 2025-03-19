use std::collections::HashMap;

use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::{map::ParticleMap, Particle, ParticleParameters};

use crate::disjoint_set::DisjointSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaceNode {
    core_particle: Option<Particle>,
    habitability: f64,
}

pub struct UnitPlaceMap {
    map: ParticleMap<PlaceNode>,
    color: [u8; 4],
}

impl UnitPlaceMap {
    pub fn new(
        place_particle_param: ParticleParameters,
        color: [u8; 4],
        base_map: &ParticleMap<f64>,
        habitability_map: &ParticleMap<f64>,
    ) -> Self {
        let mut place_hashmap = HashMap::new();

        base_map.iter().for_each(|(base_particle, _)| {
            let (x, y) = base_particle.site();
            let place_particle = Particle::from(x, y, place_particle_param);
            place_hashmap.entry(place_particle).or_insert(PlaceNode {
                core_particle: None,
                habitability: 0.0,
            });
        });

        habitability_map
            .iter()
            .for_each(|(habitability_particle, habitability)| {
                let (x, y) = habitability_particle.site();
                let place_particle = Particle::from(x, y, place_particle_param);
                // if the habitability is higher than existing habitability or None, update the place particle
                if let Some(node) = place_hashmap.get(&place_particle) {
                    if *habitability > node.habitability {
                        place_hashmap.insert(
                            place_particle,
                            PlaceNode {
                                core_particle: Some(*habitability_particle),
                                habitability: *habitability,
                            },
                        );
                    }
                }
            });

        Self {
            map: particle_map,
            color,
        }
    }
}

impl Layer for UnitPlaceMap {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let area_width = drawing_area.width();
        let area_height = drawing_area.height();

        let rect = focus_range.to_rect(area_width as f64, area_height as f64);

        for (_, node) in self.map.iter() {
            let core_particle = if let Some(core_particle) = node.core_particle {
                core_particle
            } else {
                continue;
            };

            let x = rect.map_coord_x(core_particle.site().0, 0.0, area_width as f64);
            let y = rect.map_coord_y(core_particle.site().1, 0.0, area_height as f64);

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

pub struct PlaceMapCollection {
    maps: Vec<UnitPlaceMap>,
}

impl PlaceMapCollection {
    pub fn new(
        place_particle_params: Vec<(ParticleParameters, [u8; 4])>,
        base_map: &ParticleMap<f64>,
        habitability_map: &ParticleMap<f64>,
    ) -> Self {
        let maps = place_particle_params
            .iter()
            .map(|(param, color)| {
                UnitPlaceMap::new(param.clone(), color.clone(), base_map, habitability_map)
            })
            .collect::<Vec<UnitPlaceMap>>();
        Self { maps }
    }
}

impl Layer for PlaceMapCollection {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        self.maps
            .iter()
            .for_each(|map| map.draw(drawing_area, cr, focus_range));
    }
}
