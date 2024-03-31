use rstar::{RTree, RTreeObject};

use crate::core::geometry::{line_segment::LineSegment, site::Site};

use super::undirected::UndirectedGraph;

/// Represents a network of sites.
struct Network {
    path_tree: RTree<LineSegment>,
    path_connection: UndirectedGraph<Site>,
}

impl Network {
    /// Create a new network.
    fn new() -> Self {
        Self {
            path_tree: RTree::new(),
            path_connection: UndirectedGraph::new(),
        }
    }

    /// Add a path between two sites.
    fn add_path(&mut self, from: Site, to: Site) {
        if from == to {
            return;
        }
        self.path_connection.add_edge(from, to);
        self.path_tree.insert(LineSegment::new(from, to));
    }

    /// Remove a path between two sites.
    fn remove_path(&mut self, from: Site, to: Site) {
        self.path_connection.remove_edge(from, to);
        self.path_tree.remove(&LineSegment::new(from, to));
    }

    /// Check if there is a path between two sites.
    fn has_path(&self, from: Site, to: Site) -> bool {
        self.path_connection.has_edge(from, to)
    }

    /// Remove a site from the network.
    fn remove_site(&mut self, site: Site) {
        self.path_connection.neighbors_iter(site).map(|iter| {
            iter.for_each(|neighbor| {
                self.path_tree.remove(&LineSegment::new(site, *neighbor));
            });
        });
        self.path_connection.remove_node(site);
    }

    /// Search paths around a site within a radius.
    fn search_path_around_site(&self, site: Site, radius: f64) -> Vec<&LineSegment> {
        self.path_tree
            .locate_within_distance([site.x, site.y], radius * radius)
            .collect::<Vec<_>>()
    }

    /// Search paths crossing a line segment.
    fn search_path_crossing(&self, line: LineSegment) -> Vec<&LineSegment> {
        self.path_tree
            .locate_in_envelope_intersecting(&line.into_rect().envelope())
            .filter(|path| path.get_intersection(&line).is_some())
            .collect::<Vec<_>>()
    }

    /// this function is only for testing
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
        let mut network = Network::new();
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

        network.remove_site(site1);
        assert_eq!(network.has_path(site0, site1), false);

        let paths = network.search_path_around_site(site2, 1.0);
        assert_eq!(paths.len(), 2);
        let paths = network.search_path_around_site(site2, 2.0);
        assert_eq!(paths.len(), 3);
        let paths = network.search_path_around_site(Site::new(1.1, 1.1), 1.0);
        assert_eq!(paths.len(), 0);

        let path = LineSegment::new(Site::new(1.0, 3.0), Site::new(3.0, 4.0));

        let paths = network.search_path_crossing(path);
        assert_eq!(paths.len(), 2);

        assert_eq!(network.check_path_size_is_valid(), true);
    }

    // Test with no crossing paths
    #[test]
    fn test_search_path_crossing_no_crosses() {
        let mut network = Network::new();
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
        let mut network = Network::new();

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
        let mut network = Network::new();
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
}