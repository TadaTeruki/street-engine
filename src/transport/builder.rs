use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::{NodeId, PathNetwork},
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::{
    node::{growth_type::GrowthType, node_stump::NodeStump, transport_node::TransportNode},
    params::{
        evaluation::PathEvaluationFactors, metrics::PathMetrics, numeric::Stage,
        rules::TransportRules, PathParams,
    },
    path_network_repository::PathNetworkRepository,
    traits::{PathEvaluator, RandomF64Provider, TerrainProvider, TransportRulesProvider},
};

pub struct TransportBuilder<'a, RP, TP, PE>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
    PE: PathEvaluator,
{
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
    path_evaluator: &'a PE,
    stump_heap: BinaryHeap<NodeStump>,
}

impl<'a, RP, TP, PE> TransportBuilder<'a, RP, TP, PE>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
    PE: PathEvaluator,
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
        let origin_node = TransportNode::new(origin_site, Stage::from_num(0), false);
        let origin_node_id = path_network.add_node(origin_node);

        let origin_metrics: PathMetrics = PathMetrics::default();

        self.push_new_stump(
            path_network,
            origin_node_id,
            Angle::new(angle_radian),
            origin_node.stage,
            origin_metrics.incremented(false, false),
        );
        self.push_new_stump(
            path_network,
            origin_node_id,
            Angle::new(angle_radian).opposite(),
            origin_node.stage,
            origin_metrics.incremented(false, false),
        );

        Some(self)
    }

    /// Add a path stump to the path network.
    fn push_new_stump(
        &mut self,
        path_network: &PathNetwork<TransportNode>,
        node_start_id: NodeId,
        angle_expected_end: Angle,
        stage: Stage,
        metrics: PathMetrics,
    ) -> Option<NodeStump> {
        let node_start = path_network.get_node(node_start_id)?;

        let rules_start =
            self.rules_provider
                .get_rules(&node_start.site, angle_expected_end, stage, &metrics)?;

        let (estimated_end_site, estimated_end_creates_bridge) =
            self.expect_end_of_path(node_start.site, angle_expected_end, stage, &rules_start)?;

        let evaluation = self.path_evaluator.evaluate(PathEvaluationFactors {
            site_start: node_start.site,
            site_end: estimated_end_site,
            angle: angle_expected_end,
            path_length: rules_start.path_normal_length,
            stage,
            creates_bridge: estimated_end_creates_bridge,
        })?;

        let stump = NodeStump::new(
            node_start_id,
            angle_expected_end,
            PathParams {
                stage,
                rules_start,
                metrics,
                evaluation,
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
        angle_expected: Angle,
        stage: Stage,
        rules_start: &TransportRules,
    ) -> Option<(Site, bool)> {
        let path_direction_rules = &rules_start.path_direction_rules;
        angle_expected
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
                    if let Some(evaluation) = self.path_evaluator.evaluate(PathEvaluationFactors {
                        site_start,
                        site_end,
                        angle,
                        path_length,
                        stage,
                        creates_bridge,
                    }) {
                        if let (Some(elevation_start), Some(elevation_end)) = (
                            self.terrain_provider.get_elevation(&site_start),
                            self.terrain_provider.get_elevation(&site_end),
                        ) {
                            if rules_start.check_elevation_diff_to_create_path_on_land(
                                elevation_start,
                                elevation_end,
                                path_length,
                            ) {
                                return Some((site_end, evaluation, creates_bridge));
                            }
                        }
                    }
                }
                None
            })
            .max_by(|(_, ev0, _), (_, ev1, _)| ev0.total_cmp(ev1))
            .map(|(site, _, creates_bridge)| (site, creates_bridge))
    }

    pub fn determine_growth_from_stump(
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
            stump.get_path_params().stage,
            &stump.get_path_params().rules_start,
        );

        let (site_expected_end, to_be_bridge_end) = site_expected_end_opt?;

        let related_nodes = path_network_repository
            .related_nodes_iter(
                LineSegment::new(stump_node.site, site_expected_end),
                stump
                    .get_path_params()
                    .rules_start
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

        let node_expected_end = TransportNode::new(
            site_expected_end,
            stump.get_path_params().stage,
            to_be_bridge_end,
        );

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

    pub fn apply_next_growth<R>(
        &mut self,
        rng: &mut R,
        path_network: &mut PathNetwork<TransportNode>,
        growth: GrowthType,
        stump_node_id: NodeId,
        path_params: &PathParams,
    ) where
        R: RandomF64Provider,
    {
        let start_site = if let Some(node) = path_network.get_node(stump_node_id) {
            node.site
        } else {
            return;
        };

        match growth {
            GrowthType::None => {}
            GrowthType::Existing(node_id) => {
                path_network.add_path(stump_node_id, node_id);
            }
            GrowthType::Intersect(node_next, encount_path) => {
                let next_node_id = path_network.add_node(node_next);
                path_network.remove_path(encount_path.0, encount_path.1);
                path_network.add_path(stump_node_id, next_node_id);
                path_network.add_path(next_node_id, encount_path.0);
                path_network.add_path(next_node_id, encount_path.1);
            }
            GrowthType::New(node_next) => {
                let node_id = path_network.add_node(node_next);
                path_network.add_path(stump_node_id, node_id);

                // if the new node is a bridge, the start node will be a bridge too.
                if node_next.creates_bridge {
                    path_network.modify_node(stump_node_id, |node| {
                        node.creates_bridge = true;
                    });
                }

                let straight_angle = start_site.get_angle(&node_next.site);
                self.push_new_stump(
                    path_network,
                    node_id,
                    straight_angle,
                    path_params.stage,
                    path_params.metrics.incremented(false, false),
                );
                let clockwise_branch =
                    rng.gen_f64() < path_params.rules_start.branch_rules.branch_density;
                if clockwise_branch {
                    let clockwise_staging =
                        rng.gen_f64() < path_params.rules_start.branch_rules.staging_probability;
                    let next_stage = if clockwise_staging {
                        path_params.stage.incremented()
                    } else {
                        path_params.stage
                    };
                    self.push_new_stump(
                        path_network,
                        node_id,
                        straight_angle.right_clockwise(),
                        next_stage,
                        path_params.metrics.incremented(clockwise_staging, true),
                    );
                }

                let counterclockwise_branch =
                    rng.gen_f64() < path_params.rules_start.branch_rules.branch_density;
                if counterclockwise_branch {
                    let counterclockwise_staging =
                        rng.gen_f64() < path_params.rules_start.branch_rules.staging_probability;
                    let next_stage = if counterclockwise_staging {
                        path_params.stage.incremented()
                    } else {
                        path_params.stage
                    };
                    self.push_new_stump(
                        path_network,
                        node_id,
                        straight_angle.right_counterclockwise(),
                        next_stage,
                        path_params
                            .metrics
                            .incremented(counterclockwise_staging, true),
                    );
                }
            }
        }
    }

    pub fn pop_stump(&mut self) -> Option<NodeStump> {
        self.stump_heap.pop()
    }
}
