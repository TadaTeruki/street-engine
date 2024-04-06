use crate::core::geometry::{angle::Angle, line_segment::LineSegment, site::Site};

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
pub enum NextTransportNodeType {
    New(TransportNode),
    Existing(TransportNode),
    Intersect(TransportNode, LineSegment),
}

impl NextTransportNodeType {
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

    fn determine_node_type<'a>(
        &self,
        related_path_iter: impl Iterator<Item = (&'a TransportNode, &'a TransportNode)>,
    ) -> NextTransportNodeType {
        let site_to = self.get_site_to();

        for node in related_path_iter {}

        NextTransportNodeType::New(TransportNode::new(site_to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_candidate_cmp() {
        let nodes = vec![
            TransportNode::default().set_site(Site::new(2.0, 0.0)),
            TransportNode::default().set_site(Site::new(1.0, 0.0)),
            TransportNode::default().set_site(Site::new(0.0, 1.0)),
            TransportNode::default().set_site(Site::new(0.0, 2.0)),
        ];

        let candidate = PathCandidate::new(
            TransportNode::default().set_site(Site::new(2.0, 2.0)),
            Angle::new(-std::f64::consts::PI / 4.0),
            TransportProperty::default(),
        );
    }
}
