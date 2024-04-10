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

        let site_start = prior_candidate.get_site_start();
        let site_expected_end = if let Some(site_expected_end) = {
            let curve = prior_candidate
                .get_property()
                .curve
                .clone()
                .unwrap_or_default();
            prior_candidate
                .angle_expected_end()
                .iter_range_around(curve.max_radian, curve.comparison_step)
                .map(|angle| {
                    site_start.extend(angle, prior_candidate.get_property().path_normal_length)
                })
                .filter_map(|site| Some((site, self.property_provider.get_property(&site)?)))
                .max_by(|(_, property1), (_, property2)| {
                    property1.path_priority.total_cmp(&property2.path_priority)
                })
                .map(|(site, _)| site)
        } {
            site_expected_end
        } else {
            return self;
        };

        println!("{:?} -> {:?}", site_start, site_expected_end);
        println!("except: {:?}", prior_candidate.get_node_start_id());

        let related_nodes = self
            .path_network
            .nodes_around_line_iter(
                LineSegment::new(site_start, site_expected_end),
                prior_candidate
                    .get_property()
                    .path_extra_length_for_intersection,
            )
            .filter(|&node_id| *node_id != prior_candidate.get_node_start_id())
            .filter_map(|node_id| Some((self.path_network.get_node(*node_id)?, *node_id)))
            .collect::<Vec<_>>();

        let related_paths = self
            .path_network
            .paths_touching_rect_iter(site_start, site_expected_end)
            .filter(|(node_id_start, node_id_end)| {
                *node_id_start != prior_candidate.get_node_start_id()
                    && *node_id_end != prior_candidate.get_node_start_id()
            })
            .filter_map(|(node_id_start, node_id_end)| {
                let node_start = self.path_network.get_node(*node_id_start)?;
                let node_end = self.path_network.get_node(*node_id_end)?;
                Some(((node_start, *node_id_start), (node_end, *node_id_end)))
            })
            .collect::<Vec<_>>();

        let candidate_node_id = prior_candidate.get_node_start_id();
        let next_node =
            prior_candidate.determine_next_node(site_expected_end, &related_nodes, &related_paths);

        println!("{:?}", next_node);

        match next_node {
            NextTransportNode::New(node_end) => {
                let node_id = self.path_network.add_node(node_end);
                self.path_network.add_path(candidate_node_id, node_id);

                // add new path candidates
                if let Some(property) = self.property_provider.get_property(&node_end.into()) {
                    let straight_angle = site_start.get_angle(&site_expected_end);
                    self.path_candidate_container.push(PathCandidate::new(
                        node_end,
                        node_id,
                        straight_angle,
                        property.clone(),
                    ));

                    self.path_candidate_container.push(PathCandidate::new(
                        node_end,
                        node_id,
                        straight_angle.right_clockwise(),
                        property.clone(),
                    ));

                    self.path_candidate_container.push(PathCandidate::new(
                        node_end,
                        node_id,
                        straight_angle.right_counterclockwise(),
                        property.clone(),
                    ));
                }
            }
            NextTransportNode::Existing(node_id) => {
                self.path_network.add_path(candidate_node_id, node_id);
            }
            NextTransportNode::Intersect(node_end, encount_path) => {
                let next_node_id = self.path_network.add_node(node_end);
                self.path_network
                    .remove_path(encount_path.0, encount_path.1);
                self.path_network.add_path(candidate_node_id, next_node_id);
                self.path_network.add_path(next_node_id, encount_path.0);
                self.path_network.add_path(next_node_id, encount_path.1);
            }
        }

        self
    }

    pub fn build(self) -> PathNetwork<TransportNode> {
        self.path_network.into_optimized()
    }
}
