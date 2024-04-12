use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::PathNetwork,
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
    Stage,
};

use super::{
    node::{NextTransportNode, PathCandidate, TransportNode},
    rules::PathDirectionRules,
    traits::{RandomF64Provider, TransportRulesProvider},
};

pub struct TransportBuilder<'a, TP>
where
    TP: TransportRulesProvider,
{
    path_network: PathNetwork<TransportNode>,
    rules_provider: &'a TP,
    path_candidate_container: BinaryHeap<PathCandidate>,
}

impl<'a, TP> TransportBuilder<'a, TP>
where
    TP: TransportRulesProvider,
{
    /// Create a new `TransportBuilder`.
    pub fn new(rules_provider: &'a TP) -> Self {
        Self {
            path_network: PathNetwork::new(),
            rules_provider,
            path_candidate_container: BinaryHeap::new(),
        }
    }

    /// Add an origin node to the path network.
    ///
    /// The path which is extended from `origin_site` by `angle_radian` (and the opposite path) will be the first candidates.
    pub fn add_origin(
        mut self,
        origin_site: Site,
        angle_radian: f64,
        stage_num: Option<usize>,
    ) -> Option<Self> {
        let node = TransportNode::new(origin_site);
        let node_id = self.path_network.add_node(node);
        let stage = if let Some(stage_num) = stage_num {
            Stage::new(stage_num)
        } else {
            Stage::default()
        };
        let rules = self.rules_provider.get_rules(&node.into(), stage.get())?;
        self.path_candidate_container.push(PathCandidate::new(
            node,
            node_id,
            Angle::new(angle_radian),
            stage,
            rules.clone(),
        ));

        self.path_candidate_container.push(PathCandidate::new(
            node,
            node_id,
            Angle::new(angle_radian).opposite(),
            stage,
            rules.clone(),
        ));

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
    fn query_expected_end_of_path(
        &self,
        site_start: Site,
        angle_expected: Angle,
        stage_num: usize,
        path_length: f64,
        path_direction_rules: &PathDirectionRules,
    ) -> Option<Site> {
        angle_expected
            .iter_range_around(
                path_direction_rules.max_radian,
                path_direction_rules.comparison_step,
            )
            .map(|angle| site_start.extend(angle, path_length))
            .filter_map(|site| Some((site, self.rules_provider.get_rules(&site, stage_num)?)))
            .max_by(|(_, rules1), (_, rules2)| {
                rules1.path_priority.total_cmp(&rules2.path_priority)
            })
            .map(|(site, _)| site)
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

        let rules = prior_candidate.get_rules();

        let site_start = prior_candidate.get_site_start();
        let site_expected_end_opt = self.query_expected_end_of_path(
            site_start,
            prior_candidate.angle_expected_end(),
            prior_candidate.get_stage().get(),
            rules.path_normal_length,
            &rules.path_direction_rules,
        );

        let site_expected_end = if let Some(site_expected_end) = site_expected_end_opt {
            site_expected_end
        } else {
            return self;
        };

        let related_nodes = self
            .path_network
            .nodes_around_line_iter(
                LineSegment::new(site_start, site_expected_end),
                prior_candidate
                    .get_rules()
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
        let next_node =
            prior_candidate.determine_next_node(site_expected_end, &related_nodes, &related_paths);

        match next_node {
            NextTransportNode::New(node_end) => {
                let node_id = self.path_network.add_node(node_end);
                self.path_network.add_path(candidate_node_id, node_id);

                // If the node is newly created, extend the path.
                if let Some(rules) = self
                    .rules_provider
                    .get_rules(&node_end.into(), prior_candidate.get_stage().get())
                {
                    let straight_angle = site_start.get_angle(&site_expected_end);
                    let straight_stage = prior_candidate.get_stage();
                    // pre-check if the path can be extended to the straight direction.
                    // If it can, the straight path candidate is added. The branch paths are added probabilistically.
                    // If it can't, the branch paths are added instead.
                    let extend_to_straight = self
                        .query_expected_end_of_path(
                            node_end.into(),
                            straight_angle,
                            straight_stage.get(),
                            rules.path_normal_length,
                            &rules.path_direction_rules,
                        )
                        .is_some();

                    if extend_to_straight {
                        self.path_candidate_container.push(PathCandidate::new(
                            node_end,
                            node_id,
                            straight_angle,
                            straight_stage,
                            rules.clone(),
                        ));
                    }

                    let clockwise_branch =
                        rng.gen_f64() < prior_candidate.get_rules().branch_rules.branch_density;
                    if clockwise_branch || !extend_to_straight {
                        let clockwise_staging = rng.gen_f64()
                            < prior_candidate.get_rules().branch_rules.staging_probability;
                        let next_stage = if clockwise_staging {
                            prior_candidate.get_stage().incremented()
                        } else {
                            prior_candidate.get_stage()
                        };

                        self.path_candidate_container.push(PathCandidate::new(
                            node_end,
                            node_id,
                            straight_angle.right_clockwise(),
                            next_stage,
                            rules.clone(),
                        ));
                    }

                    let counterclockwise_branch =
                        rng.gen_f64() < prior_candidate.get_rules().branch_rules.branch_density;
                    if counterclockwise_branch || !extend_to_straight {
                        let counterclockwise_staging = rng.gen_f64()
                            < prior_candidate.get_rules().branch_rules.staging_probability;
                        let next_stage = if counterclockwise_staging {
                            prior_candidate.get_stage().incremented()
                        } else {
                            prior_candidate.get_stage()
                        };

                        self.path_candidate_container.push(PathCandidate::new(
                            node_end,
                            node_id,
                            straight_angle.right_counterclockwise(),
                            next_stage,
                            rules.clone(),
                        ));
                    }
                }
            }
            NextTransportNode::Existing(node_id) => {
                self.path_network.add_path(candidate_node_id, node_id);
            }
            NextTransportNode::Intersect(node_end, encount_path) => {
                let next_node_id = self.path_network.add_node(node_end);
                self.path_network
                    .remove_path(encount_path.0, encount_path.1);
                self.path_network.add_path(candidate_node_id, next_node_id);
                self.path_network.add_path(next_node_id, encount_path.0);
                self.path_network.add_path(next_node_id, encount_path.1);
            }
        }

        self
    }

    pub fn build(self) -> PathNetwork<TransportNode> {
        self.path_network.into_optimized()
    }
}
