use citymodelgen::Site2D;
use fastlem::core::{parameters::TopographicalParameters, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::builder::TerrainModel2DBulider;
use noise::{NoiseFn, Perlin};

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

#[test]
fn test_terrain() {
    let node_num = 50000;
    let bound_min = into_fastlem_site(Site2D { x: 0.0, y: 0.0 });
    let bound_max = into_fastlem_site(Site2D { x: 100.0, y: 100.0 });
    let seed = 10;

    let model =
        TerrainModel2DBulider::from_random_sites(node_num, bound_min.into(), bound_max.into())
            .relaxate_sites(1)
            .unwrap()
            .add_edge_sites(None, None)
            .unwrap()
            .build()
            .unwrap();

    let sites = model.sites().to_vec();

    let perlin = Perlin::new(seed);

    let terrain = TerrainGenerator::default()
        .set_model(model)
        .set_parameters(
            (0..sites.len())
                .map(|i| {
                    let site = sites[i];
                    let octaves = 8;
                    let x = site.x / (bound_max.x - bound_min.x);
                    let y = site.y / (bound_max.y - bound_min.y);
                    let dist_from_center = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
                    let noise_erodibility =
                        octaved_perlin(&perlin, x * 0.1, y * 0.1, octaves, 0.55)
                            .abs()
                            .powi(2)
                            * 3.0
                            + (1.0 - dist_from_center).powi(4) * 3.0;
                    let noise_is_outlet = (octaved_perlin(&perlin, x, y, octaves, 0.55) * 0.5
                        + 0.5)
                        * dist_from_center
                        + (1.0 - dist_from_center) * 0.5;
                    TopographicalParameters::default()
                        .set_erodibility(noise_erodibility)
                        .set_is_outlet(noise_is_outlet > 0.55)
                })
                .collect::<_>(),
        )
        .generate()
        .unwrap();

    let img_width = 500;
    let img_height = 500;

    let mut image_buf = image::RgbImage::new(img_width, img_height);

    for imgx in 0..img_width {
        for imgy in 0..img_height {
            let x = bound_max.x * (imgx as f64 / img_width as f64);
            let y = bound_max.y * (imgy as f64 / img_height as f64);
            let site = Site2D { x, y };
            let elevation = terrain.get_elevation(&into_fastlem_site(site));
            if let Some(elevation) = elevation {
                image_buf.put_pixel(imgx as u32, imgy as u32, image::Rgb(get_color(elevation)));
            }
        }
    }

    image_buf.save("image.png").unwrap();
}

fn get_color(elevation: f64) -> [u8; 3] {
    let colormap: [([u8; 3], f64); 6] = [
        ([70, 150, 200], 0.0),
        ([70, 150, 200], 0.1),
        ([240, 240, 210], 0.25),
        ([190, 200, 120], 0.5),
        ([25, 100, 25], 25.0),
        ([15, 60, 15], 40.0),
    ];
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

    if color_index == 0 {
        colormap[0].0
    } else if color_index == colormap.len() {
        colormap[colormap.len() - 1].0
    } else {
        let color_a = colormap[color_index - 1];
        let color_b = colormap[color_index];

        let prop = (elevation - color_a.1) / (color_b.1 - color_a.1);

        [
            (color_a.0[0] as f64 + (color_b.0[0] as f64 - color_a.0[0] as f64) * prop) as u8,
            (color_a.0[1] as f64 + (color_b.0[1] as f64 - color_a.0[1] as f64) * prop) as u8,
            (color_a.0[2] as f64 + (color_b.0[2] as f64 - color_a.0[2] as f64) * prop) as u8,
        ]
    }
}
