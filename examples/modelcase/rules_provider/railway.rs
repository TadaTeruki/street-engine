use street_engine::{
    core::geometry::{angle::Angle, site::Site},
    transport::{
        params::{
            metrics::PathMetrics,
            numeric::Stage,
            prioritization::PathPrioritizationFactors,
            rules::{BranchRules, BridgeRules, PathDirectionRules, TransportRules},
        },
        traits::{PathPrioritizator, TransportRulesProvider},
    },
};

use crate::map_provider::{into_fastlem_site, MapProvider};

pub struct RulesProviderForRailway<'a> {
    map_provider: &'a MapProvider<'a>,
}

impl RulesProviderForRailway<'_> {
    pub fn new<'a>(map_provider: &'a MapProvider<'a>) -> RulesProviderForRailway<'a> {
        RulesProviderForRailway { map_provider }
    }
}

impl<'a> TransportRulesProvider for RulesProviderForRailway<'a> {
    fn get_rules(
        &self,
        site: &Site,
        _: Angle,
        _: Stage,
        metrics: &PathMetrics,
    ) -> Option<TransportRules> {
        let population_density = self.map_provider.get_population_density(site)?;
        let path_normal_length = 0.7;

        let branch_motivation = if metrics.extend_count_since_last_branched % 7 == 0
            && metrics.extend_count_since_last_branched > 7
        {
            1.0
        } else {
            0.0
        };

        Some(TransportRules {
            path_normal_length,
            path_extra_length_for_intersection: path_normal_length * 0.7,
            path_elevation_diff_limit: Some(10.0),
            branch_rules: BranchRules {
                branch_density: (0.3 + population_density * 0.2) * branch_motivation,
                staging_probability: 0.0,
            },
            path_direction_rules: PathDirectionRules {
                max_radian: std::f64::consts::PI / (10.0 + 50.0 * population_density),
                comparison_step: 3,
            },
            bridge_rules: BridgeRules {
                max_bridge_length: 8.0,
                check_step: 8,
            },
        })
    }
}

impl<'a> PathPrioritizator for RulesProviderForRailway<'a> {
    fn evaluate(&self, factor: PathPrioritizationFactors) -> Option<f64> {
        let site = factor.site_end;
        let elevation = self
            .map_provider
            .get_terrain()
            .get_elevation(&into_fastlem_site(site))?;
        let population_density = self.map_provider.get_population_density(&site)?;

        Some((1e-9 + population_density) * (-elevation))
    }
}
