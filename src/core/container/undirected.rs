use std::collections::{BTreeMap, BTreeSet};

pub trait UndirectedGraphNodeTrait: Eq + Ord + Copy {}
impl<T> UndirectedGraphNodeTrait for T where T: Eq + Ord + Copy {}

/// Undirected graph.
#[derive(Debug, Clone)]
pub struct UndirectedGraph<N>
where
    N: UndirectedGraphNodeTrait,
{
    edges: BTreeMap<N, BTreeSet<N>>,
}

impl<N> UndirectedGraph<N>
where
    N: UndirectedGraphNodeTrait,
{
    /// Create a new undirected graph.
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, a: N, b: N) -> Option<(N, N)> {
        if self.has_edge(a, b) {
            return None;
        }
        self.edges.entry(a).or_default().insert(b);
        self.edges.entry(b).or_default().insert(a);
        Some((a, b))
    }

    /// Check if there is an edge between two nodes.
    pub fn has_edge(&self, a: N, b: N) -> bool {
        self.edges.get(&a).is_some_and(|set| set.contains(&b))
    }

    /// Remove an edge from the graph.
    pub fn remove_edge(&mut self, a: N, b: N) -> Option<(N, N)> {
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

        Some((a, b))
    }

    /// Get the number of nodes in the graph.
    /// This function is now only for testing, but it may be useful in the future.
    #[cfg(test)]
    fn order(&self) -> usize {
        self.edges.len()
    }

    #[cfg(test)]
    /// Get the number of edges in the graph.
    pub fn size(&self) -> usize {
        self.edges.values().map(|set| set.len()).sum::<usize>() / 2
    }

    /// Get the neighbors of a node as an iterator.
    pub fn neighbors_iter(&self, node: N) -> Option<impl Iterator<Item = &N> + '_> {
        self.edges.get(&node).map(|set| set.iter())
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
