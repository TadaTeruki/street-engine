use crate::{bands::Bands, flatness::create_flatness_map, places::collection::PlaceMapCollection};
use drainage_basin_builder::map::DrainageMap;
use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use vislayers::{colormap::SimpleColorMap, geometry::FocusRange, window::Layer};
use worley_particle::map::ParticleMap;

pub struct FactorsMap {
    elevation_map: ParticleMap<f64>,
    drainage_map: DrainageMap,
    flatness_map: ParticleMap<f64>,
    place_map_collection: PlaceMapCollection,

    elevation_bands: Bands,
}

impl FactorsMap {
    pub fn new(
        elevation_map: ParticleMap<f64>,
        drainage_map: DrainageMap,
        flatness_map: Option<ParticleMap<f64>>,
        sea_level: f64,
    ) -> Self {
        let flatness_map =
            flatness_map.unwrap_or_else(|| create_flatness_map(&elevation_map, 2, sea_level));

        let elevation_bands = Bands::new(&elevation_map, 80, 300000.0, sea_level, 1.0);

        let region_scales = vec![2.0, 4.0, 8.0, 16.0, 32.0, 64.0]
            .iter()
            .map(|scale| elevation_map.params().scale * scale)
            .collect::<Vec<_>>();

        let place_map_collection = PlaceMapCollection::new(
            &elevation_map,
            &flatness_map,
            &region_scales,
            elevation_map.params().scale * 0.5,
        );

        Self {
            elevation_map,
            drainage_map,
            flatness_map,
            place_map_collection,
            elevation_bands,
        }
    }

    pub fn flatness_map(&self) -> &ParticleMap<f64> {
        &self.flatness_map
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

        self.place_map_collection
            .draw(drawing_area, cr, focus_range);
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
