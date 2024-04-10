use crate::core::{
    container::path_network::NodeId,
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::rules::TransportRules;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TransportNode {
    pub site: Site,
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
    node_start: TransportNode,
    node_start_id: NodeId,
    angle_expected_end: Angle,
    rules: TransportRules,
}

impl Eq for PathCandidate {}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rules
            .path_priority
            .total_cmp(&other.rules.path_priority)
    }
}

type RelatedNode<'a> = (&'a TransportNode, NodeId);

impl PathCandidate {
    /// Create a new path candidate.
    pub fn new(
        node_start: TransportNode,
        node_start_id: NodeId,
        angle_expected_end: Angle,
        rules: TransportRules,
    ) -> Self {
        Self {
            node_start,
            node_start_id,
            angle_expected_end,
            rules,
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
    pub fn get_rules(&self) -> &TransportRules {
        &self.rules
    }

    /// Get the end site of the path with extra length.
    /// This is temporary used for searching intersections.
    fn get_expected_site_to_with_extra_length(&self) -> Site {
        self.node_start.site.extend(
            self.angle_expected_end,
            self.rules.path_normal_length + self.rules.path_extra_length_for_intersection,
        )
    }

    /// Determine the next node type from related(close) nodes and paths.
    pub fn determine_next_node(
        &self,
        site_expected_end: Site,
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
                    LineSegment::new(search_start, site_expected_end)
                        .get_distance(&existing_node.site)
                        < self.rules.path_extra_length_for_intersection
                })
                .filter(|(existing_node, existing_node_id)| {
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
            let search_end = self.get_expected_site_to_with_extra_length();
            let search_line = LineSegment::new(search_start, search_end);

            let crossing_path = related_paths
                .iter()
                .filter_map(|(path_start, path_end)| {
                    let path_line = LineSegment::new(path_start.0.site, path_end.0.site);

                    if let Some(intersect) = path_line.get_intersection(&search_line) {
                        return Some((TransportNode::new(intersect), (path_start.1, path_end.1)));
                    }
                    None
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
        NextTransportNode::New(TransportNode::new(site_expected_end))
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::rules::PathDirectionRules;

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
            .map(|(start, end)| (nodes_parsed[*start], nodes_parsed[*end]))
            .collect::<Vec<_>>();

        let rules = TransportRules {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 1.0,
            path_extra_length_for_intersection: 0.25,
            branch_probability: 0.0,
            path_direction_rules: PathDirectionRules::default(),
        };

        let (node_start, angle_expected_end) = (
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            Angle::new(std::f64::consts::PI * 0.75),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        // New node
        let new = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            rules.clone(),
        )
        .determine_next_node(site_expected_end, &nodes_parsed, &paths_parsed);

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
        let (node_start, angle_expected_end) = (
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            Angle::new(-std::f64::consts::PI * 0.25),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        let intersect = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            rules.clone(),
        )
        .determine_next_node(site_expected_end, &nodes_parsed, &paths_parsed);

        if let NextTransportNode::Intersect(node, _) = intersect {
            assert_eq_f64!(node.site.distance(&Site::new(0.5, 0.5)), 0.0);
        } else {
            panic!("Unexpected node type");
        }

        // Existing node (close between two nodes)
        let (node_start, angle_expected_end) = (
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            Angle::new(std::f64::consts::PI * 0.05),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        let existing = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            rules.clone(),
        )
        .determine_next_node(site_expected_end, &nodes_parsed, &paths_parsed);

        if let NextTransportNode::Existing(node_id) = existing {
            assert_eq!(node_id, NodeId::new(1));
        } else {
            panic!("Unexpected node type");
        }

        // Existing node (close between an existing node and expected path)
        let (node_start, angle_expected_end) = (
            TransportNode::default().set_site(Site::new(1.0, 0.5)),
            Angle::new(std::f64::consts::PI * 0.05),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        let existing = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            rules.clone(),
        )
        .determine_next_node(site_expected_end, &nodes_parsed, &paths_parsed);

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
            .map(|(start, end)| (nodes_parsed[*start], nodes_parsed[*end]))
            .collect::<Vec<_>>();

        let rules = TransportRules {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 10000.0,
            path_extra_length_for_intersection: 0.0,
            branch_probability: 0.0,
            path_direction_rules: PathDirectionRules::default(),
        };

        let (node_start, angle_expected_end) = (
            TransportNode::default().set_site(Site::new(-1.0, 1.0)),
            Angle::new(std::f64::consts::PI * 0.5),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        let next = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            rules.clone(),
        )
        .determine_next_node(site_expected_end, &nodes_parsed, &paths_parsed);

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
