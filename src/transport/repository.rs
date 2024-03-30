use rstar::RTree;

use crate::{container::undirected::UndirectedGraph, geometry::site::Site};

struct TransportRepository {
    tree: RTree<Site>,
    connection: UndirectedGraph<Site>,
}

impl TransportRepository {
    fn new() -> Self {
        Self {
            tree: RTree::new(),
            connection: UndirectedGraph::new(),
        }
    }

    fn add_site(&mut self, site: Site) {
        self.tree.insert(site);
    }

    fn remove_site(&mut self, site: Site) {
        self.tree.remove(&site);
    }

    fn add_path(&mut self, from: Site, to: Site) {
        self.connection.add_edge(from, to);
    }

    fn remove_path(&mut self, from: Site, to: Site) {
        self.connection.remove_edge(from, to);
    }
}
