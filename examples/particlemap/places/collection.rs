use gtk4::{cairo::Context, DrawingArea};
use vislayers::{geometry::FocusRange, window::Layer};
use worley_particle::map::ParticleMap;

use super::{
    quarter::{create_quarter_place_map, QuarterAttributes},
    PlaceMap,
};

pub struct PlaceMapCollection {
    quarter: PlaceMap<QuarterAttributes>,
}

impl PlaceMapCollection {
    pub fn new(elevation_map: &ParticleMap<f64>, flatness_map: &ParticleMap<f64>) -> Self {
        let quarter_place_map = create_quarter_place_map(elevation_map, flatness_map);

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
