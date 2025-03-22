use std::collections::HashMap;

use gtk4::{cairo::Context, DrawingArea};
use vislayers::geometry::FocusRange;
use worley_particle::{map::ParticleMap, ParticleParameters};

use super::{
    region::{create_region_map_from_flatness, create_region_map_from_region, RegionAttributes},
    section::{create_section_place_map, SectionAttributes},
    PlaceMap,
};

pub struct PlaceMapCollection {
    section: HashMap<String, PlaceMap<SectionAttributes>>,
    region: HashMap<String, PlaceMap<RegionAttributes>>,
}

impl PlaceMapCollection {
    pub fn new(
        elevation_map: &ParticleMap<f64>,
        flatness_map: &ParticleMap<f64>,
        region_scales: &[(String, f64)],
        section_scale: f64,
        //usage_fns: Vec<&dyn FnOnce(Vec<PlaceMap<SectionAttributes>>) -> f64>, 
    ) -> Self {
        let region_params = region_scales
            .iter()
            .map(|&(_, scale)| ParticleParameters {
                scale,
                min_randomness: 0.8,
                max_randomness: 0.8,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let mut region_place_maps = Vec::new();

        for (i, params) in region_params.iter().enumerate() {
            if i == 0 {
                region_place_maps.push(create_region_map_from_flatness(*params, flatness_map));
            } else {
                region_place_maps.push(create_region_map_from_region(
                    *params,
                    &region_place_maps[i - 1],
                    elevation_map,
                ));
            }
        }

        let section_params = ParticleParameters {
            scale: section_scale,
            min_randomness: 0.8,
            max_randomness: 0.8,
            ..Default::default()
        };

        let section_place_maps = region_place_maps
            .iter()
            .map(|region_map| create_section_place_map(section_params, flatness_map, region_map))
            .collect::<Vec<_>>();

        //let usage_maps = usage_fns.iter().map(|f| f(section_place_maps)).collect::<Vec<_>>();

        let scale_names = region_scales.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();

        Self {
            section: scale_names
                .iter()
                .cloned()
                .zip(section_place_maps.into_iter())
                .collect::<HashMap<_, _>>(),
            region: scale_names
                .iter()
                .cloned()
                .zip(region_place_maps.into_iter())
                .collect::<HashMap<_, _>>(),
            //usage: usage_maps,
        }
    }

    pub fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange, draw_sections: &[String], draw_regions: &[String]) {
        let section_color = [0.6, 0.4, 0.0];
        draw_sections.iter().filter_map(|name| self.section.get(name)).for_each(|section| section.draw(drawing_area, cr, focus_range, section_color));
        let region_color = [1.0, 1.0, 1.0];
        draw_regions.iter().filter_map(|name| self.region.get(name)).for_each(|region| region.draw(drawing_area, cr, focus_range, region_color));
    }
}
