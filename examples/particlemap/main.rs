use std::{cell::RefCell, rc::Rc};

use drainage_basin_builder::map::DrainageMap;
use factor::FactorsMap;
use vislayers::window::Visualizer;
use worley_particle::map::ParticleMap;

mod bands;
mod factor;
mod habitability;

fn main() {
    let particlemap_dir = "examples/particlemap/data";
    let particlemap_id = "11008264925851530191";

    let elevation_path = format!("{}/{}.particlemap", particlemap_dir, particlemap_id);
    let elevation_map =
        ParticleMap::<f64>::read_from_file(&elevation_path).expect("Error reading terrain map");

    let drainage_path = format!(
        "{}/drainage-{}.particlemap",
        particlemap_dir, particlemap_id
    );
    let drainage_map = DrainageMap::load_from_file(&drainage_path, 1.0, 0.01);

    let habitablity_path = format!(
        "{}/habitability-{}.particlemap",
        particlemap_dir, particlemap_id
    );
    let habitability_map = ParticleMap::<f64>::read_from_file(&habitablity_path).ok();
    let habitability_map_is_none = habitability_map.is_none();

    let factors_map = FactorsMap::new(elevation_map, drainage_map, habitability_map, 0.0025);

    if habitability_map_is_none {
        factors_map
            .habitability_map()
            .write_to_file(&habitablity_path)
            .expect("Error writing habitability map");
    }

    let mut visualizer = Visualizer::new(800, 600);
    visualizer.add_layer(Rc::new(RefCell::new(factors_map)), 0);
    visualizer.run();
}
