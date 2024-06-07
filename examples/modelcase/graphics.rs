use fastlem::models::surface::terrain::Terrain2D;
use naturalneighbor::Interpolator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use street_engine::{
    core::{container::path_network::PathNetwork, geometry::site::Site},
    transport::node::transport_node::TransportNode,
};
use tiny_skia::{Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::map_provider::into_fastlem_site;

pub fn write_to_image(
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
