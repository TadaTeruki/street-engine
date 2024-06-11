use factors::{calculate_population_density, create_terrain};
use graphics::write_to_image;
use map_provider::MapProvider;
use naturalneighbor::Interpolator;
use random::RandomF64;
use rules_provider::{railway::RulesProviderForRailway, road::RulesProviderForRoad};
use street_engine::{
    core::geometry::site::Site,
    transport::{
        builder::TransportBuilder,
        path_network_repository::{PathNetworkGroup, PathNetworkRepository},
    },
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
    let rules_provider_road: RulesProviderForRoad = RulesProviderForRoad::new(&map_provider);
    let rules_provider_railway = RulesProviderForRailway::new(&map_provider);

    let mut rnd = RandomF64::new();

    let mut network_repository = PathNetworkRepository::new();
    let network_id = network_repository.create_network(PathNetworkGroup::new(0));

    let mut builder = network_repository
        .modify_network(network_id, |network| {
            TransportBuilder::new(
                &rules_provider_railway,
                &map_provider,
                &rules_provider_railway,
            )
            .add_origin(network, Site { x: 0.0, y: 0.0 }, 0.0)
            .unwrap()
        })
        .unwrap();

    while let Some(stump) = builder.pop_stump() {
        let growth = if let Some(growth) = builder.determine_growth_from_stump(
            &network_repository,
            network_repository.get_network(network_id).unwrap(),
            &stump,
        ) {
            growth
        } else {
            return;
        };

        network_repository.modify_network(network_id, |network| {
            builder.apply_next_growth(
                &mut rnd,
                network,
                growth.next_node,
                growth.bridge_node,
                stump.get_node_id(),
                stump.get_path_params(),
            )
        });
    }

    println!("Writing to image...");

    write_to_image(
        bound_min,
        bound_max,
        img_width,
        img_height,
        &terrain,
        &network_repository.get_network(network_id).unwrap(),
        &population_densities,
        filename,
    );
}
