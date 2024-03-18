use std::collections::BinaryHeap;

use rand::{Rng, SeedableRng};

use crate::geom::Site2D;
use crate::interface::{ElevationModel, PopulationDensityModel};
use crate::model::{Network, Node};

enum CandidateNodeType {
    New(Node),
    Existing(usize),
    Branch(Node, usize, usize),
}

struct NodeCandidate {
    pub node: CandidateNodeType,
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
    pub merge_node_distance: f64,
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
        if elevation < 1e-2 {
            return None;
        }
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

    fn search_next_node_position(
        &self,
        site: Site2D,
        angle: f64,
    ) -> Option<(CandidateNodeType, f64)> {
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
            // if there are existing node, return existing node
            if let Some(id) = self
                .network
                .get_nearest_node_in_distance(next_node.site, self.config.merge_node_distance)
            {
                return Some((CandidateNodeType::Existing(id), next_evaluation));
            }

            // if there are crossing connection, return branch node
            if let Some(crossing) = self.network.get_crossing_connection(
                site,
                next_node.site,
                self.config.merge_node_distance,
                self.config.road_length,
            ) {
                let (crossing_site, id1, id2) = crossing;
                //let angle = (crossing_site.y - site.y).atan2(crossing_site.x - site.x);
                return Some((
                    CandidateNodeType::Branch(
                        Node {
                            site: crossing_site,
                            angle,
                        },
                        id1,
                        id2,
                    ),
                    next_evaluation,
                ));
            }

            Some((CandidateNodeType::New(next_node), next_evaluation))
        } else {
            None
        }
    }

    fn apply_new_candidate_node(&mut self, candidate: NodeCandidate) {
        match candidate.node {
            CandidateNodeType::New(node) => {
                let id = self.network.add_new_node(node);
                self.network.connect_nodes(candidate.parent_id, id);

                // add straight node to open list
                if let Some((straight_node, straight_evaluation)) =
                    self.search_next_node_position(node.site, node.angle)
                {
                    let candidate_straight = NodeCandidate {
                        node: straight_node,
                        parent_id: id,
                        evaluation: straight_evaluation,
                    };
                    self.node_open.push(candidate_straight);
                }

                let branch_probability =
                    (self.config.road_branch_probability_by_evaluation)(candidate.evaluation);

                let branch_left = branch_probability > self.rng.gen::<f64>();
                let branch_right = branch_probability > self.rng.gen::<f64>();

                // add left node to open list
                if branch_left {
                    if let Some((left_node, left_evaluation)) = self.search_next_node_position(
                        node.site,
                        node.angle - std::f64::consts::PI / 2.0,
                    ) {
                        let candidate_left = NodeCandidate {
                            node: left_node,
                            parent_id: id,
                            evaluation: left_evaluation,
                        };
                        self.node_open.push(candidate_left);
                    }
                }

                // add right node to open list
                if branch_right {
                    if let Some((right_node, right_evaluation)) = self.search_next_node_position(
                        node.site,
                        node.angle + std::f64::consts::PI / 2.0,
                    ) {
                        let candidate_right = NodeCandidate {
                            node: right_node,
                            parent_id: id,
                            evaluation: right_evaluation,
                        };
                        self.node_open.push(candidate_right);
                    }
                }
            }
            CandidateNodeType::Existing(id) => {
                self.network.connect_nodes(candidate.parent_id, id);
            }
            CandidateNodeType::Branch(node, id1, id2) => {
                let id = self.network.add_new_node(node);
                self.network.connect_nodes(candidate.parent_id, id);
                self.network.connect_nodes(id, id1);
                self.network.connect_nodes(id, id2);
                self.network.remove_connection(id1, id2);
            }
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

        let next_node_site = match next.node {
            CandidateNodeType::New(node) => node.site,
            CandidateNodeType::Existing(id) => {
                if let Some(node) = self.network.get_node(id) {
                    node.site
                } else {
                    return self;
                }
            }
            CandidateNodeType::Branch(node, _, _) => node.site,
        };

        let next_node_angle = match next.node {
            CandidateNodeType::New(node) => node.angle,
            CandidateNodeType::Existing(id) => {
                if let Some(node) = self.network.get_node(id) {
                    node.angle
                } else {
                    return self;
                }
            }
            CandidateNodeType::Branch(node, _, _) => node.angle,
        };

        let next = if let Some((new_next, new_evaluation)) =
            self.search_next_node_position(next_node_site, next_node_angle)
        {
            NodeCandidate {
                node: new_next,
                parent_id: next.parent_id,
                evaluation: new_evaluation,
            }
        } else {
            return self;
        };

        // apply
        self.apply_new_candidate_node(next);

        self
    }

    pub fn iterate_n_times(mut self, n: usize) -> Self {
        for _ in 0..n {
            self = self.iterate();
        }
        self
    }
}
