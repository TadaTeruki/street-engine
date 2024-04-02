use rstar::{RTree, RTreeObject};

use crate::core::geometry::{line_segment::LineSegment, site::Site};

use super::undirected::UndirectedGraph;

#[derive(Debug, Clone)]
pub struct NetworkBuilder<N>
where
    N: Eq + Ord + Copy + Into<Site>,
{
    path_tree: RTree<LineSegment>,
    path_connection: UndirectedGraph<N>,
}

impl<N> NetworkBuilder<N>
where
    N: Eq + Ord + Copy + Into<Site>,
{
    /// Create a new network.
    pub fn new() -> Self {
        Self {
            path_tree: RTree::new(),
            path_connection: UndirectedGraph::new(),
        }
    }

    /// Add a path between two nodes.
    fn add_path(&mut self, from: N, to: N) {
        if from == to {
            return;
        }
        if self.has_path(from, to) {
            return;
        }
        self.path_connection.add_edge(from, to);
        self.path_tree
            .insert(LineSegment::new(from.into(), to.into()));
    }

    /// Remove a path between two nodes.
    fn remove_path(&mut self, from: N, to: N) {
        self.path_connection.remove_edge(from, to);
        self.path_tree
            .remove(&LineSegment::new(from.into(), to.into()));
    }

    /// Remove a node from the network.
    fn remove_node(&mut self, node: N) {
        self.path_connection.neighbors_iter(node).map(|iter| {
            iter.for_each(|neighbor| {
                self.path_tree
                    .remove(&LineSegment::new(node.into(), (*neighbor).into()));
            });
        });
        self.path_connection.remove_node(node);
    }

    /// Check if there is a path between two nodes.
    fn has_path(&self, from: N, to: N) -> bool {
        self.path_connection.has_edge(from, to)
    }

    /// Search paths around a node within a radius.
    fn search_path_around_node(&self, node: N, radius: f64) -> Vec<&LineSegment> {
        self.path_tree
            .locate_within_distance([node.into().x, node.into().y], radius * radius)
            .collect::<Vec<_>>()
    }

    /// Search paths crossing a line segment.
    fn search_path_crossing(&self, line: LineSegment) -> Vec<&LineSegment> {
        self.path_tree
            .locate_in_envelope_intersecting(&line.into_rect().envelope())
            .filter(|path| path.get_intersection(&line).is_some())
            .collect::<Vec<_>>()
    }

    fn size(&self) -> usize {
        self.path_tree.size()
    }

    /// This function is only for testing
    #[allow(dead_code)]
    fn check_path_size_is_valid(&self) -> bool {
        self.path_tree.size() == self.path_connection.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network() {
        let mut network = NetworkBuilder::new();
        let site0 = Site::new(0.0, 0.0);
        let site1 = Site::new(1.0, 1.0);
        let site2 = Site::new(2.0, 2.0);
        let site3 = Site::new(3.0, 3.0);
        let site4 = Site::new(1.0, 4.0);

        network.add_path(site0, site1);
        network.add_path(site1, site2);
        network.add_path(site2, site3);
        network.add_path(site3, site4);
        network.add_path(site4, site2);

        assert_eq!(network.has_path(site0, site1), true);
        assert_eq!(network.has_path(site1, site2), true);
        assert_eq!(network.has_path(site2, site3), true);
        assert_eq!(network.has_path(site3, site4), true);
        assert_eq!(network.has_path(site0, site2), false);

        network.remove_path(site1, site2);
        assert_eq!(network.has_path(site1, site2), false);
        assert_eq!(network.has_path(site2, site3), true);

        network.remove_node(site1);
        assert_eq!(network.has_path(site0, site1), false);

        let paths = network.search_path_around_node(site2, 1.0);
        assert_eq!(paths.len(), 2);
        let paths = network.search_path_around_node(site2, 2.0);
        assert_eq!(paths.len(), 3);
        let paths = network.search_path_around_node(Site::new(1.1, 1.1), 1.0);
        assert_eq!(paths.len(), 0);

        let path = LineSegment::new(Site::new(1.0, 3.0), Site::new(3.0, 4.0));

        let paths = network.search_path_crossing(path);
        assert_eq!(paths.len(), 2);

        assert_eq!(network.check_path_size_is_valid(), true);
    }

    // Test with no crossing paths
    #[test]
    fn test_search_path_crossing_no_crosses() {
        let mut network = NetworkBuilder::new();
        let site0 = Site::new(0.0, 1.0);
        let site1 = Site::new(2.0, 3.0);
        let site2 = Site::new(4.0, 5.0);

        network.add_path(site0, site1);
        network.add_path(site1, site2);

        let path = LineSegment::new(Site::new(0.0, 0.95), Site::new(1.0, 1.95));

        let paths = network.search_path_crossing(path);
        assert_eq!(paths.len(), 0);

        assert_eq!(network.check_path_size_is_valid(), true);
    }

    // Test with all paths crossing
    #[test]
    fn test_search_path_crossing_all_cross() {
        let mut network = NetworkBuilder::new();

        let sites = vec![
            Site::new(0.0, 2.0),
            Site::new(2.0, 2.0),
            Site::new(2.0, 0.0),
            Site::new(0.0, 0.0),
        ];

        for i in 0..sites.len() {
            // Add all paths between sites
            // When i == j, the path is expected to be ignored
            for j in i..sites.len() {
                network.add_path(sites[i], sites[j]);
            }
        }

        for i in 0..sites.len() {
            for j in 0..sites.len() {
                if i != j {
                    assert_eq!(network.has_path(sites[i], sites[j]), true);
                }
            }
        }

        let path = LineSegment::new(Site::new(1.0, 3.0), Site::new(0.0, -1.0));

        let paths = network.search_path_crossing(path);
        assert_eq!(paths.len(), 4);

        assert_eq!(network.check_path_size_is_valid(), true);
    }

    // Test with intersecting at endpoints
    #[test]
    fn test_search_path_crossing_at_endpoints() {
        let mut network = NetworkBuilder::new();
        let site0 = Site::new(0.0, 0.0);
        let site1 = Site::new(1.0, 1.0);
        let site2 = Site::new(1.0, -1.0);

        network.add_path(site0, site1);
        network.add_path(site1, site2);

        let path = LineSegment::new(Site::new(1.0, 1.0), Site::new(2.0, 2.0));

        let paths = network.search_path_crossing(path);
        assert_eq!(paths.len(), 1);

        assert_eq!(network.check_path_size_is_valid(), true);
    }

    // Test creating a complex network
    #[test]
    fn test_complex_network() {
        let mut network_builder = NetworkBuilder::new();

        let xorshift = |x: usize| -> usize {
            let mut x = x;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x
        };

        let sites = (0..100)
            .map(|i| Site::new(xorshift(i * 2) as f64, xorshift(i * 2 + 1) as f64))
            .collect::<Vec<_>>();

        let loop_count = 100;

        for l in 0..loop_count {
            let seed_start = l * sites.len() * sites.len();
            (0..sites.len()).for_each(|i| {
                (0..sites.len()).for_each(|j| {
                    let id = i * sites.len() + j;
                    if xorshift(id + seed_start) % 2 == 0 {
                        network_builder.add_path(sites[i], sites[j]);
                    }
                });
            });

            assert!(network_builder.check_path_size_is_valid());

            (0..sites.len()).for_each(|i| {
                (0..sites.len()).for_each(|j| {
                    let id = i * sites.len() + j;
                    if xorshift(id + seed_start) % 3 == 0 {
                        network_builder.remove_path(sites[i], sites[j]);
                    }
                });
            });

            assert!(network_builder.check_path_size_is_valid());

            assert!(network_builder.check_path_size_is_valid());

            println!("Loop: {}", l);
        }
    }
}
