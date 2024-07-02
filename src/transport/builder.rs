use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::{NodeId, PathNetwork},
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::{
    growth::{
        growth_type::{BridgeNodeType, GrowthTypes, NextNodeType},
        stump::Stump,
    },
    node::TransportNode,
    params::{metrics::PathMetrics, numeric::Stage},
    traits::{PathPrioritizator, RandomF64Provider, TerrainProvider, TransportRulesProvider},
};

pub struct TransportBuilder<'a, RP, TP, PP>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
    PP: PathPrioritizator,
{
    path_network: PathNetwork<TransportNode>,
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
    path_prioritizator: &'a PP,
    stump_heap: BinaryHeap<Stump>,
}

impl<'a, RP, TP, PP> TransportBuilder<'a, RP, TP, PP>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
    PP: PathPrioritizator,
{
    /// Create a new `TransportBuilder`.
    pub fn new(
        rules_provider: &'a RP,
        terrain_provider: &'a TP,
        path_prioritizator: &'a PP,
    ) -> Self {
        Self {
            path_network: PathNetwork::new(),
            rules_provider,
            terrain_provider,
            path_prioritizator,
            stump_heap: BinaryHeap::new(),
        }
    }

    /// Add a path stump to the path network.
    fn push_new_stump(
        &mut self,
        node_start_id: NodeId,
        angle_expected_end: Angle,
        stage: Stage,
        metrics: PathMetrics,
    ) -> Option<()> {
        let node = self.path_network.get_node(node_start_id)?;

        let rules = self.rules_provider.get_rules(&node.site, stage, &metrics)?;

        let stump = Stump::create(
            self.terrain_provider,
            self.path_prioritizator,
            (node, node_start_id),
            angle_expected_end,
            stage,
            &rules,
            &metrics,
        )?;

        self.stump_heap.push(stump);

        Some(())
    }

    /// Add an origin node to the path network.
    ///
    /// The path which is extended from `origin_site` by `angle_radian` (and the opposite path) will be the first stump_heap.
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
            origin_node_id,
            Angle::new(angle_radian),
            stage,
            origin_metrics.incremented(false, false),
        );
        self.push_new_stump(
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

    /// Iterate network generation until there are no more stump_heap of new paths.
    pub fn iterate_as_possible<R>(mut self, rng: &mut R) -> Self
    where
        R: RandomF64Provider,
    {
        while !self.stump_heap.is_empty() {
            self = self.iterate::<R>(rng);
        }
        self
    }

    fn determine_growth_from_stump(&self, stump: &Stump) -> Option<GrowthTypes> {
        let stump_node = self.path_network.get_node(stump.get_node_id())?;

        // Find nodes around the line from the start site to the expected end site.
        let related_nodes = self
            .path_network
            .nodes_around_line_iter(
                LineSegment::new(stump_node.site, stump.get_node_expected_end().site),
                stump.get_rules().path_extra_length_for_intersection,
            )
            .filter(|&node_id| *node_id != stump.get_node_id())
            .filter_map(|node_id| Some((self.path_network.get_node(*node_id)?, *node_id)))
            .collect::<Vec<_>>();

        // Find paths touching the rectangle around the line.
        let related_paths = self
            .path_network
            .paths_touching_rect_iter(stump_node.site, stump.get_node_expected_end().site)
            .filter(|(node_id_start, node_id_end)| {
                *node_id_start != stump.get_node_id() && *node_id_end != stump.get_node_id()
            })
            .filter_map(|(node_id_start, node_id_end)| {
                let node_start = self.path_network.get_node(*node_id_start)?;
                let node_end = self.path_network.get_node(*node_id_end)?;
                Some(((node_start, *node_id_start), (node_end, *node_id_end)))
            })
            .collect::<Vec<_>>();

        // Determine the growth of the path.
        let growth = stump.determine_growth(stump_node, &related_nodes, &related_paths);

        Some(growth)
    }

    /// Iterate the path network to the next step.
    pub fn iterate<R>(mut self, rng: &mut R) -> Self
    where
        R: RandomF64Provider,
    {
        let prior_stump = if let Some(stump) = self.stump_heap.pop() {
            stump
        } else {
            return self;
        };

        let growth = if let Some(growth) = self.determine_growth_from_stump(&prior_stump) {
            growth
        } else {
            return self;
        };

        self.apply_next_growth(
            rng,
            growth.next_node,
            growth.bridge_node,
            prior_stump.get_node_id(),
            &prior_stump,
        )
    }

    fn apply_next_growth<R>(
        mut self,
        rng: &mut R,
        next_node_type: NextNodeType,
        bridge_node_type: BridgeNodeType,
        stump_node_id: NodeId,
        stump: &Stump,
    ) -> Self
    where
        R: RandomF64Provider,
    {
        if let BridgeNodeType::Middle(bridge_node) = bridge_node_type {
            let bridge_node_id = self.path_network.add_node(bridge_node);
            self.path_network.add_path(stump_node_id, bridge_node_id);

            return self.apply_next_growth(
                rng,
                next_node_type,
                BridgeNodeType::None,
                bridge_node_id,
                stump,
            );
        }

        let start_site = if let Some(node) = self.path_network.get_node(stump_node_id) {
            node.site
        } else {
            return self;
        };

        match next_node_type {
            NextNodeType::None => {
                return self;
            }
            NextNodeType::Existing(node_id) => {
                self.path_network.add_path(stump_node_id, node_id);
            }
            NextNodeType::Intersect(node_next, encount_path) => {
                let next_node_id = self.path_network.add_node(node_next);
                self.path_network
                    .remove_path(encount_path.0, encount_path.1);
                self.path_network.add_path(stump_node_id, next_node_id);
                self.path_network.add_path(next_node_id, encount_path.0);
                self.path_network.add_path(next_node_id, encount_path.1);
            }
            NextNodeType::New(node_next) => {
                let node_id = self.path_network.add_node(node_next);
                self.path_network.add_path(stump_node_id, node_id);

                let straight_angle = start_site.get_angle(&node_next.site);
                self.push_new_stump(
                    node_id,
                    straight_angle,
                    stump.get_stage(),
                    stump.get_metrics().incremented(false, false),
                );
                let clockwise_branch =
                    rng.gen_f64() < stump.get_rules().branch_rules.branch_density;
                if clockwise_branch {
                    let clockwise_staging =
                        rng.gen_f64() < stump.get_rules().branch_rules.staging_probability;
                    let next_stage = if clockwise_staging {
                        stump.get_stage().incremented()
                    } else {
                        stump.get_stage()
                    };
                    self.push_new_stump(
                        node_id,
                        straight_angle.right_clockwise(),
                        next_stage,
                        stump.get_metrics().incremented(clockwise_staging, true),
                    );
                }

                let counterclockwise_branch =
                    rng.gen_f64() < stump.get_rules().branch_rules.branch_density;
                if counterclockwise_branch {
                    let counterclockwise_staging =
                        rng.gen_f64() < stump.get_rules().branch_rules.staging_probability;
                    let next_stage = if counterclockwise_staging {
                        stump.get_stage().incremented()
                    } else {
                        stump.get_stage()
                    };
                    self.push_new_stump(
                        node_id,
                        straight_angle.right_counterclockwise(),
                        next_stage,
                        stump
                            .get_metrics()
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
