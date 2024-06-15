use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::{NodeId, PathNetwork},
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::{
    node::{growth_type::GrowthType, node_stump::NodeStump, transport_node::TransportNode},
    params::{
        numeric::Stage, prioritization::PathPrioritizationFactors,
        rules::GrowthRules, StumpParams,
    },
    path_network_repository::PathNetworkRepository,
    traits::{PathPrioritizator, RandomF64Provider, TerrainProvider, GrowthRulesProvider},
};

pub struct TransportBuilder<'a, RP, TP, PE>
where
    RP: GrowthRulesProvider,
    TP: TerrainProvider,
    PE: PathPrioritizator,
{
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
    path_evaluator: &'a PE,
    stump_heap: BinaryHeap<NodeStump>,
}

impl<'a, RP, TP, PE> TransportBuilder<'a, RP, TP, PE>
where
    RP: GrowthRulesProvider,
    TP: TerrainProvider,
    PE: PathPrioritizator,
{
    /// Create a new `TransportBuilder`.
    pub fn new(rules_provider: &'a RP, terrain_provider: &'a TP, path_evaluator: &'a PE) -> Self {
        Self {
            rules_provider,
            terrain_provider,
            path_evaluator,
            stump_heap: BinaryHeap::new(),
        }
    }

    /// Add an origin node to the path network.
    ///
    /// The path which is extended from `origin_site` by `angle_radian` (and the opposite path) will be the first stump_heap.
    pub fn add_origin(
        mut self,
        path_network: &mut PathNetwork<TransportNode>,
        origin_site: Site,
        angle_radian: f64,
    ) -> Option<Self> {
        let origin_node = TransportNode::new(origin_site, Stage::from_num(0));
        let origin_node_id = path_network.add_node(origin_node);

        self.push_new_stump(
            path_network,
            origin_node_id,
            Angle::new(angle_radian),
            origin_node.stage,
        );
        self.push_new_stump(
            path_network,
            origin_node_id,
            Angle::new(angle_radian).opposite(),
            origin_node.stage,
        );

        Some(self)
    }

    /// Add a path stump to the path network.
    fn push_new_stump(
        &mut self,
        path_network: &PathNetwork<TransportNode>,
        node_id: NodeId,
        angle_expected: Angle,
        stage: Stage,
    ) -> Option<NodeStump> {
        let node_start = path_network.get_node(node_id)?;

        let rules =
            self.rules_provider
                .get_rules(&node_start.site, stage)?;

        let (estimated_end_site, estimated_end_creates_bridge) =
            self.expect_end_of_path(node_start.site, angle_expected, stage, &rules)?;

        let priority = self.path_evaluator.evaluate(PathPrioritizationFactors {
            site_start: node_start.site,
            site_end: estimated_end_site,
            angle: angle_expected,
            path_length: rules.path_normal_length,
            stage,
            creates_bridge: estimated_end_creates_bridge,
        })?;

        let stump = NodeStump::new(
            node_id,
            angle_expected,
            StumpParams {
                stage,
                rules,
                priority,
            },
        );

        self.stump_heap.push(stump.clone());

        Some(stump)
    }

    /// Query the expected end of the path.
    /// Return the site and is the path to be a bridge.
    fn expect_end_of_path(
        &self,
        site_start: Site,
        angle_expecteded: Angle,
        stage: Stage,
        rules_start: &GrowthRules,
    ) -> Option<(Site, bool)> {
        let path_direction_rules = &rules_start.path_direction_rules;
        angle_expecteded
            .iter_range_around(
                path_direction_rules.max_radian,
                path_direction_rules.comparison_step,
            )
            .filter_map(|angle| {
                for i in 0..=rules_start.bridge_rules.check_step {
                    let bridge_path_length = if rules_start.bridge_rules.check_step == 0 {
                        0.0
                    } else {
                        rules_start.bridge_rules.max_bridge_length * (i as f64)
                            / (rules_start.bridge_rules.check_step as f64)
                    };
                    let path_length = rules_start.path_normal_length + bridge_path_length;
                    let site_end = site_start.extend(angle, path_length);
                    let creates_bridge = i > 0;
                    if let Some(priority) =
                        self.path_evaluator.evaluate(PathPrioritizationFactors {
                            site_start,
                            site_end,
                            angle,
                            path_length,
                            stage,
                            creates_bridge,
                        })
                    {
                        if let (Some(elevation_start), Some(elevation_end)) = (
                            self.terrain_provider.get_elevation(&site_start),
                            self.terrain_provider.get_elevation(&site_end),
                        ) {
                            if rules_start.check_elevation_diff_to_create_path_on_land(
                                elevation_start,
                                elevation_end,
                                path_length,
                            ) {
                                return Some((site_end, priority, creates_bridge));
                            }
                        }
                    }
                }
                None
            })
            .max_by(|(_, ev0, _), (_, ev1, _)| ev0.total_cmp(ev1))
            .map(|(site, _, creates_bridge)| (site, creates_bridge))
    }

    pub fn determine_growth_of_stump(
        &self,
        path_network_repository: &PathNetworkRepository,
        path_network: &PathNetwork<TransportNode>,
        stump: &NodeStump,
    ) -> Option<GrowthType> {
        let stump_node = path_network.get_node(stump.get_node_id())?;
        // Set the end site of the path again.
        let site_expected_end_opt = self.expect_end_of_path(
            stump_node.site,
            stump.angle_expected(),
            stump.get_stump_params().stage,
            &stump.get_stump_params().rules,
        );

        let (site_expected_end, to_be_bridge_end) = site_expected_end_opt?;

        let related_nodes = path_network_repository
            .related_nodes_iter(
                LineSegment::new(stump_node.site, site_expected_end),
                stump
                    .get_stump_params()
                    .rules
                    .path_extra_length_for_intersection,
            )
            .filter(|related_node| related_node.node_id != stump.get_node_id())
            .collect::<Vec<_>>();

        let related_paths = path_network_repository
            .related_paths_iter(LineSegment::new(stump_node.site, site_expected_end))
            .filter(|(node_start, node_end)| {
                node_start.node_id != stump.get_node_id() && node_end.node_id != stump.get_node_id()
            })
            .collect::<Vec<_>>();

        let node_expected_end =
            TransportNode::new(site_expected_end, stump.get_stump_params().stage);

        // Determine the growth of the path.
        let growth = stump.determine_growth(
            stump_node,
            &node_expected_end,
            &related_nodes,
            &related_paths,
            self.terrain_provider,
        );

        Some(growth)
    }

    /// Apply the next growth to the network.
    pub fn apply_next_growth<R>(
        &mut self,
        rng: &mut R,
        path_network: &mut PathNetwork<TransportNode>,
        growth: GrowthType,
        stump: NodeStump,
    ) where
        R: RandomF64Provider,
    {
        let start_site = if let Some(node) = path_network.get_node(stump.get_node_id()) {
            node.site
        } else {
            return;
        };

        match growth {
            GrowthType::None => {}
            GrowthType::Existing(node_id) => {
                path_network.add_path(stump.get_node_id(), node_id);
            }
            GrowthType::Intersect(node_next, encount_path) => {
                let next_node_id = path_network.add_node(node_next);
                path_network.remove_path(encount_path.0, encount_path.1);
                path_network.add_path(stump.get_node_id(), next_node_id);
                path_network.add_path(next_node_id, encount_path.0);
                path_network.add_path(next_node_id, encount_path.1);
            }
            GrowthType::New(node_next) => {
                let node_id = path_network.add_node(node_next);
                path_network.add_path(stump.get_node_id(), node_id);

                let straight_angle = start_site.get_angle(&node_next.site);
                self.push_new_stump(
                    path_network,
                    node_id,
                    straight_angle,
                    stump.get_stump_params().stage,
                );
                let clockwise_branch =
                    rng.gen_f64() < stump.get_stump_params().rules.branch_rules.branch_density;
                if clockwise_branch {
                    let clockwise_staging = rng.gen_f64()
                        < stump
                            .get_stump_params()
                            .rules
                            .branch_rules
                            .staging_probability;
                    let next_stage = if clockwise_staging {
                        stump.get_stump_params().stage.incremented()
                    } else {
                        stump.get_stump_params().stage
                    };
                    self.push_new_stump(
                        path_network,
                        node_id,
                        straight_angle.right_clockwise(),
                        next_stage,
                    );
                }

                let counterclockwise_branch =
                    rng.gen_f64() < stump.get_stump_params().rules.branch_rules.branch_density;
                if counterclockwise_branch {
                    let counterclockwise_staging = rng.gen_f64()
                        < stump
                            .get_stump_params()
                            .rules
                            .branch_rules
                            .staging_probability;
                    let next_stage = if counterclockwise_staging {
                        stump.get_stump_params().stage.incremented()
                    } else {
                        stump.get_stump_params().stage
                    };
                    self.push_new_stump(
                        path_network,
                        node_id,
                        straight_angle.right_counterclockwise(),
                        next_stage,
                    );
                }
            }
        }
    }

    pub fn pop_stump(&mut self) -> Option<NodeStump> {
        self.stump_heap.pop()
    }
}
