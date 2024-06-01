use std::{collections::BinaryHeap, f32::consts::E};

use crate::core::{
    container::path_network::{NodeId, PathNetwork},
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
    Stage,
};

use super::{
    node::{
        candidate::{BridgeNode, NextTransportNode, PathCandidate},
        node::TransportNode,
    },
    rules::{check_slope, TransportRules},
    traits::{RandomF64Provider, TerrainProvider, TransportRulesProvider},
};

pub struct TransportBuilder<'a, RP, TP>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
{
    path_network: PathNetwork<TransportNode>,
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
    path_candidate_container: BinaryHeap<PathCandidate>,
}

impl<'a, RP, TP> TransportBuilder<'a, RP, TP>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
{
    /// Create a new `TransportBuilder`.
    pub fn new(rules_provider: &'a RP, terrain_provider: &'a TP) -> Self {
        Self {
            path_network: PathNetwork::new(),
            rules_provider,
            terrain_provider,
            path_candidate_container: BinaryHeap::new(),
        }
    }

    fn create_new_candidate(
        &mut self,
        node_start: TransportNode,
        node_start_id: NodeId,
        angle_expected_end: Angle,
        stage: Stage,
    ) -> bool {
        let rules_start = if let Some(rules) =
            self.rules_provider
                .get_rules(&node_start.site, angle_expected_end, stage)
        {
            rules
        } else {
            return false;
        };
        self.path_candidate_container.push(PathCandidate::new(
            node_start,
            node_start_id,
            angle_expected_end,
            stage,
            rules_start,
        ));

        true
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

        self.create_new_candidate(origin_node, origin_node_id, Angle::new(angle_radian), stage);
        self.create_new_candidate(
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
    fn query_expected_end_of_path(
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
                    if let Some(rules_end) = self.rules_provider.get_rules(&site_end, angle, stage)
                    {
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
                                return Some((site_end, rules_end, i > 0));
                            }
                        }
                    }
                }
                return None;
            })
            .max_by(|(_, rules1, _), (_, rules2, _)| {
                rules1.path_priority.total_cmp(&rules2.path_priority)
            })
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
        let site_expected_end_opt = self.query_expected_end_of_path(
            site_start,
            prior_candidate.angle_expected_end(),
            prior_candidate.get_stage(),
            &rules_start,
        );

        let (site_expected_end, to_be_bridge_end) = if let Some(end) = site_expected_end_opt {
            end
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

        let elevation_expected_end =
            if let Some(elevation) = self.terrain_provider.get_elevation(&site_expected_end) {
                elevation
            } else {
                return self;
            };

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

                let extend_to_straight =
                    self.create_new_candidate(node_next, node_id, straight_angle, stage);

                let clockwise_branch = rng.gen_f64() < rules_start.branch_rules.branch_density;
                if clockwise_branch || !extend_to_straight {
                    let clockwise_staging =
                        rng.gen_f64() < rules_start.branch_rules.staging_probability;
                    let next_stage = if clockwise_staging {
                        stage.incremented()
                    } else {
                        stage
                    };

                    self.create_new_candidate(
                        node_next,
                        node_id,
                        straight_angle.right_clockwise(),
                        next_stage,
                    );
                }

                let counterclockwise_branch =
                    rng.gen_f64() < rules_start.branch_rules.branch_density;
                if counterclockwise_branch || !extend_to_straight {
                    let counterclockwise_staging =
                        rng.gen_f64() < rules_start.branch_rules.staging_probability;
                    let next_stage = if counterclockwise_staging {
                        stage.incremented()
                    } else {
                        stage
                    };

                    self.create_new_candidate(
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
