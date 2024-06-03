use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::{NodeId, PathNetwork},
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
    Stage,
};

use super::{
    evaluation::PathEvaluationFactors,
    node::{
        candidate::{BridgeNode, NextTransportNode, PathCandidate},
        transport_node::TransportNode,
    },
    rules::{check_slope, TransportRules},
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
    path_candidate_container: BinaryHeap<PathCandidate>,
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
            path_candidate_container: BinaryHeap::new(),
        }
    }

    fn push_new_candidate(
        &mut self,
        node_start: TransportNode,
        node_start_id: NodeId,
        angle_expected_end: Angle,
        stage: Stage,
    ) -> Option<PathCandidate> {
        let rules_start =
            self.rules_provider
                .get_rules(&node_start.site, angle_expected_end, stage)?;

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

        let candidate = PathCandidate::new(
            node_start,
            node_start_id,
            angle_expected_end,
            stage,
            rules_start,
            evaluation,
        );

        self.path_candidate_container.push(candidate.clone());

        Some(candidate)
    }

    /// Add an origin node to the path network.
    ///
    /// The path which is extended from `origin_site` by `angle_radian` (and the opposite path) will be the first candidates.
    pub fn add_origin(
        mut self,
        origin_site: Site,
        angle_radian: f64,
        stage: Option<Stage>,
    ) -> Option<Self> {
        let stage = if let Some(stage) = stage {
            stage
        } else {
            Stage::new(0)
        };
        let origin_node = TransportNode::new(
            origin_site,
            stage,
            self.terrain_provider.get_elevation(&origin_site)?,
            false,
        );
        let origin_node_id = self.path_network.add_node(origin_node);

        self.push_new_candidate(origin_node, origin_node_id, Angle::new(angle_radian), stage);
        self.push_new_candidate(
            origin_node,
            origin_node_id,
            Angle::new(angle_radian).opposite(),
            stage,
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

    /// Iterate network generation until there are no more candidates of new paths.
    pub fn iterate_as_possible<R>(mut self, rng: &mut R) -> Self
    where
        R: RandomF64Provider,
    {
        while !self.path_candidate_container.is_empty() {
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
                            if check_slope(
                                elevation_start,
                                elevation_end,
                                path_length,
                                rules_start.path_max_elevation_diff,
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
        let prior_candidate = if let Some(candidate) = self.path_candidate_container.pop() {
            candidate
        } else {
            return self;
        };

        let rules_start = prior_candidate.get_rules_start();

        let site_start = prior_candidate.get_site_start();
        // Set the end site of the path again.
        let site_expected_end_opt = self.expect_end_of_path(
            site_start,
            prior_candidate.angle_expected_end(),
            prior_candidate.get_stage(),
            rules_start,
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

        let related_nodes = self
            .path_network
            .nodes_around_line_iter(
                LineSegment::new(site_start, site_expected_end),
                prior_candidate
                    .get_rules_start()
                    .path_extra_length_for_intersection,
            )
            .filter(|&node_id| *node_id != prior_candidate.get_node_start_id())
            .filter_map(|node_id| Some((self.path_network.get_node(*node_id)?, *node_id)))
            .collect::<Vec<_>>();

        let related_paths = self
            .path_network
            .paths_touching_rect_iter(site_start, site_expected_end)
            .filter(|(node_id_start, node_id_end)| {
                *node_id_start != prior_candidate.get_node_start_id()
                    && *node_id_end != prior_candidate.get_node_start_id()
            })
            .filter_map(|(node_id_start, node_id_end)| {
                let node_start = self.path_network.get_node(*node_id_start)?;
                let node_end = self.path_network.get_node(*node_id_end)?;
                Some(((node_start, *node_id_start), (node_end, *node_id_end)))
            })
            .collect::<Vec<_>>();

        let candidate_node_id = prior_candidate.get_node_start_id();
        let (next_node_type, bridge_node) = prior_candidate.determine_next_node(
            site_expected_end,
            elevation_expected_end,
            prior_candidate.get_stage(),
            to_be_bridge_end,
            &related_nodes,
            &related_paths,
        );

        if let BridgeNode::Middle(bridge_node) = bridge_node {
            let bridge_node_id = self.path_network.add_node(bridge_node);
            self.path_network
                .add_path(candidate_node_id, bridge_node_id);

            self.add_path(
                rng,
                next_node_type,
                bridge_node.site,
                bridge_node_id,
                prior_candidate.get_stage(),
                prior_candidate.get_rules_start(),
            )
        } else {
            self.add_path(
                rng,
                next_node_type,
                site_start,
                candidate_node_id,
                prior_candidate.get_stage(),
                prior_candidate.get_rules_start(),
            )
        }
    }

    fn add_path<R>(
        mut self,
        rng: &mut R,
        next_node_type: NextTransportNode,
        site_start: Site,
        start_node_id: NodeId,
        stage: Stage,
        rules_start: &TransportRules,
    ) -> Self
    where
        R: RandomF64Provider,
    {
        match next_node_type {
            NextTransportNode::IntersectBridge => {
                return self;
            }
            NextTransportNode::Existing(node_id) => {
                self.path_network.add_path(start_node_id, node_id);
            }
            NextTransportNode::Intersect(node_next, encount_path) => {
                let next_node_id = self.path_network.add_node(node_next);
                self.path_network
                    .remove_path(encount_path.0, encount_path.1);
                self.path_network.add_path(start_node_id, next_node_id);
                self.path_network.add_path(next_node_id, encount_path.0);
                self.path_network.add_path(next_node_id, encount_path.1);
            }
            NextTransportNode::New(node_next) => {
                let node_id = self.path_network.add_node(node_next);
                self.path_network.add_path(start_node_id, node_id);

                let straight_angle = site_start.get_angle(&node_next.site);
                self.push_new_candidate(node_next, node_id, straight_angle, stage);
                let clockwise_branch = rng.gen_f64() < rules_start.branch_rules.branch_density;
                if clockwise_branch {
                    let clockwise_staging =
                        rng.gen_f64() < rules_start.branch_rules.staging_probability;
                    let next_stage = if clockwise_staging {
                        stage.incremented()
                    } else {
                        stage
                    };
                    self.push_new_candidate(
                        node_next,
                        node_id,
                        straight_angle.right_clockwise(),
                        next_stage,
                    );
                }

                let counterclockwise_branch =
                    rng.gen_f64() < rules_start.branch_rules.branch_density;
                if counterclockwise_branch {
                    let counterclockwise_staging =
                        rng.gen_f64() < rules_start.branch_rules.staging_probability;
                    let next_stage = if counterclockwise_staging {
                        stage.incremented()
                    } else {
                        stage
                    };
                    self.push_new_candidate(
                        node_next,
                        node_id,
                        straight_angle.right_counterclockwise(),
                        next_stage,
                    );
                }
            }
        }

        self
    }

    pub fn build(self) -> PathNetwork<TransportNode> {
        self.path_network.into_optimized()
    }
}
