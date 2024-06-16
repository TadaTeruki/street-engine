use std::collections::BTreeMap;

use rstar::RTree;

use crate::core::{
    generator::id_generator::IdGenerator,
    geometry::{line_segment::LineSegment, site::Site},
};

use super::{
    index_object::{NodeTreeObject, PathTreeObject},
    undirected::UndirectedGraph,
};

/// Trait for a node in the path network.
pub trait PathNetworkNodeTrait: Eq + Clone {
    fn get_site(&self) -> Site;
}

/// ID for identifying a node in the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// Path network represents a network of nodes and paths.
/// This struct is used to manage nodes and paths between nodes in 2D space.
///
/// This struct provides:
///  - functions to add, remove, and search nodes and paths.
///  - functions to search nodes around a site or a line segment.
#[derive(Debug, Clone)]
pub struct PathNetwork<N>
where
    N: PathNetworkNodeTrait,
{
    nodes: BTreeMap<NodeId, N>,
    path_tree: RTree<PathTreeObject<NodeId>>,
    node_tree: RTree<NodeTreeObject<NodeId>>,
    path_connection: UndirectedGraph<NodeId>,
    id_generator: IdGenerator,
}

impl<N> Default for PathNetwork<N>
where
    N: PathNetworkNodeTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N> PathNetwork<N>
where
    N: PathNetworkNodeTrait,
{
    /// Create a new path network.
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            path_tree: RTree::new(),
            node_tree: RTree::new(),
            path_connection: UndirectedGraph::new(),
            id_generator: IdGenerator::new(),
        }
    }

    /// Create a new path network from lists of nodes and paths.
    ///
    /// This function always returns an optimized path network as a result
    /// because all nodes and paths are bulk added to the network.
    pub fn from(nodes: Vec<N>, paths: &[(usize, usize)]) -> Option<Self> {
        let mut id_generator: IdGenerator = IdGenerator::new();

        // distribute NodeIds to nodes
        let nodes = nodes
            .into_iter()
            .map(|node| (NodeId(id_generator.generate_id()), node))
            .collect::<Vec<_>>();

        // original paths length
        let paths_len = paths.len();

        // convert paths from usize to NodeId
        let paths = paths
            .iter()
            .filter_map(|(start, end)| Some((nodes.get(*start)?.0, nodes.get(*end)?.0)))
            .collect::<Vec<_>>();
        // if there are invalid paths, return None
        if paths.len() != paths_len {
            return None;
        }

        // rtree for nodes
        let node_tree = RTree::bulk_load(
            nodes
                .iter()
                .map(|(node_id, node)| NodeTreeObject::new(node.get_site(), *node_id))
                .collect::<Vec<_>>(),
        );

        let path_connection = paths.iter().fold(
            UndirectedGraph::new(),
            |mut path_connection, (start, end)| {
                path_connection.add_edge(*start, *end);
                path_connection
            },
        );

        let nodes = nodes.into_iter().collect::<BTreeMap<_, _>>();

        let path_tree = RTree::bulk_load(
            paths
                .iter()
                .filter_map(|(start, end)| {
                    let (start_site, end_site) =
                        (nodes.get(start)?.get_site(), nodes.get(end)?.get_site());
                    Some(PathTreeObject::new(
                        LineSegment::new(start_site, end_site),
                        (*start, *end),
                    ))
                })
                .collect::<Vec<_>>(),
        );
        // if there are invalid paths, return None
        if path_tree.size() != paths_len {
            return None;
        }
        Some(Self {
            nodes,
            path_tree,
            node_tree,
            path_connection,
            id_generator,
        })
    }

    /// Get nodes in the network.
    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeId, &N)> {
        self.nodes.iter().map(|(node_id, node)| (*node_id, node))
    }

    /// Get the pairs of connected nodes in the network.
    fn paths_iter(&self) -> impl Iterator<Item = (NodeId, NodeId)> + '_ {
        self.path_connection.edges_iter()
    }

    /// Get neighbors of a node.
    pub fn neighbors_iter(&self, node_id: NodeId) -> Option<impl Iterator<Item = (NodeId, &N)>> {
        self.path_connection
            .neighbors_iter(node_id)
            .map(|neighbors| {
                neighbors.filter_map(move |neighbor| Some((*neighbor, self.nodes.get(neighbor)?)))
            })
    }

    /// Generate a new NodeId.
    /// This checks if the generated NodeId is unique in the network.
    fn new_node_id(&mut self) -> NodeId {
        loop {
            let id = self.id_generator.generate_id();
            if !self.nodes.contains_key(&NodeId(id)) {
                break NodeId(id);
            }
        }
    }

    /// Add a node to the network.
    pub fn add_node(&mut self, node: N) -> NodeId {
        let node_id = self.new_node_id();
        let site = node.get_site();
        self.nodes.insert(node_id, node);
        self.node_tree.insert(NodeTreeObject::new(site, node_id));
        node_id
    }

    /// Modify a node in the network.
    pub fn modify_node<T>(&mut self, node_id: NodeId, f: impl FnOnce(&mut N) -> T) -> Option<T> {
        self.nodes.get_mut(&node_id).map(f)
    }

    /// Remove a node from the network.
    /// This function can be never used, but it is kept for future use.
    #[allow(dead_code)]
    fn remove_node(&mut self, node_id: NodeId) -> Option<NodeId> {
        let neighbors = if let Some(neighbors) = self.path_connection.neighbors_iter(node_id) {
            neighbors.copied().collect::<Vec<_>>()
        } else {
            return None;
        };

        let site = if let Some(node) = self.nodes.get(&node_id) {
            (*node).get_site()
        } else {
            return None;
        };

        neighbors.iter().for_each(|neighbor| {
            self.remove_path(node_id, *neighbor);
        });

        self.node_tree.remove(&NodeTreeObject::new(site, node_id));

        self.nodes.remove(&node_id);
        Some(node_id)
    }

    /// Add a path to the network.
    pub(crate) fn add_path(&mut self, start: NodeId, end: NodeId) -> Option<(NodeId, NodeId)> {
        if start == end {
            return None;
        }
        if self.path_connection.has_edge(start, end) {
            return None;
        }

        if let (Some(start_node), Some(end_node)) = (self.nodes.get(&start), self.nodes.get(&end)) {
            self.path_connection.add_edge(start, end);

            let (start_site, end_site) = ((*start_node).get_site(), (*end_node).get_site());

            self.path_tree.insert(PathTreeObject::new(
                LineSegment::new(start_site, end_site),
                (start, end),
            ));

            Some((start, end))
        } else {
            None
        }
    }

    /// Remove a path from the network.
    pub(crate) fn remove_path(&mut self, start: NodeId, end: NodeId) -> Option<(NodeId, NodeId)> {
        let (start_site, end_site) = if let (Some(start_node), Some(end_node)) =
            (self.nodes.get(&start), self.nodes.get(&end))
        {
            (start_node.get_site(), end_node.get_site())
        } else {
            return None;
        };

        self.path_connection.remove_edge(start, end);

        self.path_tree.remove(&PathTreeObject::new(
            LineSegment::new(start_site, end_site),
            (start, end),
        ));

        Some((start, end))
    }

    /// Get a node by its NodeId.
    pub fn get_node(&self, node_id: NodeId) -> Option<&N> {
        self.nodes.get(&node_id)
    }

    /// Check if there is a path between two nodes.
    pub fn has_path(&self, start: NodeId, to: NodeId) -> bool {
        self.path_connection.has_edge(start, to)
    }

    /// Search nodes around a site within a radius.
    pub fn nodes_around_site_iter(&self, site: Site, radius: f64) -> impl Iterator<Item = &NodeId> {
        let envelope = rstar::AABB::from_corners(
            [site.x - radius, site.y - radius],
            [site.x + radius, site.y + radius],
        );
        self.node_tree
            .locate_in_envelope(&envelope)
            .filter(move |object| site.distance(object.site()) <= radius)
            .map(|object| object.node_id())
    }

    /// Search the nearest node from a site.
    pub fn search_nearest_node(&self, site: Site) -> Option<NodeId> {
        self.node_tree
            .nearest_neighbor(&[site.x, site.y])
            .map(|object| *object.node_id())
    }

    /// Search nodes around a line segment within a radius.
    pub fn nodes_around_line_iter(
        &self,
        line: LineSegment,
        radius: f64,
    ) -> impl Iterator<Item = &NodeId> {
        let envelope = rstar::AABB::from_corners(
            [
                line.0.x.min(line.1.x) - radius,
                line.0.y.min(line.1.y) - radius,
            ],
            [
                line.0.x.max(line.1.x) + radius,
                line.0.y.max(line.1.y) + radius,
            ],
        );
        self.node_tree
            .locate_in_envelope(&envelope)
            .filter(move |object: &&NodeTreeObject<NodeId>| {
                line.get_distance(object.site()) <= radius
            })
            .map(|object| object.node_id())
    }

    /// Search paths touching a rectangle.
    pub fn paths_touching_rect_iter(
        &self,
        corner_0: Site,
        corner_1: Site,
    ) -> impl Iterator<Item = &(NodeId, NodeId)> {
        let search_rect =
            rstar::AABB::from_corners([corner_0.x, corner_0.y], [corner_1.x, corner_1.y]);

        self.path_tree
            .locate_in_envelope_intersecting(&search_rect)
            .map(|object| object.node_ids())
    }

    /// Parse the network into a list of nodes and paths.
    ///
    /// This function is not exposed now, but it may be useful in the future.
    fn parse(&self) -> (Vec<N>, Vec<(usize, usize)>) {
        let nodes = self.nodes.values().collect::<Vec<_>>();

        // temporary data structure to convert NodeId to usize
        let node_id_to_index = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, (node_id, _))| (*node_id, index))
            .collect::<BTreeMap<_, _>>();

        let paths = self
            .paths_iter()
            .filter_map(|(start, end)| {
                Some((*node_id_to_index.get(&start)?, *node_id_to_index.get(&end)?))
            })
            .collect::<Vec<_>>();

        let nodes = nodes.into_iter().cloned().collect::<Vec<_>>();

        (nodes, paths)
    }

    /// Reconstruct the network.
    ///
    /// The structure of the network is optimized by bulk adding all nodes and paths.
    pub fn reconstruct(self) -> Option<Self> {
        let (nodes, paths) = self.parse();
        Self::from(nodes, &paths)
    }

    /// This function is only for testing
    #[allow(dead_code)]
    fn check_path_state_is_consistent(&self) -> bool {
        self.path_tree.size() == self.path_connection.size()
            && self.nodes.len() == self.node_tree.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Node for testing the path network.
    /// This struct is used only for testing.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MockNode {
        site: Site,
    }

    impl MockNode {
        fn new(x: f64, y: f64) -> Self {
            Self {
                site: Site::new(x, y),
            }
        }
    }

    impl PathNetworkNodeTrait for MockNode {
        fn get_site(&self) -> Site {
            self.site
        }
    }

    #[test]
    fn test_path_network() {
        let mut network = PathNetwork::new();
        let node0 = network.add_node(MockNode::new(0.0, 0.0));
        let node1 = network.add_node(MockNode::new(1.0, 1.0));
        let node2 = network.add_node(MockNode::new(2.0, 2.0));
        let node3 = network.add_node(MockNode::new(3.0, 3.0));
        let node4 = network.add_node(MockNode::new(1.0, 4.0));

        network.add_path(node0, node1);
        network.add_path(node1, node2);
        network.add_path(node2, node3);
        network.add_path(node3, node4);
        network.add_path(node4, node2);

        assert!(network.has_path(node0, node1));
        assert!(network.has_path(node1, node2));
        assert!(network.has_path(node2, node3));
        assert!(network.has_path(node3, node4));
        assert!(!network.has_path(node0, node2));

        assert!(network.check_path_state_is_consistent());

        network.remove_path(node1, node2);
        assert!(!network.has_path(node1, node2));
        assert!(network.has_path(node2, node3));

        assert!(network.check_path_state_is_consistent());

        network.remove_node(node1);
        assert!(!network.has_path(node0, node1));

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_path_crossing_no_crosses() {
        let mut network = PathNetwork::new();
        let node0 = network.add_node(MockNode::new(0.0, 1.0));
        let node1 = network.add_node(MockNode::new(2.0, 3.0));
        let node2 = network.add_node(MockNode::new(4.0, 5.0));

        network.add_path(node0, node1);
        network.add_path(node1, node2);

        let paths = network
            .paths_touching_rect_iter(Site::new(0.0, 0.0), Site::new(1.0, 1.0))
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 1);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_path_crossing_all_cross() {
        let mut network = PathNetwork::new();

        let nodes = vec![
            MockNode::new(0.0, 2.0),
            MockNode::new(2.0, 2.0),
            MockNode::new(2.0, 0.0),
            MockNode::new(0.0, 0.0),
        ];

        let nodes = nodes
            .iter()
            .map(|site| network.add_node(*site))
            .collect::<Vec<_>>();

        for i in 0..nodes.len() {
            // Add all paths between nodes
            // When i == j, the path is expected to be ignored
            for j in i..nodes.len() {
                network.add_path(nodes[i], nodes[j]);
            }
        }

        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i != j {
                    assert!(network.has_path(NodeId(i), NodeId(j)));
                }
            }
        }

        let paths = network
            .paths_touching_rect_iter(Site::new(0.0, 0.0), Site::new(1.0, 2.0))
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 5);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_nodes_around_site() {
        let mut network = PathNetwork::new();
        let node0 = network.add_node(MockNode::new(0.0, 0.0));
        let node1 = network.add_node(MockNode::new(1.0, 1.0));
        let node2 = network.add_node(MockNode::new(2.0, 2.0));
        let node3 = network.add_node(MockNode::new(3.0, 3.0));
        let node4 = network.add_node(MockNode::new(1.0, 4.0));

        network.add_path(node0, node1);
        network.add_path(node1, node2);
        network.add_path(node2, node3);
        network.add_path(node3, node4);
        network.add_path(node4, node2);

        let site = Site::new(1.0, 1.0);
        let nodes = network
            .nodes_around_site_iter(site, 1.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);

        let site = Site::new(2.0, 1.0);
        let nodes = network
            .nodes_around_site_iter(site, 2.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 2);

        let site = Site::new(2.0, 3.0);
        let nodes = network
            .nodes_around_site_iter(site, 2.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        let line = LineSegment::new(Site::new(1.0, 3.0), Site::new(3.0, 2.0));
        let nodes = network
            .nodes_around_line_iter(line, 1.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        let line = LineSegment::new(Site::new(1.0, 0.0), Site::new(0.0, 1.0));
        let nodes = network
            .nodes_around_line_iter(line, 2.5)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        network.remove_path(node3, node4);
        network.remove_node(node1);

        let site = Site::new(2.0, 1.0);
        let nodes = network
            .nodes_around_site_iter(site, 2.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);

        let line = LineSegment::new(Site::new(1.0, 0.0), Site::new(0.0, 1.0));
        let nodes = network
            .nodes_around_line_iter(line, 2.5)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 2);

        assert!(network.check_path_state_is_consistent());
    }

    fn xorshift(x: usize) -> usize {
        let mut x = x;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    }

    #[test]
    fn test_construction() {
        let nodes = (0..300)
            .map(|i| MockNode::new(xorshift(i * 2) as f64, xorshift(i * 2 + 1) as f64))
            .collect::<Vec<_>>();

        let paths = {
            let mut paths = Vec::new();
            for i in 0..nodes.len() {
                for j in i + 1..nodes.len() {
                    if xorshift(i * nodes.len() + j) % 2 == 0 {
                        paths.push((i, j));
                    }
                }
            }
            paths
        };

        let mut network0 = PathNetwork::new();
        let nodeids0 = nodes
            .iter()
            .map(|site| network0.add_node(*site))
            .collect::<Vec<_>>();

        for (start, end) in paths.iter() {
            network0.add_path(nodeids0[*start], nodeids0[*end]);
        }

        let network1 = PathNetwork::from(nodes.clone(), &paths).unwrap();
        let nodeids1 = nodes
            .iter()
            .map(|node| network1.search_nearest_node(node.get_site()).unwrap())
            .collect::<Vec<_>>();

        let network2 = network0.clone().reconstruct().unwrap();
        let nodeids2 = nodes
            .iter()
            .map(|node| network2.search_nearest_node(node.get_site()).unwrap())
            .collect::<Vec<_>>();

        let network3 = network1.clone().reconstruct().unwrap();
        let nodeids3 = nodes
            .iter()
            .map(|node| network3.search_nearest_node(node.get_site()).unwrap())
            .collect::<Vec<_>>();

        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                let r0 = network0.has_path(nodeids0[i], nodeids0[j]);
                let r1 = network1.has_path(nodeids1[i], nodeids1[j]);
                assert_eq!(r0, r1);

                let r2 = network2.has_path(nodeids2[i], nodeids2[j]);
                assert_eq!(r1, r2);

                let r3 = network3.has_path(nodeids3[i], nodeids3[j]);
                assert_eq!(r2, r3);
            }
        }
    }

    #[test]
    fn test_complex_network() {
        let nodes = (0..50)
            .map(|i| MockNode::new(xorshift(i * 2) as f64, xorshift(i * 2 + 1) as f64))
            .collect::<Vec<_>>();

        let loop_count = 10;

        let mut network = PathNetwork::new();

        let nodeids = nodes
            .iter()
            .map(|site| network.add_node(*site))
            .collect::<Vec<_>>();

        for l in 0..loop_count {
            let seed_start = l * nodes.len() * nodes.len();
            (0..nodes.len()).for_each(|i| {
                (0..nodes.len()).for_each(|j| {
                    let id = i * nodes.len() + j;
                    if xorshift(id + seed_start) % 2 == 0 {
                        network.add_path(nodeids[i], nodeids[j]);
                    }
                });
            });

            assert!(network.check_path_state_is_consistent());

            (0..nodes.len()).for_each(|i| {
                (0..nodes.len()).for_each(|j| {
                    let id = i * nodes.len() + j;
                    if xorshift(id + seed_start) % 3 == 0 {
                        network.remove_path(nodeids[i], nodeids[j]);
                    }
                });
            });

            assert!(network.check_path_state_is_consistent());
        }
    }
}
