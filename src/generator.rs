use std::collections::BinaryHeap;

use rand::{Rng, SeedableRng};

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
    pub road_evaluation: fn(f64, f64) -> f64,
    pub road_branch_probability_by_evaluation: fn(f64) -> f64,
}

pub struct NetworkGenerator<'a, EM, PM>
where
    EM: ElevationModel,
    PM: PopulationDensityModel,
{
    rng: rand::rngs::StdRng,
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
        seed: u64,
        elevation_model: &'a EM,
        population_density_model: &'a PM,
        config: NetworkConfig,
    ) -> Self {
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            network: Network::new(),
            elevation_model,
            population_density_model,
            node_open: BinaryHeap::new(),
            config,
        }
    }

    fn evaluate_site(&self, site: Site2D) -> Option<f64> {
        let population_density = self.population_density_model.get_population_density(site)?;
        let elevation = self.elevation_model.get_elevation(site)?;
        Some((self.config.road_evaluation)(population_density, elevation))
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
        // choose next node from open list
        let next = {
            let candidate = self.node_open.pop();
            if let Some(candidate) = candidate {
                candidate
            } else {
                return self;
            }
        };

        // add next node to network
        let next_id = self.network.add_new_node(next.node);
        // connect next node to parent
        self.network.connect_nodes(next.parent_id, next_id);

        // add straight node to open list
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

        let branch_probability =
            (self.config.road_branch_probability_by_evaluation)(next.evaluation);

        let branch_left = branch_probability > self.rng.gen::<f64>();
        let branch_right = branch_probability > self.rng.gen::<f64>();

        // add left node to open list
        if branch_left {
            if let Some((left_node, left_evaluation)) = self.search_next_node_position(
                next.node.site,
                next.node.angle - std::f64::consts::PI / 2.0,
            ) {
                let candidate_left = NodeCandidate {
                    node: left_node,
                    parent_id: next_id,
                    evaluation: left_evaluation,
                };
                self.node_open.push(candidate_left);
            }
        }

        // add right node to open list
        if branch_right {
            if let Some((right_node, right_evaluation)) = self.search_next_node_position(
                next.node.site,
                next.node.angle + std::f64::consts::PI / 2.0,
            ) {
                let candidate_right = NodeCandidate {
                    node: right_node,
                    parent_id: next_id,
                    evaluation: right_evaluation,
                };
                self.node_open.push(candidate_right);
            }
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
