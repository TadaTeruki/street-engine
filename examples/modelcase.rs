use fastlem::core::{parameters::TopographicalParameters, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::builder::TerrainModel2DBulider;
use fastlem::models::surface::model::TerrainModel2D;
use fastlem::models::surface::terrain::Terrain2D;
use naturalneighbor::Interpolator;
use noise::{NoiseFn, Perlin};
use rand::SeedableRng;
use rayon::prelude::*;
use street_engine::core::container::path_network::PathNetwork;
use street_engine::core::geometry::angle::Angle;
use street_engine::core::geometry::site::Site;
use street_engine::core::{Group, Stage};
use street_engine::transport::builder::TransportBuilder;
use street_engine::transport::node::transport_node::TransportNode;
use street_engine::transport::params::evaluation::PathEvaluationFactors;
use street_engine::transport::params::metrics::PathMetrics;
use street_engine::transport::params::rules::{
    BranchRules, BridgeRules, PathDirectionRules, TransportRules,
};
use street_engine::transport::traits::{
    PathEvaluator, RandomF64Provider, TerrainProvider, TransportRulesProvider,
};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;
use tiny_skia::{Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

struct MapProvider<'a> {
    terrain: &'a Terrain2D,
    population_densities: &'a Vec<f64>,
    interpolator: Interpolator,
}

impl<'a> MapProvider<'a> {
    fn new(
        terrain: &'a Terrain2D,
        population_densities: &'a Vec<f64>,
        interpolator: Interpolator,
    ) -> Self {
        Self {
            terrain,
            population_densities,
            interpolator,
        }
    }

    fn get_population_density(&self, site: &Site) -> Option<f64> {
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

impl<'a> TerrainProvider for MapProvider<'a> {
    fn get_elevation(&self, site: &Site) -> Option<f64> {
        let elevation = self.terrain.get_elevation(&into_fastlem_site(*site))?;
        if elevation < 1e-1 {
            return None;
        }
        return Some(elevation);
    }
}

impl<'a> PathEvaluator for MapProvider<'a> {
    fn evaluate(&self, factor: PathEvaluationFactors) -> Option<f64> {
        let site = factor.site_end;
        let elevation = self.terrain.get_elevation(&into_fastlem_site(site))?;
        let population_density = self.get_population_density(&site)?;

        let path_priority = (1e-9 + population_density) * (-elevation);

        let stage = factor.stage;
        if stage.as_num() > 0 {
            return Some(path_priority);
        } else {
            return Some(path_priority + 1e5);
        }
    }
}

impl<'a> TransportRulesProvider for MapProvider<'a> {
    fn get_rules(
        &self,
        site: &Site,
        _: Angle,
        stage: Stage,
        metrics: &PathMetrics,
    ) -> Option<TransportRules> {
        let population_density = self.get_population_density(site)?;
        let is_street = stage.as_num() > 0;

        let path_normal_length = if metrics.branch_count % 2 == 0 {
            0.35
        } else {
            0.45
        };

        if is_street {
            // street
            Some(TransportRules {
                path_normal_length,
                path_extra_length_for_intersection: path_normal_length * 0.7,
                path_max_elevation_diff: None,
                branch_rules: BranchRules {
                    branch_density: 0.01 + population_density * 0.99,
                    staging_probability: 0.0,
                },
                path_direction_rules: PathDirectionRules {
                    max_radian: std::f64::consts::PI / (5.0 + 1000.0 * population_density),
                    comparison_step: 3,
                },
                bridge_rules: BridgeRules::default(),
            })
        } else {
            // highway
            Some(TransportRules {
                path_normal_length,
                path_extra_length_for_intersection: path_normal_length * 0.7,
                path_max_elevation_diff: Some(10.0),
                branch_rules: BranchRules {
                    branch_density: 0.2 + population_density * 0.8,
                    staging_probability: 0.97,
                },
                path_direction_rules: PathDirectionRules {
                    max_radian: std::f64::consts::PI / (10.0 + 100.0 * population_density),
                    comparison_step: 3,
                },
                bridge_rules: BridgeRules {
                    max_bridge_length: 25.0,
                    check_step: 15,
                },
            })
        }
    }
}

struct RandomF64<R> {
    rng: R,
}

impl<R: rand::Rng> RandomF64<R> {
    fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl<R: rand::Rng> RandomF64Provider for RandomF64<R> {
    fn gen_f64(&mut self) -> f64 {
        self.rng.gen()
    }
}

fn main() {
    let node_num = 50000;
    let seed = 20;
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

    let mut rnd = RandomF64::new(rand::rngs::StdRng::seed_from_u64(0));

    let network = TransportBuilder::new(&map_provider, &map_provider, &map_provider)
        .add_origin(Site { x: 0.0, y: 0.0 }, 0.0, Group::from_num(0), None)
        .unwrap()
        .iterate_as_possible(&mut rnd)
        .build();

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

fn write_to_image(
    bound_min: Site,
    bound_max: Site,
    img_width: u32,
    img_height: u32,
    terrain: &Terrain2D,
    network: &PathNetwork<TransportNode>,
    population_densities: &Vec<f64>,
    filename: &str,
) {
    let sites = terrain.sites();
    let interpolator = &Interpolator::new(
        &sites
            .iter()
            .map(|site| naturalneighbor::Point {
                x: site.x,
                y: site.y,
            })
            .collect::<Vec<_>>(),
    );

    let pixels: Vec<_> = (0..img_width)
        .into_par_iter()
        .flat_map(|imgx| {
            (0..img_height).into_par_iter().filter_map(move |imgy| {
                let x = bound_min.x
                    + (bound_max.x - bound_min.x) * ((imgx as f64 + 0.5) / img_width as f64);
                let y = bound_min.y
                    + (bound_max.y - bound_min.y) * ((imgy as f64 + 0.5) / img_height as f64);
                let site = Site { x, y };
                let elevation = terrain.get_elevation(&into_fastlem_site(site));
                let population_density = interpolator.interpolate(
                    &population_densities,
                    naturalneighbor::Point {
                        x: site.x,
                        y: site.y,
                    },
                );
                if let (Some(elevation), Ok(Some(population_density))) =
                    (elevation, population_density)
                {
                    let color = get_color(elevation, population_density);
                    Some((imgx, imgy, color))
                } else {
                    None
                }
            })
        })
        .collect();

    let mut pixmap = Pixmap::new(img_width, img_height).unwrap();
    let mut paint = Paint::default();

    for (imgx, imgy, color) in pixels {
        paint.set_color_rgba8(color[0], color[1], color[2], 255);
        pixmap.fill_rect(
            Rect::from_xywh(imgx as f32, imgy as f32, 1.0, 1.0).unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
    }

    network.nodes_iter().for_each(|(inode_id, inode)| {
        // draw node
        network.neighbors_iter(inode_id).map(|neighbors_iter| {
            neighbors_iter.for_each(|(_, jnode)| {
                let width = if inode.path_stage(jnode).as_num() == 0 {
                    2.0
                } else {
                    0.8
                };

                let color = if inode.path_is_bridge(jnode) {
                    [0, 230, 240]
                } else {
                    [0, 0, 0]
                };

                let stroke = Stroke {
                    width,
                    ..Default::default()
                };
                let site_a = inode.site;
                let site_b = jnode.site;
                let x_a = (site_a.x - bound_min.x) / (bound_max.x - bound_min.x) * img_width as f64;
                let y_a =
                    (site_a.y - bound_min.y) / (bound_max.y - bound_min.y) * img_height as f64;
                let x_b = (site_b.x - bound_min.x) / (bound_max.x - bound_min.x) * img_width as f64;
                let y_b =
                    (site_b.y - bound_min.y) / (bound_max.y - bound_min.y) * img_height as f64;
                let path = {
                    let mut path = PathBuilder::new();
                    path.move_to(x_a as f32, y_a as f32);
                    path.line_to(x_b as f32, y_b as f32);
                    path.finish().unwrap()
                };

                paint.set_color_rgba8(color[0], color[1], color[2], 100);
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            })
        });
    });

    pixmap.save_png(filename).unwrap();
}

// The below functions are for generating other data

fn create_terrain(
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

fn into_fastlem_site(site: Site) -> fastlem::models::surface::sites::Site2D {
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
