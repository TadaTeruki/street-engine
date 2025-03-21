use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use vislayers::{colormap::SimpleColorMap, geometry::FocusRange};
use worley_particle::map::{
    lerp::{vertorization::Band, InterpolationMethod},
    IDWStrategy, ParticleMap,
};

pub struct Bands {
    bands: Vec<Band>,
}

fn bands_step(focus_range: &FocusRange) -> usize {
    (2.0_f64.powi((focus_range.radius() * 8.0).ceil() as i32) as usize).min(16) - 1
}

impl Bands {
    pub fn new(
        particle_map: &ParticleMap<f64>,
        num_thresholds: usize,
        rasterise_scale: f64,
        min_value: f64,
        max_value: f64,
    ) -> Self {
        let thresholds = (0..num_thresholds)
            .map(|i| i as f64 * max_value / (num_thresholds - 1) as f64 + min_value)
            .collect::<Vec<_>>();
        let bands = particle_map
            .isobands(
                particle_map.corners(),
                rasterise_scale,
                &thresholds,
                InterpolationMethod::IDW(IDWStrategy::default_from_params(particle_map.params())),
                true,
            )
            .expect("Error generating bands");
        Self { bands }
    }

    pub fn draw(
        &self,
        particle_map: &ParticleMap<f64>,
        color_map: &SimpleColorMap,
        drawing_area: &DrawingArea,
        cr: &Context,
        focus_range: &FocusRange,
        alpha: f64,
    ) {
        let area_width = drawing_area.width();
        let area_height = drawing_area.height();

        let rect = focus_range.to_rect(area_width as f64, area_height as f64);

        let bands_step = bands_step(focus_range);

        for threshold in self.bands.iter().step_by(bands_step) {
            cr.new_path();
            for polygon in &threshold.polygons {
                for (i, point) in polygon.iter().enumerate().step_by(bands_step) {
                    let x = rect.map_coord_x(
                        point.0 - particle_map.params().scale / 2.0,
                        0.0,
                        area_width as f64,
                    );
                    let y = rect.map_coord_y(
                        point.1 - particle_map.params().scale / 2.0,
                        0.0,
                        area_height as f64,
                    );

                    if i == 0 {
                        cr.move_to(x, y);
                    } else {
                        cr.line_to(x, y);
                    }
                }

                cr.close_path();
            }
            let color = color_map.get_color(threshold.threshold);
            cr.set_source_rgba(color[0] / 255.0, color[1] / 255.0, color[2] / 255.0, alpha);
            cr.fill().expect("Failed to fill polygon");
        }

        cr.set_source_rgba(1.0, 0.0, 0.0, alpha);
        cr.arc(
            rect.map_coord_x(0.0, 0.0, area_width as f64),
            rect.map_coord_y(0.0, 0.0, area_height as f64),
            2.0,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        cr.fill().expect("Failed to draw center point");
    }
}
