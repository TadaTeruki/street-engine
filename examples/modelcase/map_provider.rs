use fastlem::models::surface::terrain::Terrain2D;
use naturalneighbor::Interpolator;
use street_engine::{
    core::geometry::{angle::Angle, site::Site},
    transport::{
        params::{
            evaluation::PathEvaluationFactors,
            metrics::PathMetrics,
            numeric::Stage,
            rules::{BranchRules, BridgeRules, PathDirectionRules, TransportRules},
        },
        traits::{PathEvaluator, TerrainProvider, TransportRulesProvider},
    },
};

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
