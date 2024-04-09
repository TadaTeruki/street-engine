use crate::core::{
    container::path_network::NodeId,
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::property::TransportProperty;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TransportNode {
    site: Site,
}

impl TransportNode {
    pub fn new(site: Site) -> Self {
        Self { site }
    }

    pub fn set_site(mut self, site: Site) -> Self {
        self.site = site;
        self
    }
}

impl From<TransportNode> for Site {
    fn from(node: TransportNode) -> Self {
        node.site
    }
}

impl PartialOrd for TransportNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransportNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.site.cmp(&other.site)
    }
}

#[derive(Debug)]
pub enum NextTransportNode {
    New(TransportNode),
    Existing(NodeId),
    Intersect(TransportNode, (NodeId, NodeId)),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathCandidate {
    node_from: TransportNode,
    node_from_id: NodeId,
    angle_expected_to: Angle,
    property: TransportProperty,
}

impl Eq for PathCandidate {}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.property
            .path_priority
            .total_cmp(&other.property.path_priority)
    }
}

type RelatedNode<'a> = (&'a TransportNode, NodeId);

impl PathCandidate {
    /// Create a new path candidate.
    pub fn new(
        node_from: TransportNode,
        node_from_id: NodeId,
        angle_expected_to: Angle,
        property: TransportProperty,
    ) -> Self {
        Self {
            node_from,
            node_from_id,
            angle_expected_to,
            property,
        }
    }

    /// Get node id
    pub fn get_node_from_id(&self) -> NodeId {
        self.node_from_id
    }

    /// Get the start site of the path.
    pub fn get_site_from(&self) -> Site {
        self.node_from.site
    }

    /// Get the end site of the path.
    pub fn get_expected_site_to(&self) -> Site {
        self.node_from
            .site
            .extend(self.angle_expected_to, self.property.path_normal_length)
    }

    /// Get property of the path.
    pub fn get_property(&self) -> &TransportProperty {
        &self.property
    }

    /// Get the end site of the path with extra length.
    /// This is temporary used for searching intersections.
    fn get_expected_site_to_with_extra_length(&self) -> Site {
        self.node_from.site.extend(
            self.angle_expected_to,
            self.property.path_normal_length + self.property.path_extra_length_for_intersection,
        )
    }

    /// Determine the next node type from related(close) nodes and paths.
    pub fn determine_next_node(
        &self,
        related_nodes: &[RelatedNode],
        related_paths: &[(RelatedNode, RelatedNode)],
    ) -> NextTransportNode {
        // Crossing Paths
        let search_from = self.node_from.site;
        let site_expected_to = self.get_expected_site_to();

        // Existing Node
        // For this situation, path crosses are needed to be checked again because the direction of the path can be changed from original.
        {
            let existing_node = related_nodes
                .iter()
                .filter(|(existing_node, _)| {
                    let range_2 = self.property.path_extra_length_for_intersection.powi(2);
                    existing_node.site.distance_2(&site_expected_to) < range_2
                        || LineSegment::new(search_from, site_expected_to)
                            .get_projection(&existing_node.site)
                            .map(|projection| existing_node.site.distance_2(&projection) < range_2)
                            == Some(true)
                })
                .filter(|(existing_node, existing_node_id)| {
                    let has_intersection = related_paths.iter().any(|(path_from, path_to)| {
                        if *existing_node_id == path_from.1 || *existing_node_id == path_to.1 {
                            // ignore
                            return false;
                        }
                        let path_line = LineSegment::new(path_from.0.site, path_to.0.site);
                        let search_line = LineSegment::new(search_from, existing_node.site);
                        path_line.get_intersection(&search_line).is_some()
                    });
                    !has_intersection
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_from);
                    let distance_b = b.0.site.distance_2(&search_from);
                    distance_a.total_cmp(&distance_b)
                })
                .map(|(_, node_id)| NextTransportNode::Existing(*node_id));

            if let Some(existing_node) = existing_node {
                return existing_node;
            }
        }

        {
            let search_to = self.get_expected_site_to_with_extra_length();
            let search_line = LineSegment::new(search_from, search_to);

            let crossing_path = related_paths
                .iter()
                .filter_map(|(path_from, path_to)| {
                    let path_line = LineSegment::new(path_from.0.site, path_to.0.site);

                    if let Some(intersect) = path_line.get_intersection(&search_line) {
                        return Some((TransportNode::new(intersect), (path_from.1, path_to.1)));
                    }
                    None
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_from);
                    let distance_b = b.0.site.distance_2(&search_from);
                    distance_a.total_cmp(&distance_b)
                })
                .map(|(node, node_ids)| NextTransportNode::Intersect(node, node_ids));

            if let Some(crossing_path) = crossing_path {
                return crossing_path;
            }
        }

        // New Node
        // Path crosses are already checked in the previous steps.
        NextTransportNode::New(TransportNode::new(site_expected_to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_eq_f64 {
        ($a:expr, $b:expr) => {
            assert!(($a - $b).abs() < 1e-6);
        };
    }

    #[test]
    fn test_next_node() {
        let nodes = vec![
            TransportNode::default().set_site(Site::new(3.0, 0.0)),
            TransportNode::default().set_site(Site::new(1.0, 0.0)),
            TransportNode::default().set_site(Site::new(0.0, 1.0)),
            TransportNode::default().set_site(Site::new(0.0, 3.0)),
        ];

        let nodes_parsed = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node, NodeId::new(i)))
            .collect::<Vec<_>>();

        let paths = vec![(0, 1), (1, 2), (2, 3)];

        let paths_parsed = paths
            .iter()
            .map(|(from, to)| (nodes_parsed[*from], nodes_parsed[*to]))
            .collect::<Vec<_>>();

        let property = TransportProperty {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 1.0,
            path_extra_length_for_intersection: 0.25,
            branch_probability: 0.0,
            curve: None,
        };

        // New node
        let new = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            NodeId::new(10000),
            Angle::new(std::f64::consts::PI * 0.75),
            property.clone(),
        )
        .determine_next_node(&nodes_parsed, &paths_parsed);

        if let NextTransportNode::New(node) = new {
            assert_eq_f64!(
                node.site.distance(&Site::new(
                    1.0 + 1.0 / 2.0_f64.sqrt(),
                    1.0 + 1.0 / 2.0_f64.sqrt()
                )),
                0.0
            );
        } else {
            panic!("Unexpected node type");
        }

        // Intersect (Crossing Path)
        let intersect = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            NodeId::new(10000),
            Angle::new(-std::f64::consts::PI * 0.25),
            property.clone(),
        )
        .determine_next_node(&nodes_parsed, &paths_parsed);

        if let NextTransportNode::Intersect(node, _) = intersect {
            assert_eq_f64!(node.site.distance(&Site::new(0.5, 0.5)), 0.0);
        } else {
            panic!("Unexpected node type");
        }

        // Existing node (close between two nodes)
        let existing = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            NodeId::new(10000),
            Angle::new(std::f64::consts::PI * 0.05),
            property.clone(),
        )
        .determine_next_node(&nodes_parsed, &paths_parsed);

        if let NextTransportNode::Existing(node_id) = existing {
            assert_eq!(node_id, NodeId::new(1));
        } else {
            panic!("Unexpected node type");
        }
        // Existing node (close between an existing node and expected path)
        let existing = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 0.5)),
            NodeId::new(10000),
            Angle::new(std::f64::consts::PI * 0.05),
            property.clone(),
        )
        .determine_next_node(&nodes_parsed, &paths_parsed);

        if let NextTransportNode::Existing(node_id) = existing {
            assert_eq!(node_id, NodeId::new(1));
        } else {
            panic!("Unexpected node type");
        }
    }

    #[test]
    fn test_next_node_across_multiple_paths() {
        let nodes = vec![
            TransportNode::default().set_site(Site::new(0.0, 0.0)),
            TransportNode::default().set_site(Site::new(0.3, 0.0)),
            TransportNode::default().set_site(Site::new(0.7, 0.0)),
            TransportNode::default().set_site(Site::new(1.0, 0.0)),
            TransportNode::default().set_site(Site::new(0.0, 10.0)),
            TransportNode::default().set_site(Site::new(0.3, 10.0)),
            TransportNode::default().set_site(Site::new(0.7, 10.0)),
            TransportNode::default().set_site(Site::new(1.0, 10.0)),
        ];

        let nodes_parsed = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node, NodeId::new(i)))
            .collect::<Vec<_>>();

        let paths = vec![(0, 5), (5, 2), (2, 7), (7, 3), (3, 6), (6, 1), (1, 4)];

        let paths_parsed = paths
            .iter()
            .map(|(from, to)| (nodes_parsed[*from], nodes_parsed[*to]))
            .collect::<Vec<_>>();

        let property = TransportProperty {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 10000.0,
            path_extra_length_for_intersection: 0.0,
            branch_probability: 0.0,
            curve: None,
        };

        let next = PathCandidate::new(
            TransportNode::default().set_site(Site::new(-1.0, 1.0)),
            NodeId::new(10000),
            Angle::new(std::f64::consts::PI * 0.5),
            property.clone(),
        )
        .determine_next_node(&nodes_parsed, &paths_parsed);

        println!("{:?}", next);

        assert!(matches!(next, NextTransportNode::Intersect(_, _)));
        if let NextTransportNode::Intersect(node, _) = next {
            assert!(
                (node.site.x >= 0.0 && node.site.x <= 0.3)
                    && (node.site.y >= 0.0 && node.site.y <= 5.0)
            );
        } else {
            panic!("Unexpected node type");
        }
    }
}
