use factors::{calculate_population_density, create_terrain};
use graphics::write_to_image;
use map_provider::MapProvider;
use naturalneighbor::Interpolator;
use random::RandomF64;
use rules_provider::{railway::RulesProviderForRailway, road::RulesProviderForRoad};
use street_engine::{
    core::geometry::site::Site,
    transport::{builder::TransportBuilder, params::numeric::Stage},
};

mod factors;
mod graphics;
mod map_provider;
mod random;
mod rules_provider;

fn main() {
    let node_num = 50000;
    let seed = 0;
    let bound_min = Site {
        x: -100.0,
        y: -50.0,
    };
    let bound_max = Site { x: 100.0, y: 50.0 };
    let img_width = 2400;
    let img_height = 1200;
    let filename = "modelcase.png";

    println!("Creating terrain...");

    let (terrain, is_outlet, graph) = create_terrain(node_num, seed, bound_min, bound_max);

    let max_elevation = terrain.elevations().iter().cloned().fold(0.0, f64::max);
    println!("Max elevation: {}", max_elevation);

    println!("Calculating population densities...");

    let sites = terrain.sites();
    let population_densities = calculate_population_density(&terrain, &graph, &is_outlet);

    println!("Creating network...");

    let interpolator = Interpolator::new(
        &sites
            .iter()
            .map(|site| naturalneighbor::Point {
                x: site.x,
                y: site.y,
            })
            .collect::<Vec<_>>(),
    );

    let map_provider = MapProvider::new(&terrain, &population_densities, interpolator);
    let rules_provider_road = RulesProviderForRoad::new(&map_provider);
    //let rules_provider_railway = RulesProviderForRailway::new(&map_provider);

    let mut rnd = RandomF64::new();

    let network = TransportBuilder::new(&rules_provider_road, &map_provider, &rules_provider_road)
        .add_origin(Site { x: 0.0, y: 0.0 }, 0.0, Some(Stage::from_num(1)))
        .unwrap()
        .iterate_as_possible(&mut rnd)
        .snapshot()
        .0;

    println!("Writing to image...");

    write_to_image(
        bound_min,
        bound_max,
        img_width,
        img_height,
        &terrain,
        &network,
        &population_densities,
        filename,
    );
}
