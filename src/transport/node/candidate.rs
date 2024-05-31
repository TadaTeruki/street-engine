use crate::{
    core::{
        container::path_network::NodeId,
        geometry::{angle::Angle, line_segment::LineSegment, site::Site},
        Stage,
    },
    transport::rules::TransportRules,
};

use super::node::TransportNode;

#[derive(Debug)]
pub enum NextTransportNode {
    New(TransportNode),
    NewBridge(TransportNode, TransportNode),
    Existing(NodeId),
    Intersect(TransportNode, (NodeId, NodeId)),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathCandidate {
    node_start: TransportNode,
    node_start_id: NodeId,
    angle_expected_end: Angle,
    stage: Stage,
    rules_start: TransportRules,
}

impl Eq for PathCandidate {}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rules_start
            .path_priority
            .total_cmp(&other.rules_start.path_priority)
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
    ) -> Self {
        Self {
            node_start,
            node_start_id,
            angle_expected_end,
            stage,
            rules_start,
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

    /// Get the end site of the path with extra length.
    /// This is temporary used for searching intersections.
    fn get_expected_site_to_with_extra_length(&self, path_length: f64) -> Site {
        self.node_start.site.extend(
            self.angle_expected_end,
            path_length + self.rules_start.path_extra_length_for_intersection,
        )
    }

    /// Determine the next node type from related(close) nodes and paths.
    pub fn determine_next_node(
        &self,
        site_expected_end: Site,
        rules_end: TransportRules,
        stage: Stage,
        to_be_bridge_end: bool,
        related_nodes: &[RelatedNode],
        related_paths: &[(RelatedNode, RelatedNode)],
    ) -> NextTransportNode {
        let search_start = self.node_start.site;

        // Existing Node
        // For this situation, path crosses are needed to be checked again because the direction of the path can be changed from original.
        {
            let existing_node = related_nodes
                .iter()
                .filter(|(existing_node, _)| {
                    // distance check
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
                    let path_length = search_start.distance(&existing_node.site);
                    self.rules_start.check_slope(&rules_end, path_length)
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                })
                .map(|(_, node_id)| NextTransportNode::Existing(*node_id));

            if let Some(existing_node) = existing_node {
                return existing_node;
            }
        }

        // Crossing Paths
        {
            let search_end = self
                .get_expected_site_to_with_extra_length(site_expected_end.distance(&search_start));
            let search_line = LineSegment::new(search_start, search_end);

            let crossing_path = related_paths
                .iter()
                .filter(|(path_start, path_end)| {
                    // is_bridge check
                    !path_start.0.path_is_bridge(path_end.0)
                })
                .filter_map(|(path_start, path_end)| {
                    let path_line = LineSegment::new(path_start.0.site, path_end.0.site);

                    if let Some(intersect) = path_line.get_intersection(&search_line) {
                        return Some((
                            TransportNode::new(intersect, stage, false),
                            (path_start.1, path_end.1),
                        ));
                    }
                    None
                })
                .filter(|(crossing_node, _)| {
                    // slope check
                    let path_length = search_start.distance(&crossing_node.site);
                    self.rules_start.check_slope(&rules_end, path_length)
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                })
                .map(|(node, node_ids)| NextTransportNode::Intersect(node, node_ids));

            if let Some(crossing_path) = crossing_path {
                return crossing_path;
            }
        }

        // New Node
        // Path crosses are already checked in the previous steps.
        if to_be_bridge_end {
            let middle_site = search_start.midpoint(&site_expected_end);
            NextTransportNode::NewBridge(
                TransportNode::new(middle_site, stage, true),
                TransportNode::new(site_expected_end, stage, false),
            )
        } else {
            NextTransportNode::New(TransportNode::new(site_expected_end, stage, false))
        }
    }
}
