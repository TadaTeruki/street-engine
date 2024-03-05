use std::collections::BinaryHeap;

use crate::geom::Site2D;
use crate::interface::{ElevationModel, PopulationDensityModel};
use crate::model::{Network, Node};

struct NodeCandidate {
    pub node: Node,
    pub parent_id: usize,
    pub evaluation: f64,
}

impl PartialEq for NodeCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.evaluation == other.evaluation
    }
}

impl Eq for NodeCandidate {}

impl PartialOrd for NodeCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.evaluation > other.evaluation {
            std::cmp::Ordering::Greater
        } else if self.evaluation < other.evaluation {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

pub struct NetworkConfig {
    pub max_straight_angle: f64,
    pub road_length: f64,
    pub road_comparison_step: usize,
}

pub struct NetworkGenerator<'a, EM, PM>
where
    EM: ElevationModel,
    PM: PopulationDensityModel,
{
    network: Network,
    elevation_model: &'a EM,
    population_density_model: &'a PM,
    node_open: BinaryHeap<NodeCandidate>,
    config: NetworkConfig,
}

impl<'a, EM, PM> NetworkGenerator<'a, EM, PM>
where
    EM: ElevationModel,
    PM: PopulationDensityModel,
{
    pub fn new(
        elevation_model: &'a EM,
        population_density_model: &'a PM,
        config: NetworkConfig,
    ) -> Self {
        Self {
            network: Network::new(),
            elevation_model,
            population_density_model,
            node_open: BinaryHeap::new(),
            config,
        }
    }

    fn evaluate_site(&self, site: Site2D) -> Option<f64> {
        let population_density = self.population_density_model.get_population_density(site)?;
        Some(population_density)
    }

    pub fn snapshot(&self) -> &Network {
        &self.network
    }

    pub fn add_origin(mut self, site: Site2D, angle: f64) -> Self {
        let origin = Node { site, angle: 0.0 };
        let origin_id = self.network.add_new_node(origin);

        if let Some((straight_node, straight_evaluation)) =
            self.search_next_node_position(site, angle)
        {
            let candidate_straight = NodeCandidate {
                node: straight_node,
                parent_id: origin_id,
                evaluation: straight_evaluation,
            };
            self.node_open.push(candidate_straight);
        }

        if let Some((opposite_node, opposite_evaluation)) =
            self.search_next_node_position(site, angle + std::f64::consts::PI)
        {
            let candidate_opposite = NodeCandidate {
                node: opposite_node,
                parent_id: origin_id,
                evaluation: opposite_evaluation,
            };
            self.node_open.push(candidate_opposite);
        }
        self
    }

    fn search_next_node_position(&self, site: Site2D, angle: f64) -> Option<(Node, f64)> {
        let mut next_node = None;
        let mut next_evaluation = None;
        for i in 0..self.config.road_comparison_step {
            let comp_angle = angle
                + ((i as f64 / self.config.road_comparison_step as f64) - 0.5)
                    * self.config.max_straight_angle;
            let comp_site = Site2D {
                x: site.x + self.config.road_length * comp_angle.cos(),
                y: site.y + self.config.road_length * comp_angle.sin(),
            };
            let comp_evaluation = if let Some(evaluation) = self.evaluate_site(comp_site) {
                evaluation
            } else {
                continue;
            };
            if next_evaluation.is_none() || comp_evaluation > next_evaluation.unwrap() {
                next_node = Some(Node {
                    site: comp_site,
                    angle: comp_angle,
                });
                next_evaluation = Some(comp_evaluation);
            }
        }
        if let (Some(next_node), Some(next_evaluation)) = (next_node, next_evaluation) {
            Some((next_node, next_evaluation))
        } else {
            None
        }
    }

    pub fn iterate(mut self) -> Self {
        let next = {
            let candidate = self.node_open.pop();
            if let Some(candidate) = candidate {
                candidate
            } else {
                return self;
            }
        };

        let next_id = self.network.add_new_node(next.node);

        self.network.connect_nodes(next.parent_id, next_id);

        if let Some((straight_node, straight_evaluation)) =
            self.search_next_node_position(next.node.site, next.node.angle)
        {
            let candidate_straight = NodeCandidate {
                node: straight_node,
                parent_id: next_id,
                evaluation: straight_evaluation,
            };
            self.node_open.push(candidate_straight);
        }

        self
    }

    pub fn iterate_n_times(mut self, n: usize) -> Self {
        for _ in 0..n {
            self = self.iterate();
        }
        self
    }
}
