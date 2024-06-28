use fastlem::{
    core::{parameters::TopographicalParameters, traits::Model},
    lem::generator::TerrainGenerator,
    models::surface::{builder::TerrainModel2DBulider, model::TerrainModel2D, terrain::Terrain2D},
};
use noise::{NoiseFn, Perlin};
use street_engine::core::geometry::site::Site;
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;

use crate::map_provider::into_fastlem_site;

pub fn create_terrain(
    node_num: usize,
    seed: u32,
    bound_min: Site,
    bound_max: Site,
) -> (Terrain2D, Vec<bool>, EdgeAttributedUndirectedGraph<f64>) {
    let model = TerrainModel2DBulider::from_random_sites(
        node_num,
        into_fastlem_site(bound_min),
        into_fastlem_site(bound_max),
    )
    .relaxate_sites(2)
    .unwrap()
    .add_edge_sites(None, None)
    .unwrap()
    .build()
    .unwrap();

    let graph = model.graph().clone();

    let (terrain, is_outlet) = generate_terrain(seed, model, bound_min, bound_max);

    (terrain, is_outlet, graph)
}

fn octaved_perlin(perlin: &Perlin, x: f64, y: f64, octaves: usize, persistence: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += perlin.get([x * frequency, y * frequency]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    value / max_value
}

pub fn calculate_population_density(
    terrain: &Terrain2D,
    graph: &EdgeAttributedUndirectedGraph<f64>,
    is_outlet: &Vec<bool>,
) -> Vec<f64> {
    let max_slope_livable = std::f64::consts::PI / 3.0;
    let slopes = (0..terrain.sites().len())
        .map(|i| {
            let slopes = graph
                .neighbors_of(i)
                .iter()
                .map(|neighbor| {
                    let distance = neighbor.1;
                    let slope =
                        (terrain.elevations()[i] - terrain.elevations()[neighbor.0]) / distance;
                    slope.atan()
                })
                .collect::<Vec<_>>();
            slopes
        })
        .collect::<Vec<_>>();

    let densities = (0..terrain.sites().len())
        .map(|i| {
            if is_outlet[i] {
                return 0.0;
            }
            let slope_sum = slopes[i].iter().fold(0.0, |acc, slope| acc + slope.abs());
            let slope_avg = slope_sum.abs() / slopes[i].len() as f64;
            (1.0 - slope_avg / max_slope_livable).max(0.0).min(1.0)
        })
        .collect::<Vec<_>>();

    densities
}

fn generate_terrain(
    seed: u32,
    model: TerrainModel2D,
    bound_min: Site,
    bound_max: Site,
) -> (Terrain2D, Vec<bool>) {
    let sites = model.sites().to_vec();

    let perlin = Perlin::new(seed);

    let (erodibility, is_outlet) = {
        let mut erodibility = vec![0.; sites.len()];
        let mut is_outlet = vec![false; sites.len()];

        (0..sites.len()).for_each(|i| {
            let site = sites[i];
            let octaves = 5;
            let x = site.x / (bound_max.x - bound_min.x);
            let y = site.y / (bound_max.y - bound_min.y);
            let dist_from_center = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
            let x = site.x / 100.;
            let y = site.y / 100.;

            let noise_erodibility = (1.0
                - (octaved_perlin(
                    &perlin,
                    x * 1.2 + (bound_max.x - bound_min.x),
                    y * 1.2 + (bound_max.y - bound_min.y),
                    octaves,
                    0.5,
                )
                .abs()
                    + octaved_perlin(&perlin, x * 1.2, y * 1.2, octaves, 0.5).abs()))
            .max(0.0)
            .powi(5)
                * 2.5
                + 0.1;

            let noise_is_outlet = (octaved_perlin(&perlin, x * 1.0, y * 1.0, octaves, 0.5) * 0.5
                + 0.5)
                * dist_from_center
                * 1.0
                + (1.0 - dist_from_center) * 0.5;
            erodibility[i] = noise_erodibility;
            is_outlet[i] = noise_is_outlet > 0.55;
        });

        (erodibility, is_outlet)
    };

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_parameters(
            (0..sites.len())
                .map(|i| {
                    TopographicalParameters::default()
                        .set_erodibility(erodibility[i])
                        .set_is_outlet(is_outlet[i])
                })
                .collect::<_>(),
        )
        .generate()
        .unwrap();

    (terrain, is_outlet)
}
