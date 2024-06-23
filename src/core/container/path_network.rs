use std::collections::BTreeMap;

use rstar::RTree;

use crate::core::{generator::id_generator::IdGenerator, geometry::site::Site};

use super::{
    graph::UeamGraph,
    index_object::{NodeTreeObject, PathTreeObject},
};

/// Trait for a node in the path network.
pub trait NodeTrait: Eq + Clone {
    fn get_site(&self) -> Site;
}

/// Trait for paths.
pub trait PathTrait {
    /// Handle for the path. This is the identifier of the path.
    type Handle: Copy + Eq;

    /// Create a new path with start, end sites.
    fn new(start: Site, end: Site, handle: Self::Handle) -> Self;

    /// Get the handle of the path.
    fn get_handle(&self) -> Self::Handle;

    /// Calculate the intersection of two paths or return None.
    fn get_intersections(&self, other: &Self) -> Vec<Site>;

    /// Calculate the perpendicular projection of the site on the path.
    fn get_projection(&self, site: &Site) -> Option<Site>;

    /// Calculate the distance from the site to the path.
    fn get_distance(&self, site: &Site) -> f64;

    /// Calculate the bounds of the path and return the corner (min, max) sites.
    fn get_bounds(&self) -> (Site, Site);
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
pub struct PathNetwork<N, P, PA>
where
    N: NodeTrait,
    P: PathTrait,
    PA: Clone,
{
    nodes: BTreeMap<NodeId, N>,
    path_tree: RTree<PathTreeObject<NodeId, P>>,
    node_tree: RTree<NodeTreeObject<NodeId>>,
    path_connection: UeamGraph<NodeId, P::Handle, PA>,
    id_generator: IdGenerator,
}

impl<N, P, PA> Default for PathNetwork<N, P, PA>
where
    N: NodeTrait,
    P: PathTrait,
    PA: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, P, PA> PathNetwork<N, P, PA>
where
    N: NodeTrait,
    P: PathTrait,
    PA: Clone,
{
    /// Create a new path network.
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            path_tree: RTree::new(),
            node_tree: RTree::new(),
            path_connection: UeamGraph::new(),
            id_generator: IdGenerator::new(),
        }
    }

    /// Create a new path network from lists of nodes and paths.
    ///
    /// This function always returns an optimized path network as a result
    /// because all nodes and paths are bulk added to the network.
    pub fn from(nodes: Vec<N>, paths: &[(usize, usize, P::Handle, PA)]) -> Option<Self> {
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
            .filter_map(|(start, end, handle, attr)| {
                Some((
                    nodes.get(*start)?.0,
                    nodes.get(*end)?.0,
                    *handle,
                    attr.clone(),
                ))
            })
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
            UeamGraph::new(),
            |mut path_connection, (start, end, handle, attr)| {
                path_connection.add_edge(*start, *end, *handle, attr.clone());
                path_connection
            },
        );

        let nodes = nodes.into_iter().collect::<BTreeMap<_, _>>();

        let path_tree = RTree::bulk_load(
            paths
                .iter()
                .filter_map(|(start, end, handle, _)| {
                    let (start_site, end_site) =
                        (nodes.get(start)?.get_site(), nodes.get(end)?.get_site());
                    Some(PathTreeObject::new(
                        P::new(start_site, end_site, *handle),
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
    pub fn neighbors_iter(&self, node_id: NodeId) -> impl Iterator<Item = (NodeId, &N)> + '_ {
        self.path_connection
            .neighbors_iter(node_id)
            .filter_map(move |neighbors| {
                //neighbors.filter_map(move |neighbor| Some((*neighbor, self.nodes.get(neighbor)?)))
                Some((*neighbors, self.nodes.get(&node_id)?))
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
        let neighbors = self
            .path_connection
            .neighbors_iter(node_id)
            .copied()
            .collect::<Vec<_>>();

        let site = if let Some(node) = self.nodes.get(&node_id) {
            (*node).get_site()
        } else {
            return None;
        };

        neighbors.iter().for_each(|neighbor| {
            self.remove_connection(node_id, *neighbor);
        });

        self.node_tree.remove(&NodeTreeObject::new(site, node_id));

        self.nodes.remove(&node_id);
        Some(node_id)
    }

    /// Add a path to the network.
    ///
    /// If the start and end nodes are the same, this function returns None.
    pub(crate) fn add_path(
        &mut self,
        start: NodeId,
        end: NodeId,
        handle: P::Handle,
        attr: PA,
    ) -> Option<(NodeId, NodeId, P::Handle, PA)> {
        if start == end {
            return None;
        }
        if self.path_connection.has_edge(start, end, &handle).is_some() {
            return None;
        }

        if let (Some(start_node), Some(end_node)) = (self.nodes.get(&start), self.nodes.get(&end)) {
            self.path_connection
                .add_edge(start, end, handle.clone(), attr.clone());

            let (start_site, end_site) = ((*start_node).get_site(), (*end_node).get_site());

            self.path_tree.insert(PathTreeObject::new(
                P::new(start_site, end_site, handle.clone()),
                (start, end),
            ));

            Some((start, end, handle, attr))
        } else {
            None
        }
    }

    /// Remove a path from the network.
    pub(crate) fn remove_path(
        &mut self,
        start: NodeId,
        end: NodeId,
        handle: P::Handle,
    ) -> Option<(NodeId, NodeId)> {
        let (start_site, end_site) = if let (Some(start_node), Some(end_node)) =
            (self.nodes.get(&start), self.nodes.get(&end))
        {
            (start_node.get_site(), end_node.get_site())
        } else {
            return None;
        };

        self.path_connection.remove_edge(start, end, handle.clone());

        self.path_tree.remove(&PathTreeObject::new(
            P::new(start_site, end_site, handle),
            (start, end),
        ));

        Some((start, end))
    }

    /// Remove all paths between two nodes.
    pub(crate) fn remove_connection(
        &mut self,
        start: NodeId,
        end: NodeId,
    ) -> Option<(NodeId, NodeId)> {
        let handles = self
            .path_connection
            .has_connection(start, end)?
            .iter()
            .map(|(handle, _)| handle.clone())
            .collect::<Vec<_>>();

        for handle in handles {
            self.remove_path(start, end, handle);
        }
        Some((start, end))
    }

    /// Get a node by its NodeId.
    pub fn get_node(&self, node_id: NodeId) -> Option<&N> {
        self.nodes.get(&node_id)
    }

    /// Check if there is a path between two nodes.
    ///
    /// If there is a path, return the attribute of the path.
    pub fn has_path(&self, start: NodeId, to: NodeId, handle: P::Handle) -> Option<PA> {
        self.path_connection
            .has_edge(start, to, &handle)
            .map(|(_, attr)| attr.clone())
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

    /// Search nodes around a path within a radius.
    pub fn nodes_around_path_iter(&self, path: P, radius: f64) -> impl Iterator<Item = &NodeId> {
        let corners = path.get_bounds();
        let envelope = rstar::AABB::from_corners(
            [corners.0.x - radius, corners.0.y - radius],
            [corners.1.x + radius, corners.1.y + radius],
        );
        self.node_tree
            .locate_in_envelope(&envelope)
            .filter(move |object: &&NodeTreeObject<NodeId>| {
                path.get_distance(object.site()) <= radius
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
    fn parse(&self) -> (Vec<N>, Vec<(usize, usize, P::Handle, PA)>) {
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
                let start_index = *node_id_to_index.get(&start)?;
                let end_index = *node_id_to_index.get(&end)?;
                Some(
                    self.path_connection
                        .has_connection(start, end)?
                        .iter()
                        .map(|(handle, attr)| (start_index, end_index, *handle, attr.clone()))
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
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
    use crate::core::geometry::path::{bezier::PathBezier, handle::PathHandle};

    use super::*;

    /// Node for testing the path network.
    /// This struct is used only for testing.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockNode(Site);

    impl MockNode {
        fn new(x: f64, y: f64) -> Self {
            Self(Site::new(x, y))
        }
    }

    impl NodeTrait for MockNode {
        fn get_site(&self) -> Site {
            self.0
        }
    }

    /// Path for testing the path network.
    /// This struct is used only for testing.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MockPath(PathBezier);

    impl PathTrait for MockPath {
        type Handle = PathHandle;

        fn new(start: Site, end: Site, handle: Self::Handle) -> Self {
            Self(PathBezier::new(start, end, handle))
        }

        fn get_handle(&self) -> Self::Handle {
            self.0.get_handle()
        }

        fn get_intersections(&self, other: &Self) -> Vec<Site> {
            self.0.get_intersections(&other.0)
        }

        fn get_projection(&self, site: &Site) -> Option<Site> {
            self.0.get_projection(site)
        }

        fn get_distance(&self, site: &Site) -> f64 {
            self.0.get_distance(site)
        }

        fn get_bounds(&self) -> (Site, Site) {
            self.0.get_bounds()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum MockAttr {
        A,
        B,
        C,
    }

    #[test]
    fn test_path_network() {
        let mut network: PathNetwork<MockNode, MockPath, MockAttr> = PathNetwork::new();
        let node0 = network.add_node(MockNode::new(0.0, 0.0));
        let node1 = network.add_node(MockNode::new(1.0, 1.0));
        let node2 = network.add_node(MockNode::new(2.0, 2.0));
        let node3 = network.add_node(MockNode::new(3.0, 3.0));
        let node4 = network.add_node(MockNode::new(1.0, 4.0));

        network.add_path(node0, node1, PathHandle::Linear, MockAttr::A);
        network.add_path(node1, node2, PathHandle::Linear, MockAttr::B);
        network.add_path(
            node1,
            node2,
            PathHandle::Quadratic(Site::new(1.5, 1.5)),
            MockAttr::C,
        );
        network.add_path(node2, node3, PathHandle::Linear, MockAttr::C);
        network.add_path(
            node3,
            node4,
            PathHandle::Cubic(Site::new(2.0, 3.0), Site::new(1.0, 3.5)),
            MockAttr::A,
        );
        network.add_path(node4, node2, PathHandle::Linear, MockAttr::A);
        assert_eq!(
            network.has_path(node0, node1, PathHandle::Linear),
            Some(MockAttr::A)
        );
        assert_eq!(
            network.has_path(node1, node2, PathHandle::Linear),
            Some(MockAttr::B)
        );
        assert_eq!(
            network.has_path(node1, node2, PathHandle::Quadratic(Site::new(1.5, 1.5))),
            Some(MockAttr::C)
        );
        assert_eq!(
            network.has_path(node2, node3, PathHandle::Linear),
            Some(MockAttr::C)
        );
        assert_eq!(
            network.has_path(
                node3,
                node4,
                PathHandle::Cubic(Site::new(2.0, 3.0), Site::new(1.0, 3.5))
            ),
            Some(MockAttr::A)
        );

        assert_eq!(
            network.has_path(node0, node1, PathHandle::Quadratic(Site::new(1.5, 1.5))),
            None
        );
        assert_eq!(network.has_path(node0, node2, PathHandle::Linear), None);

        assert!(network.check_path_state_is_consistent());

        // remove paths
        network.remove_path(node1, node2, PathHandle::Linear);
        assert_eq!(network.has_path(node1, node2, PathHandle::Linear), None);
        assert_eq!(
            network.has_path(node1, node2, PathHandle::Quadratic(Site::new(1.5, 1.5))),
            Some(MockAttr::C)
        );
        assert_eq!(
            network.has_path(node2, node3, PathHandle::Linear),
            Some(MockAttr::C)
        );

        assert!(network.check_path_state_is_consistent());

        network.remove_node(node1);

        assert_eq!(network.has_path(node0, node1, PathHandle::Linear), None);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_path_crossing_no_crosses() {
        let mut network: PathNetwork<MockNode, MockPath, ()> = PathNetwork::new();
        let node0 = network.add_node(MockNode::new(0.0, 1.0));
        let node1 = network.add_node(MockNode::new(2.0, 3.0));
        let node2 = network.add_node(MockNode::new(4.0, 5.0));

        network.add_path(node0, node1, PathHandle::Linear, ());
        network.add_path(node1, node2, PathHandle::Linear, ());

        let paths = network
            .paths_touching_rect_iter(Site::new(0.0, 0.0), Site::new(1.0, 1.0))
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 1);

        let node3 = network.add_node(MockNode::new(-0.5, 0.0));
        let node4 = network.add_node(MockNode::new(-0.5, 1.0));
        network.add_path(node3, node4, PathHandle::Quadratic(Site::new(0.5, 1.5)), ());

        let paths = network
            .paths_touching_rect_iter(Site::new(0.0, 0.0), Site::new(1.0, 1.0))
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 2);

        assert!(network.check_path_state_is_consistent());
    }

    #[test]
    fn test_path_crossing_all_cross() {
        let mut network: PathNetwork<MockNode, MockPath, ()> = PathNetwork::new();

        let nodes = vec![
            MockNode::new(0.0, 2.0),
            MockNode::new(2.0, 2.0),
            MockNode::new(2.0, 0.0),
            MockNode::new(0.0, 0.0),
        ];

        let nodes = nodes
            .into_iter()
            .map(|node| network.add_node(node))
            .collect::<Vec<_>>();

        for i in 0..nodes.len() {
            // Add all paths between nodes
            // When i == j, the path is expected to be ignored
            for j in i..nodes.len() {
                network.add_path(nodes[i], nodes[j], PathHandle::Linear, ());
            }
        }

        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i != j {
                    assert!(network
                        .has_path(NodeId(i), NodeId(j), PathHandle::Linear)
                        .is_some());
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
        let mut network: PathNetwork<MockNode, MockPath, ()> = PathNetwork::new();
        let node0 = network.add_node(MockNode::new(0.0, 0.0));
        let node1 = network.add_node(MockNode::new(1.0, 1.0));
        let node2 = network.add_node(MockNode::new(2.0, 2.0));
        let node3 = network.add_node(MockNode::new(3.0, 3.0));
        let node4 = network.add_node(MockNode::new(1.0, 4.0));

        network.add_path(node0, node1, PathHandle::Linear, ());
        network.add_path(node1, node2, PathHandle::Linear, ());
        network.add_path(node2, node3, PathHandle::Linear, ());
        network.add_path(node3, node4, PathHandle::Linear, ());
        network.add_path(node4, node2, PathHandle::Linear, ());

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

        let line = MockPath::new(Site::new(1.0, 3.0), Site::new(3.0, 2.0), PathHandle::Linear);
        let nodes = network
            .nodes_around_path_iter(line, 1.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        let line = MockPath::new(Site::new(1.0, 0.0), Site::new(0.0, 1.0), PathHandle::Linear);
        let nodes = network
            .nodes_around_path_iter(line, 2.5)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);

        network.remove_path(node3, node4, PathHandle::Linear);
        network.remove_node(node1);

        let site = Site::new(2.0, 1.0);
        let nodes = network
            .nodes_around_site_iter(site, 2.0)
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 1);

        let line = MockPath::new(Site::new(1.0, 0.0), Site::new(0.0, 1.0), PathHandle::Linear);
        let nodes = network
            .nodes_around_path_iter(line, 2.5)
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
    fn test_reconstruction() {
        let nodes = (0..300)
            .map(|i| MockNode::new(xorshift(i * 2) as f64, xorshift(i * 2 + 1) as f64))
            .collect::<Vec<_>>();

        let paths = {
            let mut paths = Vec::new();
            for i in 0..nodes.len() {
                for j in i + 1..nodes.len() {
                    let attr = &vec![MockAttr::A, MockAttr::B, MockAttr::C]
                        [xorshift(i * nodes.len() + j) % 3];
                    if xorshift(i * nodes.len() + j) % 2 == 0 {
                        paths.push((i, j, PathHandle::Linear, attr.clone()));
                    }
                }
            }
            paths
        };

        let mut network0: PathNetwork<MockNode, MockPath, MockAttr> = PathNetwork::new();
        let nodeids0 = nodes
            .clone()
            .into_iter()
            .map(|node| network0.add_node(node))
            .collect::<Vec<_>>();

        for (start, end, handle, attr) in paths.iter() {
            network0.add_path(
                nodeids0[*start],
                nodeids0[*end],
                handle.clone(),
                attr.clone(),
            );
        }

        let network1: PathNetwork<MockNode, MockPath, MockAttr> =
            PathNetwork::from(nodes.clone(), &paths).unwrap();
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
                let r0 = network0.has_path(nodeids0[i], nodeids0[j], PathHandle::Linear);
                let r1 = network1.has_path(nodeids1[i], nodeids1[j], PathHandle::Linear);
                assert_eq!(r0, r1);

                let r2 = network2.has_path(nodeids2[i], nodeids2[j], PathHandle::Linear);
                assert_eq!(r1, r2);

                let r3 = network3.has_path(nodeids3[i], nodeids3[j], PathHandle::Linear);
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

        let mut network: PathNetwork<MockNode, MockPath, u64> = PathNetwork::new();

        let nodeids = nodes
            .clone()
            .into_iter()
            .map(|node| network.add_node(node))
            .collect::<Vec<_>>();

        for l in 0..loop_count {
            let seed_start = l * nodes.len() * nodes.len();
            (0..nodes.len()).for_each(|i| {
                (0..nodes.len()).for_each(|j| {
                    let id = i * nodes.len() + j;
                    if xorshift(id + seed_start) % 2 == 0 {
                        network.add_path(nodeids[i], nodeids[j], PathHandle::Linear, id as u64);
                    }
                });
            });

            assert!(network.check_path_state_is_consistent());

            (0..nodes.len()).for_each(|i| {
                (0..nodes.len()).for_each(|j| {
                    let id = i * nodes.len() + j;
                    if xorshift(id + seed_start) % 3 == 0 {
                        network.remove_path(nodeids[i], nodeids[j], PathHandle::Linear);
                    }
                });
            });

            assert!(network.check_path_state_is_consistent());
        }
    }
}
