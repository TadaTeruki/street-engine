use gtk4::{cairo::Context, DrawingArea};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::{map::ParticleMap, ParticleParameters};

use super::{
    region::{create_region_map_from_flatness, create_region_map_from_region, RegionAttributes},
    section::{create_section_place_map, SectionAttributes},
    PlaceMap,
};

pub struct PlaceMapCollection {
    section: Vec<PlaceMap<SectionAttributes>>,
    region: Vec<PlaceMap<RegionAttributes>>,
}

impl PlaceMapCollection {
    pub fn new(
        elevation_map: &ParticleMap<f64>,
        flatness_map: &ParticleMap<f64>,
        region_scales: &[f64],
        section_scale: f64,
    ) -> Self {
        let region_params = region_scales
            .iter()
            .map(|&scale| ParticleParameters {
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

        // let section_params = ParticleParameters {
        //     scale: elevation_map.params().scale,
        //     min_randomness: 0.8,
        //     max_randomness: 0.8,
        //     ..Default::default()
        // };

        // let evaluation_weights = vec![
        //     0.0,
        //     1.0,
        //     0.0,
        //     0.0,
        //     0.0,
        //     0.0,
        // ];
        // let section_place_map = create_section_place_map(
        //     section_params,
        //     flatness_map,
        //     &region_place_maps,
        //     evaluation_weights,
        //     [0.6, 0.4, 0.0],
        // );

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

        Self {
            section: section_place_maps,
            region: region_place_maps,
        }
    }
}

impl Layer for PlaceMapCollection {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let section_color = [0.6, 0.4, 0.0];

        let section = &self.section[3];
        section.draw(drawing_area, cr, focus_range, section_color);

        let region_colors = [
            [0.9, 0.5, 0.3],
            [0.9, 0.9, 0.5],
            [0.0, 0.5, 0.6],
            [0.5, 0.0, 1.0],
            [1.0, 0.0, 0.5],
            [1.0, 1.0, 1.0],
        ];

        let region = &self.region[3];
        region.draw(drawing_area, cr, focus_range, region_colors[3]);
    }
}
