use std::collections::BinaryHeap;

use crate::core::{
    container::network::Network,
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
        ));
        self.path_candidate_container.push(PathCandidate::new(
            origin,
            Angle::new(angle_radian).opposite(),
            property.path_length,
            property.path_priority,
        ));
        Some(self)
    }

    pub fn iterate_n_times(mut self, n: usize) -> Self {
        for _ in 0..n {
            self = self.iterate();
        }
        self
    }

    pub fn iterate(mut self) -> Self {
        let prior_candidate = if let Some(candidate) = self.path_candidate_container.pop() {
            candidate
        } else {
            return self;
        };
        let node_from = prior_candidate.node_from();
        println!("node_from: {:?}", node_from);

        // determine node to apply
        let (paths_to_add, paths_to_remove, node_next) = {
            let node_to =
                TransportNode::new(prior_candidate.query_site_to(), node_from.elevated_height);

            let line_segment = LineSegment::new(node_from, node_to);

            // search the nearest crossing line segment
            if let Some((crossing_line_segment, crossing_site)) = self
                .network
                .search_path_crossing(line_segment)
                .iter()
                .filter(|(_, site)| site != &node_from.site)
                .min_by(|(_, site), (_, other_site)| {
                    node_from
                        .site
                        .distance_2(&site)
                        .total_cmp(&node_from.site.distance_2(&other_site))
                })
            {
                // remove the crossing line segment
                let paths_to_remove = vec![**crossing_line_segment];

                // add new line segments
                let node_crossing = TransportNode::new(*crossing_site, node_from.elevated_height);

                let paths_to_add = vec![
                    LineSegment::new(node_crossing, crossing_line_segment.0),
                    LineSegment::new(node_crossing, crossing_line_segment.1),
                    LineSegment::new(node_from, node_crossing),
                ];
                (paths_to_add, paths_to_remove, node_crossing)
            } else {
                // add new line segments
                (vec![line_segment], vec![], node_to)
            }
        };

        // remove paths
        paths_to_remove.iter().for_each(|line_segment| {
            self.network.remove_path(line_segment.0, line_segment.1);
        });

        // add paths
        paths_to_add.iter().for_each(|line_segment| {
            self.network.add_path(line_segment.0, line_segment.1);
        });

        let straight_angle = node_from.site.get_angle(&node_next.site);
        if let Some(property) = self.property_provider.get_property(&node_next.into()) {
            // add new path candidates
            self.path_candidate_container.push(PathCandidate::new(
                node_next,
                straight_angle,
                property.path_length,
                property.path_priority,
            ));
        }

        self
    }

    pub fn build(self) -> Network<TransportNode> {
        self.network.into_optimized()
    }
}
