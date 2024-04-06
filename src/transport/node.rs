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
    Existing(TransportNode),
    Intersect(TransportNode, (NodeId, NodeId)),
}

impl NextTransportNode {
    fn node_to(&self) -> TransportNode {
        match self {
            Self::New(node) => *node,
            Self::Existing(node) => *node,
            Self::Intersect(node, _) => *node,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathCandidate {
    node_from: TransportNode,
    angle_to: Angle,
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

impl PathCandidate {
    pub fn new(node_from: TransportNode, angle_to: Angle, property: TransportProperty) -> Self {
        Self {
            node_from,
            angle_to,
            property,
        }
    }

    fn get_site_to(&self) -> Site {
        self.node_from
            .site
            .extend(self.angle_to, self.property.path_normal_length)
    }

    fn determine_node_type(
        &self,
        related_paths: &[(&TransportNode, &TransportNode)],
    ) -> NextTransportNode {
        let site_to = self.get_site_to();

        for node in related_paths.iter() {}

        NextTransportNode::New(TransportNode::new(site_to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_node() {
        let nodes = vec![
            TransportNode::default().set_site(Site::new(3.0, 0.0)),
            TransportNode::default().set_site(Site::new(1.0, 0.0)),
            TransportNode::default().set_site(Site::new(0.0, 1.0)),
            TransportNode::default().set_site(Site::new(0.0, 3.0)),
        ];

        let paths = vec![
            (&nodes[0], &nodes[1]),
            (&nodes[1], &nodes[2]),
            (&nodes[2], &nodes[3]),
        ];

        let property = TransportProperty {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 1.0,
            path_merge_length: 0.25,
            branch_probability: 0.0,
            curve: None,
        };

        // New node
        let new = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            Angle::new(std::f64::consts::PI * 0.75),
            property.clone(),
        )
        .determine_node_type(&paths);

        assert!(matches!(new, NextTransportNode::New(_)));
        assert_eq!(
            new.node_to().site,
            Site::new(1.0 + 1.0 / 2.0_f64.sqrt(), 1.0 + 1.0 / 2.0_f64.sqrt())
        );

        // Intersect (Close Path)
        let intersect = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            Angle::new(std::f64::consts::PI * 0.25),
            property.clone(),
        )
        .determine_node_type(&paths);

        assert!(matches!(intersect, NextTransportNode::Intersect(_, _)));

        assert_eq!(
            intersect.node_to().site,
            Site::new(1.0 + 1.0 / 2.0_f64.sqrt(), 0.0)
        );

        // Intersect (Crossing Path)
        let intersect = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            Angle::new(-std::f64::consts::PI * 0.25),
            property.clone(),
        )
        .determine_node_type(&paths);

        assert!(matches!(intersect, NextTransportNode::Intersect(_, _)));

        assert_eq!(intersect.node_to().site, Site::new(0.5, 0.5));

        // Existing node
        let existing = PathCandidate::new(
            TransportNode::default().set_site(Site::new(1.0, 1.0)),
            Angle::new(std::f64::consts::PI * 0.05),
            property.clone(),
        )
        .determine_node_type(&paths);

        assert!(matches!(existing, NextTransportNode::Existing(_)));

        assert_eq!(existing.node_to(), nodes[1]);
    }

    #[test]
    fn test_next_node_across_multiple_paths() {
        let nodes_upper = vec![
            TransportNode::default().set_site(Site::new(0.0, 0.0)),
            TransportNode::default().set_site(Site::new(0.3, 0.0)),
            TransportNode::default().set_site(Site::new(0.7, 0.0)),
            TransportNode::default().set_site(Site::new(1.0, 0.0)),
        ];
        let nodes_lower = vec![
            TransportNode::default().set_site(Site::new(0.0, 10.0)),
            TransportNode::default().set_site(Site::new(0.3, 10.0)),
            TransportNode::default().set_site(Site::new(0.7, 10.0)),
            TransportNode::default().set_site(Site::new(1.0, 10.0)),
        ];

        let paths = vec![
            (&nodes_upper[0], &nodes_lower[1]),
            (&nodes_lower[1], &nodes_upper[2]),
            (&nodes_upper[2], &nodes_lower[3]),
            (&nodes_lower[3], &nodes_upper[3]),
            (&nodes_upper[3], &nodes_lower[2]),
            (&nodes_lower[2], &nodes_upper[1]),
            (&nodes_upper[1], &nodes_lower[0]),
        ];

        let property = TransportProperty {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 10000.0,
            path_merge_length: 0.0,
            branch_probability: 0.0,
            curve: None,
        };

        let next = PathCandidate::new(
            TransportNode::default().set_site(Site::new(-1.0, 1.0)),
            Angle::new(0.),
            property.clone(),
        )
        .determine_node_type(&paths);

        assert!(matches!(next, NextTransportNode::Intersect(_, _)));

        let next_site = next.node_to().site;

        assert!(
            (next_site.x >= 0.0 && next_site.x <= 0.3)
                && (next_site.y >= 0.0 && next_site.y <= 5.0)
        );
    }
}
