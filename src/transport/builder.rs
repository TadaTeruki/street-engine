use std::collections::BinaryHeap;

use crate::core::{
    container::path_network::PathNetwork,
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::{
    node::{NextTransportNode, PathCandidate, TransportNode},
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
        let node_id = self.path_network.add_node(node);
        let property = self.property_provider.get_property(&node.into())?;
        self.path_candidate_container.push(PathCandidate::new(
            node,
            node_id,
            Angle::new(angle_radian),
            property.clone(),
        ));

        self.path_candidate_container.push(PathCandidate::new(
            node,
            node_id,
            Angle::new(angle_radian).opposite(),
            property.clone(),
        ));

        Some(self)
    }

    pub fn iterate(mut self) -> Self {
        let prior_candidate = if let Some(candidate) = self.path_candidate_container.pop() {
            candidate
        } else {
            return self;
        };

        let site_from = prior_candidate.get_site_from();
        let site_expected_to = prior_candidate.get_expected_site_to();

        let related_nodes = self
            .path_network
            .nodes_around_line_iter(
                LineSegment::new(site_from, site_expected_to),
                prior_candidate
                    .get_property()
                    .path_extra_length_for_intersection,
            )
            .filter_map(|node_id| Some((self.path_network.get_node(*node_id)?, *node_id)))
            .collect::<Vec<_>>();

        let related_paths = self
            .path_network
            .paths_touching_rect_iter(site_from, site_expected_to)
            .filter_map(|(node_id_from, node_id_to)| {
                let node_from = self.path_network.get_node(*node_id_from)?;
                let node_to = self.path_network.get_node(*node_id_to)?;
                Some(((node_from, *node_id_from), (node_to, *node_id_to)))
            })
            .collect::<Vec<_>>();

        let candidate_node_id = prior_candidate.get_node_from_id();
        let next_node = prior_candidate.determine_next_node(&related_nodes, &related_paths);

        match next_node {
            NextTransportNode::New(node_to) => {
                let node_id = self.path_network.add_node(node_to);
                self.path_network.add_path(candidate_node_id, node_id);
            }
            NextTransportNode::Existing(node_id) => {
                self.path_network.add_path(candidate_node_id, node_id);
            }
            NextTransportNode::Intersect(node_to, encount_path) => {
                let next_node_id = self.path_network.add_node(node_to);
                self.path_network
                    .remove_path(encount_path.0, encount_path.1);
                self.path_network.add_path(candidate_node_id, next_node_id);
                self.path_network.add_path(next_node_id, encount_path.0);
                self.path_network.add_path(next_node_id, encount_path.1);
            }
        }

        if let NextTransportNode::New(next_node_to) = next_node {
            let next_node_id = self.path_network.add_node(next_node_to);
            if let Some(property) = self.property_provider.get_property(&next_node_to.into()) {
                let straight_angle = site_from.get_angle(&site_expected_to);
                self.path_candidate_container.push(PathCandidate::new(
                    next_node_to,
                    next_node_id,
                    straight_angle,
                    property.clone(),
                ));

                self.path_candidate_container.push(PathCandidate::new(
                    next_node_to,
                    next_node_id,
                    straight_angle.right_clockwise(),
                    property.clone(),
                ));

                self.path_candidate_container.push(PathCandidate::new(
                    next_node_to,
                    next_node_id,
                    straight_angle.right_counterclockwise(),
                    property.clone(),
                ));
            }
        }

        self
    }

    pub fn build(self) -> PathNetwork<TransportNode> {
        self.path_network.into_optimized()
    }
}
