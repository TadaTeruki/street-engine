use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::{NodeId, PathNetwork},
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::{
    node::{
        growable_node::NodeStump,
        growth_type::{BridgeNodeType, NextNodeType},
        transport_node::TransportNode,
    },
    params::{
        evaluation::PathEvaluationFactors,
        metrics::PathMetrics,
        numeric::Stage,
        rules::{check_elevation_diff, TransportRules},
        PathParams,
    },
    traits::{PathEvaluator, RandomF64Provider, TerrainProvider, TransportRulesProvider},
};

pub struct TransportBuilder<'a, RP, TP, PE>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
    PE: PathEvaluator,
{
    path_network: PathNetwork<TransportNode>,
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
    path_evaluator: &'a PE,
    open_nodes: BinaryHeap<NodeStump>,
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
            path_network: PathNetwork::new(),
            rules_provider,
            terrain_provider,
            path_evaluator,
            open_nodes: BinaryHeap::new(),
        }
    }

    /// Add a path stump to the path network.
    fn push_new_stump(
        &mut self,
        node_start: TransportNode,
        node_start_id: NodeId,
        angle_expected_end: Angle,
        stage: Stage,
        metrics: PathMetrics,
    ) -> Option<NodeStump> {
        let rules_start =
            self.rules_provider
                .get_rules(&node_start.site, angle_expected_end, stage, &metrics)?;

        let (estimated_end_site, estimated_end_is_bridge) =
            self.expect_end_of_path(node_start.site, angle_expected_end, stage, &rules_start)?;

        let evaluation = self.path_evaluator.evaluate(PathEvaluationFactors {
            site_start: node_start.site,
            site_end: estimated_end_site,
            angle: angle_expected_end,
            path_length: rules_start.path_normal_length,
            stage,
            is_bridge: estimated_end_is_bridge,
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

        self.open_nodes.push(stump.clone());

        Some(stump)
    }

    /// Add an origin node to the path network.
    ///
    /// The path which is extended from `origin_site` by `angle_radian` (and the opposite path) will be the first stumps.
    pub fn add_origin(
        mut self,
        origin_site: Site,
        angle_radian: f64,
        stage: Option<Stage>,
    ) -> Option<Self> {
        let stage = if let Some(stage) = stage {
            stage
        } else {
            Stage::from_num(0)
        };
        let origin_node = TransportNode::new(
            origin_site,
            self.terrain_provider.get_elevation(&origin_site)?,
            stage,
            false,
        );
        let origin_node_id = self.path_network.add_node(origin_node);
        let origin_metrics = PathMetrics::default();

        self.push_new_stump(
            origin_node,
            origin_node_id,
            Angle::new(angle_radian),
            stage,
            origin_metrics.incremented(false, false),
        );
        self.push_new_stump(
            origin_node,
            origin_node_id,
            Angle::new(angle_radian).opposite(),
            stage,
            origin_metrics.incremented(false, false),
        );

        Some(self)
    }

    /// Iterate the path network `n` times.
    pub fn iterate_n_times<R>(mut self, n: usize, rng: &mut R) -> Self
    where
        R: RandomF64Provider,
    {
        for _ in 0..n {
            self = self.iterate::<R>(rng);
        }
        self
    }

    /// Iterate network generation until there are no more stumps of new paths.
    pub fn iterate_as_possible<R>(mut self, rng: &mut R) -> Self
    where
        R: RandomF64Provider,
    {
        while !self.open_nodes.is_empty() {
            self = self.iterate::<R>(rng);
        }
        self
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
                    let is_bridge = i > 0;
                    if let Some(evaluation) = self.path_evaluator.evaluate(PathEvaluationFactors {
                        site_start,
                        site_end,
                        angle,
                        path_length,
                        stage,
                        is_bridge,
                    }) {
                        if let (Some(elevation_start), Some(elevation_end)) = (
                            self.terrain_provider.get_elevation(&site_start),
                            self.terrain_provider.get_elevation(&site_end),
                        ) {
                            if check_elevation_diff(
                                elevation_start,
                                elevation_end,
                                path_length,
                                rules_start.path_elevation_diff_limit,
                            ) {
                                return Some((site_end, evaluation, is_bridge));
                            }
                        }
                    }
                }
                None
            })
            .max_by(|(_, ev0, _), (_, ev1, _)| ev0.total_cmp(ev1))
            .map(|(site, _, is_bridge)| (site, is_bridge))
    }

    /// Iterate the path network to the next step.
    pub fn iterate<R>(mut self, rng: &mut R) -> Self
    where
        R: RandomF64Provider,
    {
        let prior_open_node = if let Some(stump) = self.open_nodes.pop() {
            stump
        } else {
            return self;
        };

        let prior_node =
            if let Some(node) = self.path_network.get_node(prior_open_node.get_node_id()) {
                node
            } else {
                return self;
            };

        // Set the end site of the path again.
        let site_expected_end_opt = self.expect_end_of_path(
            prior_node.site,
            prior_open_node.angle_expected(),
            prior_open_node.get_path_params().stage,
            &prior_open_node.get_path_params().rules_start,
        );

        let (site_expected_end, to_be_bridge_end) = if let Some(end) = site_expected_end_opt {
            end
        } else {
            return self;
        };

        let elevation_expected_end =
            if let Some(elevation) = self.terrain_provider.get_elevation(&site_expected_end) {
                elevation
            } else {
                return self;
            };

        // Find nodes around the line from the start site to the expected end site.
        let related_nodes = self
            .path_network
            .nodes_around_line_iter(
                LineSegment::new(prior_node.site, site_expected_end),
                prior_open_node
                    .get_path_params()
                    .rules_start
                    .path_extra_length_for_intersection,
            )
            .filter(|&node_id| *node_id != prior_open_node.get_node_id())
            .filter_map(|node_id| Some((self.path_network.get_node(*node_id)?, *node_id)))
            .collect::<Vec<_>>();

        // Find paths touching the rectangle around the line.
        let related_paths = self
            .path_network
            .paths_touching_rect_iter(prior_node.site, site_expected_end)
            .filter(|(node_id_start, node_id_end)| {
                *node_id_start != prior_open_node.get_node_id()
                    && *node_id_end != prior_open_node.get_node_id()
            })
            .filter_map(|(node_id_start, node_id_end)| {
                let node_start = self.path_network.get_node(*node_id_start)?;
                let node_end = self.path_network.get_node(*node_id_end)?;
                Some(((node_start, *node_id_start), (node_end, *node_id_end)))
            })
            .collect::<Vec<_>>();

        // Determine the next node.
        let growth = prior_open_node.determine_growth(
            prior_node,
            &TransportNode::new(
                site_expected_end,
                elevation_expected_end,
                prior_open_node.get_path_params().stage,
                to_be_bridge_end,
            ),
            &related_nodes,
            &related_paths,
        );

        self.apply_next_growth(
            rng,
            growth.next_node,
            growth.bridge_node,
            prior_open_node.get_node_id(),
            prior_open_node.get_path_params(),
        )
    }

    fn apply_next_growth<R>(
        mut self,
        rng: &mut R,
        next_node_type: NextNodeType,
        bridge_node_type: BridgeNodeType,
        base_node_id: NodeId,
        path_params: &PathParams,
    ) -> Self
    where
        R: RandomF64Provider,
    {
        if let BridgeNodeType::Middle(bridge_node) = bridge_node_type {
            let bridge_node_id = self.path_network.add_node(bridge_node);
            self.path_network.add_path(base_node_id, bridge_node_id);

            return self.apply_next_growth(
                rng,
                next_node_type,
                BridgeNodeType::None,
                bridge_node_id,
                path_params,
            );
        }

        let start_site = if let Some(node) = self.path_network.get_node(base_node_id) {
            node.site
        } else {
            return self;
        };

        match next_node_type {
            NextNodeType::None => {
                return self;
            }
            NextNodeType::Existing(node_id) => {
                self.path_network.add_path(base_node_id, node_id);
            }
            NextNodeType::Intersect(node_next, encount_path) => {
                let next_node_id = self.path_network.add_node(node_next);
                self.path_network
                    .remove_path(encount_path.0, encount_path.1);
                self.path_network.add_path(base_node_id, next_node_id);
                self.path_network.add_path(next_node_id, encount_path.0);
                self.path_network.add_path(next_node_id, encount_path.1);
            }
            NextNodeType::New(node_next) => {
                let node_id = self.path_network.add_node(node_next);
                self.path_network.add_path(base_node_id, node_id);

                let straight_angle = start_site.get_angle(&node_next.site);
                self.push_new_stump(
                    node_next,
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
                        node_next,
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
                        node_next,
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

        self
    }

    pub fn snapshot(self) -> (PathNetwork<TransportNode>, Self) {
        (self.path_network.clone().into_optimized(), self)
    }
}
