use crate::{
    core::{
        container::path_network::NodeId,
        geometry::{angle::Angle, line_segment::LineSegment, site::Site},
        Stage,
    },
    transport::{
        metrics::PathMetrics,
        rules::{check_slope, TransportRules},
    },
};

use super::transport_node::TransportNode;

#[derive(Debug)]
pub enum NextTransportNode {
    New(TransportNode),
    Existing(NodeId),
    Intersect(TransportNode, (NodeId, NodeId)),
    None,
}

#[derive(Debug)]
pub enum BridgeNode {
    Middle(TransportNode),
    None,
}

impl BridgeNode {
    pub fn get_middle(&self) -> Option<&TransportNode> {
        match self {
            BridgeNode::Middle(node) => Some(node),
            BridgeNode::None => None,
        }
    }

    pub fn has_middle(&self) -> bool {
        match self {
            BridgeNode::Middle(_) => true,
            BridgeNode::None => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            BridgeNode::Middle(_) => false,
            BridgeNode::None => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathCandidate {
    node_start: TransportNode,
    node_start_id: NodeId,
    angle_expected_end: Angle,
    stage: Stage,
    rules_start: TransportRules,
    metrics: PathMetrics,
    evaluation: f64,
}

impl Eq for PathCandidate {}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluation.total_cmp(&other.evaluation)
    }
}

type RelatedNode<'a> = (&'a TransportNode, NodeId);

impl PathCandidate {
    /// Create a new path candidate.
    pub fn new(
        node_start: TransportNode,
        node_start_id: NodeId,
        angle_expected_end: Angle,
        stage: Stage,
        rules_start: TransportRules,
        metrics: PathMetrics,
        evaluation: f64,
    ) -> Self {
        Self {
            node_start,
            node_start_id,
            angle_expected_end,
            stage,
            rules_start,
            metrics,
            evaluation,
        }
    }

    /// Get node id
    pub fn get_node_start_id(&self) -> NodeId {
        self.node_start_id
    }

    /// Get the start site of the path.
    pub fn get_site_start(&self) -> Site {
        self.node_start.site
    }

    /// Get the end site of the path.
    pub fn angle_expected_end(&self) -> Angle {
        self.angle_expected_end
    }

    /// Get rules of the path.
    pub fn get_rules_start(&self) -> &TransportRules {
        &self.rules_start
    }

    /// Get stage of the path.
    pub fn get_stage(&self) -> Stage {
        self.stage
    }

    /// Get metrics of the path.
    pub fn get_metrics(&self) -> &PathMetrics {
        &self.metrics
    }

    /// Get the end site of the path with extra length.
    /// This is temporary used for searching intersections.
    fn get_expected_site_to_with_extra_length(&self, site_expected_end: Site) -> Site {
        let path_length = site_expected_end.distance(&self.node_start.site);
        let scale =
            (path_length + self.rules_start.path_extra_length_for_intersection) / path_length;
        Site::new(
            self.node_start.site.x + (site_expected_end.x - self.node_start.site.x) * scale,
            self.node_start.site.y + (site_expected_end.y - self.node_start.site.y) * scale,
        )
    }

    /// Determine the next node type from related(close) nodes and paths.
    pub fn determine_next_node(
        &self,
        site_expected_end: Site,
        elevation_expected_end: f64,
        stage: Stage,
        to_be_bridge: bool,
        related_nodes: &[RelatedNode],
        related_paths: &[(RelatedNode, RelatedNode)],
    ) -> (NextTransportNode, BridgeNode) {
        let search_start = self.node_start.site;

        // Existing Node
        // For this situation, path crosses are needed to be checked again because the direction of the path can be changed from original.
        {
            let existing_node_id = related_nodes
                .iter()
                .filter(|(existing_node, _)| {
                    // distance check for decreasing the number of candidates
                    LineSegment::new(search_start, site_expected_end)
                        .get_distance(&existing_node.site)
                        < self.rules_start.path_extra_length_for_intersection
                })
                .filter(|(existing_node, _)| {
                    // is_bridge check
                    // if the existing node is is_bridge, the path cannot be connected.
                    !existing_node.is_bridge
                })
                .filter(|(existing_node, existing_node_id)| {
                    // no intersection check
                    let has_intersection = related_paths.iter().any(|(path_start, path_end)| {
                        if *existing_node_id == path_start.1 || *existing_node_id == path_end.1 {
                            // ignore
                            return false;
                        }
                        let path_line = LineSegment::new(path_start.0.site, path_end.0.site);
                        let search_line = LineSegment::new(search_start, existing_node.site);
                        path_line.get_intersection(&search_line).is_some()
                    });
                    !has_intersection
                })
                .filter(|(existing_node, _)| {
                    // slope check
                    // if the elevation difference is too large, the path cannot be connected.
                    let distance = existing_node.site.distance(&search_start);
                    check_slope(
                        self.node_start.elevation,
                        existing_node.elevation,
                        distance,
                        self.rules_start.path_max_elevation_diff,
                    )
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some((existing_node, existing_node_id)) = existing_node_id {
                let middle = if to_be_bridge {
                    let middle_site = search_start.midpoint(&existing_node.site);
                    BridgeNode::Middle(TransportNode::new(
                        middle_site,
                        stage,
                        (existing_node.elevation + self.node_start.elevation) / 2.0,
                        true,
                    ))
                } else {
                    BridgeNode::None
                };
                return (NextTransportNode::Existing(*existing_node_id), middle);
            }
        }

        // Crossing Paths
        {
            let search_end = self.get_expected_site_to_with_extra_length(site_expected_end);
            let search_line = LineSegment::new(search_start, search_end);

            let crossing_path = related_paths
                .iter()
                .filter_map(|(path_start, path_end)| {
                    let path_line = LineSegment::new(path_start.0.site, path_end.0.site);

                    if let Some(intersect) = path_line.get_intersection(&search_line) {
                        let distance_0 = path_start.0.site.distance(&intersect);
                        let distance_1 = path_end.0.site.distance(&intersect);
                        let prop_start = distance_1 / (distance_0 + distance_1);
                        return Some((
                            TransportNode::new(
                                intersect,
                                path_start.0.path_stage(path_end.0),
                                path_start.0.elevation * prop_start
                                    + path_end.0.elevation * (1.0 - prop_start),
                                path_start.0.path_is_bridge(path_end.0),
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
                    check_slope(
                        self.node_start.elevation,
                        crossing_node.elevation,
                        distance,
                        self.rules_start.path_max_elevation_diff,
                    )
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some((crossing_node, path_nodes)) = crossing_path {
                // if it cross the bridge, the path cannot be connected.
                if path_nodes.0 .0.path_is_bridge(path_nodes.1 .0) {
                    return (NextTransportNode::None, BridgeNode::None);
                }
                let middle = if to_be_bridge {
                    let middle_site = search_start.midpoint(&crossing_node.site);
                    BridgeNode::Middle(TransportNode::new(
                        middle_site,
                        stage,
                        (crossing_node.elevation + self.node_start.elevation) / 2.0,
                        true,
                    ))
                } else {
                    BridgeNode::None
                };

                return (
                    NextTransportNode::Intersect(crossing_node, (path_nodes.0 .1, path_nodes.1 .1)),
                    middle,
                );
            }
        }

        // check slope
        let distance = search_start.distance(&site_expected_end);
        if !check_slope(
            self.node_start.elevation,
            elevation_expected_end,
            distance,
            self.rules_start.path_max_elevation_diff,
        ) {
            return (NextTransportNode::None, BridgeNode::None);
        }

        // New Node
        // Path crosses are already checked in the previous steps.
        let middle = if to_be_bridge {
            let middle_site = search_start.midpoint(&site_expected_end);
            BridgeNode::Middle(TransportNode::new(
                middle_site,
                stage,
                (elevation_expected_end + self.node_start.elevation) / 2.0,
                true,
            ))
        } else {
            BridgeNode::None
        };
        (
            NextTransportNode::New(TransportNode::new(
                site_expected_end,
                stage,
                elevation_expected_end,
                false,
            )),
            middle,
        )
    }
}
