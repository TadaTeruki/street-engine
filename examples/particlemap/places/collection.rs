use gtk4::{cairo::Context, DrawingArea};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::{map::ParticleMap, ParticleParameters};

use super::{
    quarter::{QuarterAttributes, QuarterPlaceNodeEstimator},
    PlaceMap,
};

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
