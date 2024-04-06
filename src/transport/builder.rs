use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::PathNetwork,
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::{
    node::{PathCandidate, TransportNode},
    property::TransportPropertyProvider,
};

pub struct TransportBuilder<'a, TP>
where
    TP: TransportPropertyProvider,
{
    path_network: PathNetwork<TransportNode>,
    property_provider: &'a TP,
    path_candidate_container: BinaryHeap<PathCandidate>,
}

impl<'a, TP> TransportBuilder<'a, TP>
where
    TP: TransportPropertyProvider,
{
    pub fn new(property_provider: &'a TP) -> Self {
        Self {
            path_network: PathNetwork::new(),
            property_provider,
            path_candidate_container: BinaryHeap::new(),
        }
    }

    pub fn add_origin(mut self, site: Site, angle_radian: f64) -> Option<Self> {
        let node = TransportNode::new(site);
        self.path_network.add_node(node);
        let property = self.property_provider.get_property(&node.into())?;
        self.path_candidate_container.push(PathCandidate::new(
            node,
            Angle::new(angle_radian),
            property.clone(),
        ));

        self.path_candidate_container.push(PathCandidate::new(
            node,
            Angle::new(angle_radian).opposite(),
            property.clone(),
        ));

        Some(self)
    }
}
