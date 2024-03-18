use std::f64::consts::PI;

use city_engine::generator::{NetworkConfig, NetworkGenerator};
use city_engine::geom::Site2D;
use city_engine::interface::{ElevationModel, PopulationDensityModel};
use city_engine::model::Network;
use fastlem::core::{parameters::TopographicalParameters, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::builder::TerrainModel2DBulider;
use fastlem::models::surface::model::TerrainModel2D;
use fastlem::models::surface::terrain::Terrain2D;
use naturalneighbor::Interpolator;
use noise::{NoiseFn, Perlin};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;
use tiny_skia::{Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

struct MapModel {
    pub elevations: Vec<f64>,
    pub population_densities: Vec<f64>,
    pub interpolator: Interpolator,
}

impl ElevationModel for MapModel {
    fn get_elevation(&self, site: Site2D) -> Option<f64> {
        self.interpolator
            .interpolate(
                &self.elevations,
                naturalneighbor::Point {
                    x: site.x,
                    y: site.y,
                },
            )
            .unwrap_or(None)
    }
}

impl PopulationDensityModel for MapModel {
    fn get_population_density(&self, site: Site2D) -> Option<f64> {
        self.interpolator
            .interpolate(
                &self.population_densities,
                naturalneighbor::Point {
                    x: site.x,
                    y: site.y,
                },
            )
            .unwrap_or(None)
    }
}

fn main() {
    let node_num = 20000;
    let seed = 14;
    let bound_min = Site2D {
        x: -100.0,
        y: -50.0,
    };
    let bound_max = Site2D { x: 100.0, y: 50.0 };
    let img_width = 2000;
    let img_height = 1000;
    let filename = "modelcase.png";

    let (terrain, is_outlet, graph) = create_terrain(node_num, seed, bound_min, bound_max);

    let population_densities = calculate_population_density(&terrain, &graph, &is_outlet);

    let model = MapModel {
        elevations: terrain.elevations().to_vec(),
        population_densities,
        interpolator: Interpolator::new(
            &terrain
                .sites()
                .iter()
                .map(|site| naturalneighbor::Point {
                    x: site.x,
                    y: site.y,
                })
                .collect::<Vec<_>>(),
        ),
    };

    let constructor = NetworkGenerator::new(
        seed as u64,
        &model,
        &model,
        NetworkConfig {
            max_straight_angle: PI / 128.0,
            road_length: 0.4,
            road_comparison_step: 3,
            road_evaluation: |population_density, _| population_density,
            road_branch_probability_by_evaluation: |_| 0.4,
            merge_node_distance: 0.2,
        },
    )
    .add_origin(Site2D { x: 0.0, y: 0.0 }, 1.57)
    .iterate_n_times(1000);

    let network = constructor.snapshot();

    write_to_image(
        bound_min,
        bound_max,
        img_width,
        img_height,
        &terrain,
        network,
        &model.population_densities,
        filename,
    );
}

fn write_to_image(
    bound_min: Site2D,
    bound_max: Site2D,
    img_width: u32,
    img_height: u32,
    terrain: &Terrain2D,
    network: &Network,
    population_densities: &Vec<f64>,
    filename: &str,
) {
    let sites = terrain.sites();
    let interpolator = Interpolator::new(
        &sites
            .iter()
            .map(|site| naturalneighbor::Point {
                x: site.x,
                y: site.y,
            })
            .collect::<Vec<_>>(),
    );

    let mut pixmap = Pixmap::new(img_width, img_height).unwrap();
    let mut paint = Paint::default();

    for imgx in 0..img_width {
        for imgy in 0..img_height {
            let x = bound_min.x
                + (bound_max.x - bound_min.x) * ((imgx as f64 + 0.5) / img_width as f64);
            let y = bound_min.y
                + (bound_max.y - bound_min.y) * ((imgy as f64 + 0.5) / img_height as f64);
            let site = Site2D { x, y };
            let elevation = terrain.get_elevation(&into_fastlem_site(site));
            let population_density = interpolator.interpolate(
                &population_densities,
                naturalneighbor::Point {
                    x: site.x,
                    y: site.y,
                },
            );
            if let (Some(elevation), Ok(Some(population_density))) = (elevation, population_density)
            {
                let color = get_color(elevation, population_density);
                paint.set_color_rgba8(color[0], color[1], color[2], 255);
                pixmap.fill_rect(
                    Rect::from_xywh(imgx as f32, imgy as f32, 1.0, 1.0).unwrap(),
                    &paint,
                    Transform::identity(),
                    None,
                );
            }
        }
    }

    let stroke = Stroke {
        width: 1.3,
        ..Default::default()
    };

    (0..network.nodes().len()).for_each(|i| {
        network.for_each_neighbor(i, |j| {
            let site_a = network.nodes()[i].site;
            let site_b = network.nodes()[j].site;
            let x_a = (site_a.x - bound_min.x) / (bound_max.x - bound_min.x) * img_width as f64;
            let y_a = (site_a.y - bound_min.y) / (bound_max.y - bound_min.y) * img_height as f64;
            let x_b = (site_b.x - bound_min.x) / (bound_max.x - bound_min.x) * img_width as f64;
            let y_b = (site_b.y - bound_min.y) / (bound_max.y - bound_min.y) * img_height as f64;
            let path = {
                let mut path = PathBuilder::new();
                path.move_to(x_a as f32, y_a as f32);
                path.line_to(x_b as f32, y_b as f32);
                path.finish().unwrap()
            };

            paint.set_color_rgba8(0, 0, 0, 50);
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        });
    });

    pixmap.save_png(filename).unwrap();
}

// The below functions are for generating other data

fn create_terrain(
    node_num: usize,
    seed: u32,
    bound_min: Site2D,
    bound_max: Site2D,
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

fn into_fastlem_site(site: Site2D) -> fastlem::models::surface::sites::Site2D {
    fastlem::models::surface::sites::Site2D {
        x: site.x,
        y: site.y,
    }
}

fn calculate_population_density(
    terrain: &Terrain2D,
    graph: &EdgeAttributedUndirectedGraph<f64>,
    is_outlet: &Vec<bool>,
) -> Vec<f64> {
    let max_slope_livable = PI / 4.5;
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

    let mut densities = (0..terrain.sites().len())
        .map(|i| {
            if is_outlet[i] {
                return 0.0;
            }
            let slope_sum = slopes[i].iter().fold(0.0, |acc, slope| acc + slope.abs());
            let slope_avg = slope_sum.abs() / slopes[i].len() as f64;
            (1.0 - slope_avg / max_slope_livable).max(0.0).min(1.0)
        })
        .collect::<Vec<_>>();

    loop {
        let mut next_densities = vec![0.0; terrain.sites().len()];

        densities.iter().enumerate().for_each(|(i, density)| {
            if is_outlet[i] || *density == 0.0 {
                next_densities[i] = *density;
                return;
            }
            let drain_prop = density;
            let densities_sum = graph.neighbors_of(i).iter().fold(0.0, |acc, neighbor| {
                if is_outlet[i] {
                    return acc;
                }
                let neighbor_density = densities[neighbor.0];
                acc + neighbor_density
            });
            graph.neighbors_of(i).iter().for_each(|neighbor| {
                if is_outlet[i] || densities_sum == 0.0 {
                    return;
                }
                let neighbor_density = densities[neighbor.0];
                let density_prop = neighbor_density / densities_sum;
                let final_prop = drain_prop * density_prop;
                next_densities[neighbor.0] += final_prop * density;
            });
            next_densities[i] += density * (1.0 - drain_prop);
        });

        let mut is_same = true;
        for i in 0..densities.len() {
            if (densities[i] - next_densities[i]).abs() > 0.01 {
                is_same = false;
                break;
            }
        }
        if is_same {
            break;
        }

        densities = next_densities;
    }

    densities
}

fn generate_terrain(
    seed: u32,
    model: TerrainModel2D,
    bound_min: Site2D,
    bound_max: Site2D,
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

fn get_color(elevation: f64, population_density: f64) -> [u8; 3] {
    let colormap: [([u8; 3], f64); 6] = [
        ([70, 150, 200], 0.0),
        ([70, 150, 200], 0.05),
        ([240, 240, 210], 0.125),
        ([190, 200, 120], 0.5),
        ([25, 100, 25], 25.0),
        ([15, 60, 15], 40.0),
    ];
    let populated_color = [250, 0, 0];
    let color_index = {
        let mut i = 0;
        while i < colormap.len() {
            if elevation < colormap[i].1 {
                break;
            }
            i += 1;
        }
        i
    };

    let land_color = if color_index == 0 {
        colormap[0].0
    } else if color_index == colormap.len() {
        colormap[colormap.len() - 1].0
    } else {
        let color_a = colormap[color_index - 1];
        let color_b = colormap[color_index];

        let prop = (elevation - color_a.1) / (color_b.1 - color_a.1);
        blend_color(color_a.0, color_b.0, prop)
    };
    //land_color
    blend_color(land_color, populated_color, population_density)
}

fn blend_color(color_a: [u8; 3], color_b: [u8; 3], prop: f64) -> [u8; 3] {
    [
        (color_a[0] as f64 + (color_b[0] as f64 - color_a[0] as f64) * prop) as u8,
        (color_a[1] as f64 + (color_b[1] as f64 - color_a[1] as f64) * prop) as u8,
        (color_a[2] as f64 + (color_b[2] as f64 - color_a[2] as f64) * prop) as u8,
    ]
}
