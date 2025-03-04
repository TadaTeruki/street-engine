use std::{cell::RefCell, rc::Rc};

use drainage_basin_builder::map::DrainageMap;
use factor::FactorsMap;
use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use vislayers::{
    geometry::FocusRange,
    window::{Layer, Visualizer},
};
use worley_particle::map::ParticleMap;

mod bands;
mod factor;

struct DrainageMapWrapped(DrainageMap);

impl Layer for DrainageMapWrapped {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let area_width = drawing_area.width();
        let area_height = drawing_area.height();

        let rect = focus_range.to_rect(area_width as f64, area_height as f64);

        for (_, node) in self.0.particle_map().iter() {
            let river_width = node.river_width(self.0.river_strength());
            if river_width < self.0.river_ignoreable_width() {
                continue;
            }
            let iter_num = (0.1 / focus_range.radius()).ceil() as usize;

            let point_0 = node.main_river.evaluate(0.0);
            let x0 = rect.map_coord_x(point_0.0, 0.0, area_width as f64);
            let y0 = rect.map_coord_y(point_0.1, 0.0, area_height as f64);

            cr.move_to(x0, y0);

            for i in 1..(iter_num + 1) {
                let t = i as f64 / iter_num as f64;

                let point_1 = node.main_river.evaluate(t);
                let x1 = rect.map_coord_x(point_1.0, 0.0, area_width as f64);
                let y1 = rect.map_coord_y(point_1.1, 0.0, area_height as f64);

                cr.line_to(x1, y1);
            }

            cr.set_line_width(
                river_width / focus_range.radius() / self.0.particle_map().params().scale,
            );
            cr.set_source_rgb(0.0, 0.0, 1.0);
            cr.set_line_cap(gtk4::cairo::LineCap::Round);
            cr.stroke().expect("Failed to draw edge");
        }
    }
}

fn main() {
    let particlemap_dir = "examples/particlemap/data";
    let particlemap_id = "11008264925851530191";
    let elevation_path = format!("{}/{}.particlemap", particlemap_dir, particlemap_id);
    let elevation_map =
        ParticleMap::<f64>::read_from_file(&elevation_path).expect("Error reading terrain map");
    let factors_map = FactorsMap::from_elevation_map(elevation_map, 0.0025);

    let drainage_path = format!(
        "{}/drainage-{}.particlemap",
        particlemap_dir, particlemap_id
    );
    let drainage_map = DrainageMap::load_from_file(&drainage_path, 1.0, 0.01);

    let mut visualizer = Visualizer::new(800, 600);
    visualizer.add_layer(Rc::new(RefCell::new(factors_map)), 0);
    visualizer.add_layer(Rc::new(RefCell::new(DrainageMapWrapped(drainage_map))), 1);
    visualizer.run();
}
