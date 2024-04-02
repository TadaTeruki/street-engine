use std::collections::{btree_set, BTreeMap, BTreeSet};

/// Undirected graph.
#[derive(Debug, Clone)]
pub struct UndirectedGraph<N>
where
    N: Eq + Ord + Copy,
{
    edges: BTreeMap<N, BTreeSet<N>>,
}

impl<N> UndirectedGraph<N>
where
    N: Eq + Ord + Copy,
{
    /// Create a new undirected graph.
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, a: N, b: N) {
        if self.has_edge(a, b) {
            return;
        }
        self.edges.entry(a).or_insert_with(BTreeSet::new).insert(b);
        self.edges.entry(b).or_insert_with(BTreeSet::new).insert(a);
    }

    /// Check if there is an edge between two nodes.
    pub fn has_edge(&self, a: N, b: N) -> bool {
        self.edges.get(&a).map_or(false, |set| set.contains(&b))
    }

    /// Remove an edge from the graph.
    pub fn remove_edge(&mut self, a: N, b: N) {
        if let Some(set) = self.edges.get_mut(&a) {
            set.remove(&b);
            if set.is_empty() {
                self.edges.remove(&a);
            }
        }
        if let Some(set) = self.edges.get_mut(&b) {
            set.remove(&a);
            if set.is_empty() {
                self.edges.remove(&b);
            }
        }
    }

    /// Get the number of nodes in the graph.
    fn order(&self) -> usize {
        self.edges.len()
    }

    /// Get the number of edges in the graph.
    pub fn size(&self) -> usize {
        self.edges.values().map(|set| set.len()).sum::<usize>() / 2
    }

    /// Get the nodes of the graph.
    pub fn nodes(&self) -> Vec<N> {
        self.edges.keys().copied().collect()
    }

    /// Get the edges of the graph as an iterator.
    pub fn edges_iter(&self) -> impl Iterator<Item = (N, N)> + '_ {
        self.edges.iter().flat_map(|(node, set)| {
            set.iter()
                .filter(move |&neighbor| *node < *neighbor)
                .map(move |&neighbor| (*node, neighbor))
        })
    }

    /// Get the neighbors of a node as an iterator.
    pub fn neighbors_iter(&self, node: N) -> Option<btree_set::Iter<N>> {
        self.edges.get(&node).map(|set| set.iter())
    }

    /// Remove a node from the graph.
    pub fn remove_node(&mut self, node: N) {
        if let Some(set) = self.edges.remove(&node) {
            set.iter().for_each(|neighbor| {
                self.edges.get_mut(&neighbor).map(|set| set.remove(&node));
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undirected_graph() {
        let mut graph = UndirectedGraph::new();

        graph.add_edge(103, 25);
        graph.add_edge(85, 103);
        graph.add_edge(85, 103);
        graph.add_edge(85, 32);
        graph.add_edge(67, 25);

        let neighbors = graph
            .neighbors_iter(103)
            .unwrap()
            .map(|&x| x)
            .collect::<Vec<_>>();
        assert_eq!(neighbors, vec![25, 85]);

        let edges = graph.edges_iter().collect::<Vec<_>>();
        assert_eq!(edges.len(), graph.size(),);

        assert_eq!(graph.order(), 5);
        assert_eq!(graph.size(), 4);
        assert_eq!(graph.has_edge(103, 25), true);
        assert_eq!(graph.has_edge(25, 103), true);
        assert_eq!(graph.has_edge(85, 103), true);
        assert_eq!(graph.has_edge(103, 85), true);
        assert_eq!(graph.has_edge(85, 25), false);
        assert_eq!(graph.has_edge(25, 85), false);

        graph.remove_edge(103, 25);

        assert_eq!(graph.order(), 5);
        assert_eq!(graph.size(), 3);
        assert_eq!(graph.has_edge(103, 25), false);
        assert_eq!(graph.has_edge(25, 103), false);
        assert_eq!(graph.has_edge(85, 103), true);

        graph.remove_edge(85, 103);

        assert_eq!(graph.order(), 4);
        assert_eq!(graph.size(), 2);
        assert_eq!(graph.has_edge(85, 103), false);
        assert_eq!(graph.has_edge(103, 85), false);
        assert_eq!(graph.has_edge(85, 32), true);
    }
}
