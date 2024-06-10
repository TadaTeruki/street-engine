use crate::{
    core::{
        container::path_network::NodeId,
        geometry::{angle::Angle, line_segment::LineSegment, site::Site},
    },
    transport::{
        params::{rules::check_elevation_diff, PathParams},
        path_network_repository::RelatedNode,
    },
};

use super::{
    growth_type::{BridgeNodeType, GrowthTypes, NextNodeType},
    transport_node::TransportNode,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NodeStump {
    node_id: NodeId,
    angle_expected: Angle,
    params: PathParams,
}

impl Eq for NodeStump {}

impl PartialOrd for NodeStump {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeStump {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.params.evaluation.total_cmp(&other.params.evaluation)
    }
}

impl NodeStump {
    /// Create a new node stump.
    pub fn new(node_id: NodeId, angle_expected: Angle, params: PathParams) -> Self {
        Self {
            node_id,
            angle_expected,
            params,
        }
    }

    /// Get node id
    pub fn get_node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get the end site of the path.
    pub fn angle_expected(&self) -> Angle {
        self.angle_expected
    }

    pub fn get_path_params(&self) -> &PathParams {
        &self.params
    }

    /// Get the end site of the path with extra length.
    /// This is temporary used for searching intersections.
    fn get_expected_site_to_with_extra_length(
        &self,
        start_site: Site,
        site_expected_end: Site,
    ) -> Site {
        let path_length = site_expected_end.distance(&start_site);
        let scale = (path_length + self.params.rules_start.path_extra_length_for_intersection)
            / path_length;
        Site::new(
            start_site.x + (site_expected_end.x - start_site.x) * scale,
            start_site.y + (site_expected_end.y - start_site.y) * scale,
        )
    }

    /// Determine the next node type from related(close) nodes and paths.
    pub fn determine_growth(
        &self,
        node_start: &TransportNode,
        node_expected_end: &TransportNode,
        related_nodes: &[RelatedNode],
        related_paths: &[(RelatedNode, RelatedNode)],
    ) -> GrowthTypes {
        let search_start = node_start.site;

        // Existing Node
        // For this situation, path crosses are needed to be checked again because the direction of the path can be changed from original.
        {
            let existing_node_id = related_nodes
                .iter()
                .filter(|existing| {
                    // distance check for decreasing the number of candidates
                    LineSegment::new(search_start, node_expected_end.site)
                        .get_distance(&existing.node.site)
                        < self.params.rules_start.path_extra_length_for_intersection
                })
                .filter(|existing| {
                    // is_bridge check
                    // if the existing node is is_bridge, the path cannot be connected.
                    !existing.node.is_bridge
                })
                .filter(|existing| {
                    // no intersection check
                    let has_intersection = related_paths.iter().any(|(path_start, path_end)| {
                        if existing.node_id == path_start.node_id
                            || existing.node_id == path_end.node_id
                        {
                            // ignore
                            return false;
                        }
                        let path_line = LineSegment::new(path_start.node.site, path_end.node.site);
                        let search_line = LineSegment::new(search_start, existing.node.site);
                        path_line.get_intersection(&search_line).is_some()
                    });
                    !has_intersection
                })
                .filter(|existing| {
                    // slope check
                    // if the elevation difference is too large, the path cannot be connected.
                    let distance = existing.node.site.distance(&search_start);
                    check_elevation_diff(
                        node_start.elevation,
                        existing.node.elevation,
                        distance,
                        self.params.rules_start.path_elevation_diff_limit,
                    )
                })
                .min_by(|a, b| {
                    let distance_a = a.node.site.distance_2(&search_start);
                    let distance_b = b.node.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some(existing) = existing_node_id {
                let middle = if node_expected_end.is_bridge {
                    let middle_site = search_start.midpoint(&existing.node.site);
                    BridgeNodeType::Middle(TransportNode::new(
                        middle_site,
                        (existing.node.elevation + node_start.elevation) / 2.0,
                        node_expected_end.stage,
                        true,
                    ))
                } else {
                    BridgeNodeType::None
                };
                return GrowthTypes {
                    next_node: NextNodeType::Existing(existing.node_id),
                    bridge_node: middle,
                };
            }
        }

        // Crossing Paths
        {
            let search_end = self
                .get_expected_site_to_with_extra_length(node_start.site, node_expected_end.site);
            let search_line = LineSegment::new(search_start, search_end);

            let crossing_path = related_paths
                .iter()
                .filter_map(|(path_start, path_end)| {
                    let path_line = LineSegment::new(path_start.node.site, path_end.node.site);

                    if let Some(intersect) = path_line.get_intersection(&search_line) {
                        let distance_0 = path_start.node.site.distance(&intersect);
                        let distance_1 = path_end.node.site.distance(&intersect);
                        let prop_start = distance_1 / (distance_0 + distance_1);
                        return Some((
                            TransportNode::new(
                                intersect,
                                path_start.node.elevation * prop_start
                                    + path_end.node.elevation * (1.0 - prop_start),
                                path_start.node.path_stage(path_end.node),
                                path_start.node.path_is_bridge(path_end.node),
                            ),
                            (path_start, path_end),
                        ));
                    }
                    None
                })
                .filter(|(crossing_node, _)| {
                    // slope check
                    // if the elevation difference is too large, the path cannot be connected.
                    let distance = crossing_node.site.distance(&search_start);
                    check_elevation_diff(
                        node_start.elevation,
                        crossing_node.elevation,
                        distance,
                        self.params.rules_start.path_elevation_diff_limit,
                    )
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some((crossing_node, (path_start, path_end))) = crossing_path {
                // if it cross the bridge, the path cannot be connected.
                if path_start.node.path_is_bridge(path_end.node) {
                    return GrowthTypes {
                        next_node: NextNodeType::None,
                        bridge_node: BridgeNodeType::None,
                    };
                }
                let middle = if node_expected_end.is_bridge {
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
                        (path_start.node_id, path_end.node_id),
                    ),
                    bridge_node: middle,
                };
            }
        }

        // check slope
        let distance = search_start.distance(&node_expected_end.site);
        if !check_elevation_diff(
            node_start.elevation,
            node_expected_end.elevation,
            distance,
            self.params.rules_start.path_elevation_diff_limit,
        ) {
            return GrowthTypes {
                next_node: NextNodeType::None,
                bridge_node: BridgeNodeType::None,
            };
        }

        // New Node
        // Path crosses are already checked in the previous steps.
        let middle = if node_expected_end.is_bridge {
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
