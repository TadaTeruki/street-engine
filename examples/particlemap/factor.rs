use crate::bands::Bands;
use gtk4::{cairo::Context, DrawingArea};
use vislayers::{colormap::SimpleColorMap, geometry::FocusRange, window::Layer};
use worley_particle::map::{
    grad::GradStrategy,
    lerp::{IDWStrategy, InterpolationMethod},
    ParticleMap,
};

pub struct FactorsMap {
    elevation_map: ParticleMap<f64>,
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
    pub fn from_elevation_map(elevation_map: ParticleMap<f64>, sea_level: f64) -> Self {
        let habitability_map = elevation_map
            .iter()
            .filter_map(|(particle, elevation)| {
                if *elevation < sea_level {
                    return None;
                }
                let (x, y) = particle.site();
                let gradient = elevation_map.get_forward_gradient(
                    x,
                    y,
                    &GradStrategy {
                        delta: elevation_map.params().scale * 5.0,
                        iteration: 4,
                        sample_num: 8,
                    },
                    &InterpolationMethod::IDW(IDWStrategy::default_from_params(
                        elevation_map.params(),
                    )),
                )?;
                let havitability = gradient_to_habitability(gradient.value)?;
                Some((*particle, havitability))
            })
            .collect::<ParticleMap<f64>>();

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
            habitability_map,
            elevation_bands,
            habitability_bands,
        }
    }
}

impl Layer for FactorsMap {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        // 1. Create a color map for elevation visualization
        let elevation_color_map = SimpleColorMap::new(
            vec![
                [100.0, 150.0, 70.0],  // Green for low elevations
                [60.0, 90.0, 55.0],    // Darker green for mid elevations
                [210.0, 210.0, 210.0], // Gray for high elevations
            ],
            vec![0.0, 0.35, 0.6],
        );

        // 2. Draw the elevation bands using the Bands.draw method
        self.elevation_bands.draw(
            &self.elevation_map,
            &elevation_color_map,
            drawing_area,
            cr,
            focus_range,
            1.0,
        );

        // // 3. Create a gradient color map for population elevation gradients
        let habitability_color_map = SimpleColorMap::new(
            vec![[255.0, 50.0, 50.0], [255.0, 150.0, 50.0]],
            vec![0.0, 1.0],
        );

        // 4. Draw the population gradient bands
        // This will overlay on top of the elevation bands
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
