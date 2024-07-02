use crate::{
    core::{
        container::path_network::NodeId,
        geometry::{angle::Angle, line_segment::LineSegment, site::Site},
    },
    transport::{
        params::{
            metrics::PathMetrics, numeric::Stage, priority::PathPrioritizationFactors,
            rules::TransportRules,
        },
        traits::{PathPrioritizator, TerrainProvider},
    },
};

use super::{
    growth_type::{BridgeNodeType, GrowthTypes, NextNodeType},
    transport_node::TransportNode,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Stump {
    /// node id which this stump is created for.
    node_id: NodeId,
    /// expected end node of the path.
    node_expected_end: TransportNode,
    /// rules for the path to be created by this stump.
    rules: TransportRules,
    /// metrics for the path to be created by this stump.
    metrics: PathMetrics,
    /// priority of stump to be dequed.
    priority: f64,
    /// if the path is to be created is a bridge.
    creates_bridge: bool,
}

impl Eq for Stump {}

impl PartialOrd for Stump {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Stump {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.total_cmp(&other.priority)
    }
}

type RelatedNode<'a> = (&'a TransportNode, NodeId);

impl Stump {
    /// Create a new stump for the given conditions.
    pub fn create<TP, PP>(
        terrain_provider: &TP,
        path_prioritizator: &PP,
        node_tuple: (&TransportNode, NodeId),
        angle_expected: Angle,
        stage: Stage,
        rules: &TransportRules,
        metrics: &PathMetrics,
    ) -> Option<Self>
    where
        TP: TerrainProvider,
        PP: PathPrioritizator,
    {
        let (node, node_id) = node_tuple;

        let path_direction_rules = &rules.path_direction_rules;
        let (estimated_end_site, creates_bridge) = angle_expected
            .iter_range_around(
                path_direction_rules.max_radian,
                path_direction_rules.comparison_step,
            )
            .filter_map(|angle| {
                for i in 0..=rules.bridge_rules.check_step {
                    let bridge_path_length = if rules.bridge_rules.check_step == 0 {
                        0.0
                    } else {
                        rules.bridge_rules.max_bridge_length * (i as f64)
                            / (rules.bridge_rules.check_step as f64)
                    };
                    let path_length = rules.path_normal_length + bridge_path_length;
                    let site_end = node.site.extend(angle, path_length);
                    let creates_bridge = i > 0;
                    if let Some(priority) =
                        path_prioritizator.prioritize(PathPrioritizationFactors {
                            site_start: node.site,
                            site_end,
                            path_length,
                            stage,
                            creates_bridge,
                        })
                    {
                        if let (Some(elevation_start), Some(elevation_end)) = (
                            terrain_provider.get_elevation(&node.site),
                            terrain_provider.get_elevation(&site_end),
                        ) {
                            if rules
                                .path_slope_elevation_diff_limit
                                .check_slope((elevation_start, elevation_end), path_length)
                            {
                                return Some((site_end, priority, creates_bridge));
                            }
                        }
                    }
                }
                None
            })
            .max_by(|(_, ev0, _), (_, ev1, _)| ev0.total_cmp(ev1))
            .map(|(site, _, creates_bridge)| (site, creates_bridge))?;

        let priority = path_prioritizator.prioritize(PathPrioritizationFactors {
            site_start: node.site,
            site_end: estimated_end_site,
            path_length: rules.path_normal_length,
            stage,
            creates_bridge,
        })?;

        Some(Self {
            node_id,
            node_expected_end: TransportNode::new(
                estimated_end_site,
                terrain_provider.get_elevation(&estimated_end_site)?,
                stage,
                false,
            ),
            rules: rules.clone(),
            metrics: metrics.clone(),
            priority,
            creates_bridge,
        })
    }

    pub fn get_node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn get_node_expected_end(&self) -> &TransportNode {
        &self.node_expected_end
    }

    pub fn get_rules(&self) -> &TransportRules {
        &self.rules
    }

    pub fn get_metrics(&self) -> &PathMetrics {
        &self.metrics
    }

    pub fn get_stage(&self) -> Stage {
        self.node_expected_end.stage
    }

    /// Get the end site of the path with extra length.
    /// This is temporary used for searching intersections.
    fn get_expected_site_to_with_extra_length(
        &self,
        start_site: Site,
        site_expected_end: Site,
    ) -> Site {
        let path_length = site_expected_end.distance(&start_site);
        let scale = (path_length + self.rules.path_extra_length_for_intersection) / path_length;
        Site::new(
            start_site.x + (site_expected_end.x - start_site.x) * scale,
            start_site.y + (site_expected_end.y - start_site.y) * scale,
        )
    }

    /// Check elevation difference of two paths to determine if the paths can be grade separated.
    fn can_create_grade_separated(&self, elevation0: f64, elevation1: f64) -> bool {
        let diff = (elevation0 - elevation1).abs();
        diff > self.rules.path_grade_separation_elevation_diff_threshold
    }

    fn get_crossing<'a>(
        line: &'a LineSegment,
        related_paths: &'a [(RelatedNode<'a>, RelatedNode<'a>)],
    ) -> Vec<(&'a RelatedNode<'a>, &'a RelatedNode<'a>, (Site, f64))> {
        related_paths
            .iter()
            .filter_map(|(path_start, path_end)| {
                let path_line = LineSegment::new(path_start.0.site, path_end.0.site);
                if let Some(intersect) = path_line.get_intersection(line) {
                    let elevation = path_start.0.elevation_on_path(path_end.0, intersect);
                    return Some((path_start, path_end, (intersect, elevation)));
                }
                None
            })
            .collect::<Vec<_>>()
    }

    fn check_slope(&self, node0: &TransportNode, node1: &TransportNode) -> bool {
        // slope check
        // if the elevation difference is too large, the path cannot be connected.
        let distance = node0.site.distance(&node1.site);
        self.rules
            .path_slope_elevation_diff_limit
            .check_slope((node0.elevation, node1.elevation), distance)
    }

    /// Determine the next node type from related(close) nodes and paths.
    pub fn determine_growth(
        &self,
        node_start: &TransportNode,
        related_nodes: &[RelatedNode],
        related_paths: &[(RelatedNode, RelatedNode)],
    ) -> GrowthTypes {
        let search_start = node_start.site;
        let node_expected_end = &self.node_expected_end;

        // Existing Node
        // For this situation, path crosses are needed to be checked again because the direction of the path can be changed from original.
        {
            let existing_node_id = related_nodes
                .iter()
                .filter(|(existing_node, _)| {
                    // distance check for decreasing the number of candidates
                    LineSegment::new(search_start, node_expected_end.site)
                        .get_distance(&existing_node.site)
                        < self.rules.path_extra_length_for_intersection
                })
                .filter(|(existing_node, _)| {
                    // creates_bridge check
                    // if the existing node is creates_bridge, the path cannot be connected.
                    !existing_node.is_bridge
                })
                .filter(|(existing_node, existing_node_id)| {
                    // no intersection check
                    Self::get_crossing(
                        &LineSegment::new(search_start, existing_node.site),
                        related_paths,
                    )
                    .iter()
                    .filter(|(path_start, path_end, _)| {
                        *existing_node_id != path_start.1 && *existing_node_id != path_end.1
                    })
                    .filter(|(_, _, (_, intersect_elevation))| {
                        // if the path must be grade separated, intersection cannot be created.
                        !self.can_create_grade_separated(
                            *intersect_elevation,
                            existing_node.elevation,
                        )
                    })
                    .count()
                        == 0
                })
                .filter(|(existing_node, _)| self.check_slope(node_start, existing_node))
                .min_by(|a, b: &&(&TransportNode, NodeId)| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some((existing_node, existing_node_id)) = existing_node_id {
                let middle = if self.creates_bridge {
                    let middle_site = search_start.midpoint(&existing_node.site);
                    BridgeNodeType::Middle(TransportNode::new(
                        middle_site,
                        (existing_node.elevation + node_start.elevation) / 2.0,
                        node_expected_end.stage,
                        true,
                    ))
                } else {
                    BridgeNodeType::None
                };
                return GrowthTypes {
                    next_node: NextNodeType::Existing(*existing_node_id),
                    bridge_node: middle,
                };
            }
        }

        // Crossing Paths
        {
            let search_end = self
                .get_expected_site_to_with_extra_length(node_start.site, node_expected_end.site);
            let search_line = LineSegment::new(search_start, search_end);

            let crossings = Self::get_crossing(&search_line, related_paths);
            let crossing_path = crossings
                .iter()
                .map(|(path_start, path_end, (intersect_site, _))| {
                    (
                        TransportNode::new(
                            *intersect_site,
                            path_start.0.elevation_on_path(path_end.0, *intersect_site),
                            path_start.0.path_stage(path_end.0),
                            path_start.0.path_creates_bridge(path_end.0),
                        ),
                        (path_start, path_end),
                    )
                })
                .filter(|(crossing_node, _)| {
                    // check slope
                    self.check_slope(node_start, crossing_node)
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some((crossing_node, path_nodes)) = crossing_path {
                // if it cross the bridge, the path cannot be connected.
                if path_nodes.0 .0.path_creates_bridge(path_nodes.1 .0) {
                    return GrowthTypes {
                        next_node: NextNodeType::None,
                        bridge_node: BridgeNodeType::None,
                    };
                }
                let middle = if self.creates_bridge {
                    let middle_site = search_start.midpoint(&crossing_node.site);
                    BridgeNodeType::Middle(TransportNode::new(
                        middle_site,
                        (crossing_node.elevation + node_start.elevation) / 2.0,
                        node_expected_end.stage,
                        true,
                    ))
                } else {
                    BridgeNodeType::None
                };
                return GrowthTypes {
                    next_node: NextNodeType::Intersect(
                        crossing_node,
                        (path_nodes.0 .1, path_nodes.1 .1),
                    ),
                    bridge_node: middle,
                };
            }

            // if no intersection is created and there are existing paths
            // which prevent the incoming path from being created as grade separated, the path cannot be connected.
            if crossings.iter().any(|(_, _, (_, intersect_elevation))| {
                !self.can_create_grade_separated(*intersect_elevation, node_expected_end.elevation)
            }) {
                return GrowthTypes {
                    next_node: NextNodeType::None,
                    bridge_node: BridgeNodeType::None,
                };
            }
        }

        // check slope
        if !self.check_slope(node_start, node_expected_end) {
            return GrowthTypes {
                next_node: NextNodeType::None,
                bridge_node: BridgeNodeType::None,
            };
        }

        // New Node
        // Path crosses are already checked in the previous steps.
        let middle = if self.creates_bridge {
            let middle_site = search_start.midpoint(&node_expected_end.site);
            BridgeNodeType::Middle(TransportNode::new(
                middle_site,
                (node_expected_end.elevation + node_start.elevation) / 2.0,
                node_expected_end.stage,
                true,
            ))
        } else {
            BridgeNodeType::None
        };
        GrowthTypes {
            next_node: NextNodeType::New(TransportNode::new(
                node_expected_end.site,
                node_expected_end.elevation,
                node_expected_end.stage,
                false,
            )),
            bridge_node: middle,
        }
    }
}
