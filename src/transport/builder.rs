use std::collections::BinaryHeap;

use crate::core::{
    container::network::Network,
    geometry::{angle::Angle, line_segment::LineSegment, site::Site},
};

use super::{
    node::{PathCandidate, TransportNode},
    property::TransportPropertyProvider,
};

enum NextTransportNodeType {
    New(TransportNode),
    Existing(TransportNode),
    Intersect(TransportNode, LineSegment<TransportNode>),
}

pub struct TransportBuilder<'a, TP>
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
    pub fn new(property_provider: &'a TP) -> Self {
        Self {
            network: Network::new(),
            property_provider,
            path_candidate_container: BinaryHeap::new(),
        }
    }

    pub fn add_origin(
        mut self,
        site: Site,
        elevated_height: f64,
        angle_radian: f64,
    ) -> Option<Self> {
        let origin = TransportNode::new(site, elevated_height);
        let property = self.property_provider.get_property(&origin.into())?;
        self.path_candidate_container.push(PathCandidate::new(
            origin,
            Angle::new(angle_radian),
            property.path_length,
            property.path_priority,
            property.curve.clone(),
        ));
        self.path_candidate_container.push(PathCandidate::new(
            origin,
            Angle::new(angle_radian).opposite(),
            property.path_length,
            property.path_priority,
            property.curve,
        ));
        Some(self)
    }

    pub fn iterate_n_times(mut self, n: usize) -> Self {
        for _ in 0..n {
            self = self.iterate();
        }
        self
    }

    fn determine_node_to_apply(
        &self,
        node_from: &TransportNode,
        candidate: &PathCandidate,
    ) -> Option<TransportNode> {
        // If the curve is None, the path will be always extended to straight.
        let curve = candidate.curve.clone().unwrap_or_default();

        candidate
            .angle_to
            .iter_range_around(curve.max_radian, curve.comparison_step)
            .filter_map(|angle| {
                let site_to = candidate
                    .node_from
                    .site
                    .extend(angle, candidate.path_length);
                let node_to = TransportNode::new(site_to, node_from.elevated_height);
                if let Some(property) = self.property_provider.get_property(&node_to.into()) {
                    Some((node_to, property.path_priority))
                } else {
                    None
                }
            })
            .max_by(|(_, path_priority), (_, other_path_priority)| {
                path_priority.total_cmp(other_path_priority)
            })
            .map(|(node_to, _)| node_to)
    }

    fn determine_new_node_type(
        &self,
        node_from: &TransportNode,
        node_to: &TransportNode,
    ) -> NextTransportNodeType {
        let line_segment = LineSegment::new(*node_from, *node_to);
        // search the nearest crossing line segment
        if let Some(crossing_site) = self
            .network
            .search_path_crossing(line_segment)
            .iter()
            .filter(|(crossing_line_segment, _)| {
                crossing_line_segment.0.site != node_from.site
                    && crossing_line_segment.1.site != node_from.site
            })
            .min_by(|(_, site), (_, other_site)| {
                node_from
                    .site
                    .distance_2(site)
                    .total_cmp(&node_from.site.distance_2(other_site))
            })
            .map(|(_, site)| site)
        {
            let node_crossing = TransportNode::new(*crossing_site, node_from.elevated_height);
            NextTransportNodeType::Intersect(node_crossing, line_segment)
        } else {
            NextTransportNodeType::New(*node_to)
        }
    }

    pub fn iterate(mut self) -> Self {
        let prior_candidate = if let Some(candidate) = self.path_candidate_container.pop() {
            candidate
        } else {
            return self;
        };
        let node_from = prior_candidate.node_from;

        // determine node to apply
        let node_type = {
            let node_to =
                if let Some(node_to) = self.determine_node_to_apply(&node_from, &prior_candidate) {
                    node_to
                } else {
                    return self;
                };

            self.determine_new_node_type(&node_from, &node_to)
        };

        match node_type {
            NextTransportNodeType::New(node_to) => {
                self.network.add_path(node_from, node_to);
            }
            NextTransportNodeType::Existing(_) => {}
            NextTransportNodeType::Intersect(node_next, line_segment) => {
                self.network.remove_path(line_segment.0, line_segment.1);
                self.network.add_path(node_from, node_next);
                self.network.add_path(node_next, line_segment.0);
                self.network.add_path(node_next, line_segment.1);
            }
        }

        if let NextTransportNodeType::New(node_next) = node_type {
            if let Some(property) = self.property_provider.get_property(&node_next.into()) {
                // add new path candidates
                let straight_angle = node_from.site.get_angle(&node_next.site);
                self.path_candidate_container.push(PathCandidate::new(
                    node_next,
                    straight_angle,
                    property.path_length,
                    property.path_priority,
                    property.curve.clone(),
                ));
                /*
                self.path_candidate_container.push(PathCandidate::new(
                    node_next,
                    straight_angle.right_clockwise(),
                    property.path_length,
                    property.path_priority,
                    property.curve.clone(),
                ));
                self.path_candidate_container.push(PathCandidate::new(
                    node_next,
                    straight_angle.right_counterclockwise(),
                    property.path_length,
                    property.path_priority,
                    property.curve,
                ));
                */
            }
        }

        self
    }

    pub fn build(self) -> Network<TransportNode> {
        self.network.into_optimized()
    }
}
