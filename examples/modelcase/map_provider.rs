use fastlem::models::surface::terrain::Terrain2D;
use naturalneighbor::Interpolator;
use street_engine::{core::geometry::site::Site, transport::traits::TerrainProvider};

pub fn into_fastlem_site(site: Site) -> fastlem::models::surface::sites::Site2D {
    fastlem::models::surface::sites::Site2D {
        x: site.x,
        y: site.y,
    }
}

pub struct MapProvider<'a> {
    terrain: &'a Terrain2D,
    population_densities: &'a Vec<f64>,
    interpolator: Interpolator,
}

impl<'a> MapProvider<'a> {
    pub fn new(
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

    pub fn get_terrain(&self) -> &Terrain2D {
        self.terrain
    }

    pub fn get_population_density(&self, site: &Site) -> Option<f64> {
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
