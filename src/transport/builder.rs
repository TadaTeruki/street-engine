use std::collections::BinaryHeap;

use crate::core::{
    container::network::Network,
    geometry::{angle::Angle, site::Site},
};

use super::{
    node::{PathCandidate, TransportNode},
    property::TransportPropertyProvider,
};

struct TransportBuilder<'a, TP>
where
    TP: TransportPropertyProvider,
{
    network: Network<TransportNode>,
    property_provider: &'a TP,
    path_candidate_container: BinaryHeap<PathCandidate>,
}

impl<'a, TP> TransportBuilder<'a, TP>
where
    TP: TransportPropertyProvider,
{
    fn new(property_provider: &'a TP) -> Self {
        Self {
            network: Network::new(),
            property_provider,
            path_candidate_container: BinaryHeap::new(),
        }
    }

    fn add_origin(mut self, site: Site, angle_radian: f64) -> Self {
        let origin = TransportNode::new(site);
        let path_priority = self
            .property_provider
            .get_property(&origin.into())
            .path_priority;
        self.path_candidate_container.push(PathCandidate::new(
            origin,
            Angle::new(angle_radian),
            path_priority,
        ));
        self.path_candidate_container.push(PathCandidate::new(
            origin,
            Angle::new(angle_radian).opposite(),
            path_priority,
        ));
        self
    }

    pub fn iterate(mut self) -> Self {
        let prior_candidate = if let Some(candidate) = self.path_candidate_container.pop() {
            candidate
        } else {
            return self;
        };

        self
    }
}
