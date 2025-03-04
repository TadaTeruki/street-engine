use crate::bands::Bands;
use drainage_basin_builder::map::DrainageMap;
use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use vislayers::{colormap::SimpleColorMap, geometry::FocusRange, window::Layer};
use worley_particle::map::{
    grad::{GradDifferenceType, GradDirectionType, GradStrategy},
    lerp::{IDWStrategy, InterpolationMethod},
    ParticleMap,
};

pub struct FactorsMap {
    elevation_map: ParticleMap<f64>,
    drainage_map: DrainageMap,
    habitability_map: ParticleMap<f64>,

    elevation_bands: Bands,
    habitability_bands: Bands,
}

fn gradient_to_habitability(gradient: f64) -> Option<f64> {
    let habitability = 1.0 - gradient.abs() / 3.0;
    if habitability < 0.0 {
        return None;
    }
    Some(habitability.sqrt())
}

impl FactorsMap {
    fn create_habitability_map(
        elevation_map: &ParticleMap<f64>,
        sea_level: f64,
    ) -> ParticleMap<f64> {
        elevation_map
            .iter()
            .filter_map(|(particle, elevation)| {
                if *elevation < sea_level {
                    return None;
                }
                let (x, y) = particle.site();
                let gradient = elevation_map.get_gradient(
                    x,
                    y,
                    &GradStrategy {
                        delta: elevation_map.params().scale,
                        direction_type: GradDirectionType::Steepest,
                        difference_type: GradDifferenceType::Central,
                        ..Default::default()
                    },
                    &InterpolationMethod::IDW(IDWStrategy::default_from_params(
                        elevation_map.params(),
                    )),
                )?;
                let havitability = gradient_to_habitability(gradient.value)?;
                Some((*particle, havitability))
            })
            .collect::<ParticleMap<f64>>()
    }

    pub fn new(
        elevation_map: ParticleMap<f64>,
        drainage_map: DrainageMap,
        habitability_map: Option<ParticleMap<f64>>,
        sea_level: f64,
    ) -> Self {
        let habitability_map = habitability_map
            .unwrap_or_else(|| Self::create_habitability_map(&elevation_map, sea_level));

        let elevation_bands = Bands::new(&elevation_map, 80, 300000.0, sea_level, 1.0);

        // Create bands for the population gradient map
        let habitability_bands = Bands::new(
            &habitability_map,
            5,
            300000.0,
            0.0, // Min value as specified
            1.0, // Max value as specified
        );

        Self {
            elevation_map,
            drainage_map,
            habitability_map,
            elevation_bands,
            habitability_bands,
        }
    }

    // pub fn elevation_map(&self) -> &ParticleMap<f64> {
    //     &self.elevation_map
    // }

    // pub fn drainage_map(&self) -> &DrainageMap {
    //     &self.drainage_map
    // }

    pub fn habitability_map(&self) -> &ParticleMap<f64> {
        &self.habitability_map
    }
}

impl Layer for FactorsMap {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let elevation_color_map = SimpleColorMap::new(
            vec![
                [100.0, 150.0, 70.0],  // Green for low elevations
                [60.0, 90.0, 55.0],    // Darker green for mid elevations
                [210.0, 210.0, 210.0], // Gray for high elevations
            ],
            vec![0.0, 0.35, 0.6],
        );

        self.elevation_bands.draw(
            &self.elevation_map,
            &elevation_color_map,
            drawing_area,
            cr,
            focus_range,
            1.0,
        );

        DrainageMapLayer(&self.drainage_map).draw(drawing_area, cr, focus_range);

        let habitability_color_map = SimpleColorMap::new(
            vec![[255.0, 50.0, 50.0], [255.0, 150.0, 50.0]],
            vec![0.0, 1.0],
        );

        self.habitability_bands.draw(
            &self.habitability_map,
            &habitability_color_map,
            drawing_area,
            cr,
            focus_range,
            0.3,
        );
    }
}

#[derive(Clone)]
struct DrainageMapLayer<'a>(&'a DrainageMap);

impl<'a> Layer for DrainageMapLayer<'a> {
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
