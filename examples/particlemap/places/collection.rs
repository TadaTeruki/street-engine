use gtk4::{cairo::Context, DrawingArea};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::{map::ParticleMap, ParticleParameters};

use super::{
    quarter::{create_quarter_place_map, QuarterAttributes},
    region::{create_region_map_from_flatness, create_region_map_from_region, RegionAttributes},
    PlaceMap,
};

pub struct PlaceMapCollection {
    quarter: PlaceMap<QuarterAttributes>,
    region: Vec<PlaceMap<RegionAttributes>>,
}

impl PlaceMapCollection {
    pub fn new(elevation_map: &ParticleMap<f64>, flatness_map: &ParticleMap<f64>) -> Self {
        let region_params = (0..6)
            .map(|i| ParticleParameters {
                scale: elevation_map.params().scale * 2.0 * 2.0_f64.powi(i as i32),
                min_randomness: 0.8,
                max_randomness: 0.8,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let mut region_place_maps = Vec::new();

        let colors = [
            [0.9, 0.5, 0.3],
            [0.9, 0.9, 0.5],
            [0.0, 0.5, 0.6],
            [0.5, 0.0, 1.0],
            [1.0, 0.0, 0.5],
            [1.0, 1.0, 1.0],
        ];

        for (i, params) in region_params.iter().enumerate() {
            if i == 0 {
                region_place_maps.push(create_region_map_from_flatness(
                    *params,
                    flatness_map,
                    colors[i],
                ));
            } else {
                region_place_maps.push(create_region_map_from_region(
                    *params,
                    &region_place_maps[i - 1],
                    elevation_map,
                    colors[i],
                ));
            }
        }

        let quarter_params = ParticleParameters {
            scale: elevation_map.params().scale,
            min_randomness: 0.8,
            max_randomness: 0.8,
            ..Default::default()
        };

        let quarter_place_map = create_quarter_place_map(
            quarter_params,
            flatness_map,
            &region_place_maps,
            [0.6, 0.4, 0.0],
        );

        Self {
            quarter: quarter_place_map,
            region: region_place_maps,
        }
    }
}

impl Layer for PlaceMapCollection {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        self.quarter.draw(drawing_area, cr, focus_range);

        // for region in &self.region {
        //     region.draw(drawing_area, cr, focus_range);
        // }
        let region = &self.region.last().unwrap();
        region.draw(drawing_area, cr, focus_range);
    }
}
