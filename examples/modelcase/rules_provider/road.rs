use street_engine::{
    core::geometry::site::Site,
    transport::{
        params::{
            numeric::Stage,
            prioritization::PathPrioritizationFactors,
            rules::{BranchRules, BridgeRules, GrowthRules, PathDirectionRules},
        },
        traits::{GrowthRulesProvider, PathPrioritizator},
    },
};

use crate::map_provider::{into_fastlem_site, MapProvider};

pub struct RulesProviderForRoad<'a> {
    map_provider: &'a MapProvider<'a>,
}

impl RulesProviderForRoad<'_> {
    pub fn new<'a>(map_provider: &'a MapProvider<'a>) -> RulesProviderForRoad<'a> {
        RulesProviderForRoad { map_provider }
    }
}

impl<'a> GrowthRulesProvider for RulesProviderForRoad<'a> {
    fn get_rules(&self, site: &Site, stage: Stage) -> Option<GrowthRules> {
        let population_density = self.map_provider.get_population_density(site)?;
        let is_street = stage.as_num() > 0;
        let path_normal_length = 0.4;

        if is_street {
            // street
            Some(GrowthRules {
                path_normal_length,
                path_extra_length_for_intersection: path_normal_length * 0.7,
                path_elevation_diff_limit: None,
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
            Some(GrowthRules {
                path_normal_length,
                path_extra_length_for_intersection: path_normal_length * 0.7,
                path_elevation_diff_limit: Some(5.0),
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

impl<'a> PathPrioritizator for RulesProviderForRoad<'a> {
    fn evaluate(&self, factor: PathPrioritizationFactors) -> Option<f64> {
        let site = factor.site_end;
        let elevation = self
            .map_provider
            .get_terrain()
            .get_elevation(&into_fastlem_site(site))?;
        let population_density = self.map_provider.get_population_density(&site)?;

        let path_priority = (1e-9 + population_density) * (-elevation);

        let stage = factor.stage;
        if stage.as_num() > 0 {
            return Some(path_priority);
        } else {
            return Some(path_priority + 1e5);
        }
    }
}
