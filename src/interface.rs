use crate::geom::Site2D;

pub trait ElevationModel {
    fn get_elevation(&self, site: Site2D) -> Option<f64>;
}

pub trait PopulationDensityModel {
    fn get_population_density(&self, site: Site2D) -> Option<f64>;
}
