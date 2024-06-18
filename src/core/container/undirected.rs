use std::collections::{BTreeMap, BTreeSet};

pub trait UndirectedGraphNodeTrait: Eq + Ord + Copy {}
impl<T> UndirectedGraphNodeTrait for T where T: Eq + Ord + Copy {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AttributionKey<N>
where
    N: UndirectedGraphNodeTrait,
{
    smaller: N,
    larger: N,
}

impl<N> AttributionKey<N>
where
    N: UndirectedGraphNodeTrait,
{
    fn new(a: N, b: N) -> Self {
        if a < b {
            Self {
                smaller: a,
                larger: b,
            }
        } else {
            Self {
                smaller: b,
                larger: a,
            }
        }
    }
}

/// Undirected graph.
#[derive(Debug, Clone)]
pub struct UndirectedGraph<N, A>
where
    N: UndirectedGraphNodeTrait,
    A: Eq,
{
    edges: BTreeMap<N, BTreeSet<N>>,
    attributions: BTreeMap<AttributionKey<N>, Vec<A>>,
}

impl<N, A> UndirectedGraph<N, A>
where
    N: UndirectedGraphNodeTrait,
    A: Eq,
{
    /// Create a new undirected graph.
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
            attributions: BTreeMap::new(),
        }
    }

    /// Add an edge to the graph.
    ///
    /// Is the edge already in the graph, only the attribute is updated.
    pub fn add_edge(&mut self, a: N, b: N, attr: A) -> Option<(N, N)> {
        self.edges.entry(a).or_default().insert(b);
        self.edges.entry(b).or_default().insert(a);
        self.attributions
            .entry(AttributionKey::new(a, b))
            .or_default()
            .push(attr);
        Some((a, b))
    }

    /// Check if the two nodes are connected.
    pub fn has_connection(&self, a: N, b: N) -> Option<&Vec<A>> {
        self.edges.get(&a).and_then(|set| {
            if set.contains(&b) {
                self.attributions.get(&AttributionKey::new(a, b))
            } else {
                None
            }
        })
    }

    /// Remove all connections between two nodes.
    pub fn remove_connection(&mut self, a: N, b: N) -> Option<(N, N)> {
        self.attributions.remove(&AttributionKey::new(a, b));
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

    pub fn has_edge(&self, a: N, b: N, attr: A) -> Option<&A> {
        self.attributions
            .get(&AttributionKey::new(a, b))
            .and_then(|vec| vec.iter().find(|x| *x == &attr))
    }

    /// Remove an edge from the graph.
    pub fn remove_edge(&mut self, a: N, b: N, attr: A) -> Option<(N, N, A)> {
        let key = AttributionKey::new(a, b);
        self.attributions.entry(key).and_modify(|vec| {
            vec.retain(|x| x != &attr);
        });
        // if there is no attribution, remove the connection
        if self.attributions[&key].is_empty() {
            self.remove_connection(a, b);
        }

        Some((a, b, attr))
    }

    /// Get the number of nodes in the graph.
    /// This function is now only for testing, but it may be useful in the future.
    #[allow(dead_code)]
    fn order(&self) -> usize {
        self.edges.len()
    }

    /// Get the number of edges in the graph from the edges.
    ///
    /// This is only for testing as it is slower than `size_from_attributions`.
    #[allow(dead_code)]
    fn size_from_edges(&self) -> usize {
        self.edges.values().map(|set| set.len()).sum::<usize>() / 2
    }

    /// Get the number of edges in the graph from the attributions.
    fn size_from_attributions(&self) -> usize {
        self.attributions
            .values()
            .map(|vec| vec.len())
            .sum::<usize>()
    }

    /// Check if the size of the graph is consistent.
    #[allow(dead_code)]
    fn check_size(&self) -> bool {
        self.size_from_edges() == self.size_from_attributions()
    }

    /// Get the number of edges in the graph.
    pub fn size(&self) -> usize {
        self.size_from_attributions()
    }

    /// Get the neighbors of a node as an iterator.
    pub fn neighbors_iter(&self, node: N) -> Option<impl Iterator<Item = &N> + '_> {
        self.edges.get(&node).map(|set| set.iter())
    }

    /// Get the pairs of connected nodes as an iterator.
    pub fn edges_iter(&self) -> impl Iterator<Item = (N, N)> + '_ {
        self.edges
            .iter()
            .flat_map(|(a, set)| set.iter().map(move |&b| (*a, b)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_eq_vec {
        ($a:expr, $b:expr) => {
            assert_eq!($a.len(), $b.len());
            for x in $a {
                assert!($b.contains(&x));
            }
        };
    }

    #[test]
    fn test_undirected_graph() {
        let mut graph = UndirectedGraph::new();

        graph.add_edge(103, 25, 0);
        graph.add_edge(85, 103, 1);
        graph.add_edge(85, 103, 2);
        graph.add_edge(85, 32, 3);
        graph.add_edge(67, 25, 4);

        let neighbors = graph
            .neighbors_iter(103)
            .unwrap()
            .map(|&x| x)
            .collect::<Vec<i32>>();
        assert_eq_vec!(neighbors, vec![25, 85]);

        assert_eq!(graph.order(), 5);
        assert_eq!(graph.size(), 5);
        assert_eq!(graph.has_edge(103, 25, 0), Some(&0));
        assert!(graph.has_edge(25, 103, 0).is_some());
        assert!(graph.has_edge(103, 25, 1).is_none());

        assert_eq!(graph.has_edge(85, 103, 1), Some(&1));
        assert!(graph.has_edge(103, 85, 1).is_some());
        assert!(graph.has_edge(85, 103, 2).is_some());

        assert_eq_vec!(graph.has_connection(85, 103).unwrap(), vec![1, 2]);

        assert!(graph.has_edge(85, 32, 3).is_some());
        assert!(graph.has_edge(32, 85, 3).is_some());

        // remove connection
        graph.remove_connection(103, 85);

        assert_eq!(graph.order(), 5);
        assert_eq!(graph.size(), 3);
        assert!(graph.has_connection(103, 85).is_none());
        assert!(graph.has_connection(85, 103).is_none());
        assert!(graph.has_edge(103, 85, 1).is_none());
        assert!(graph.has_edge(85, 103, 1).is_none());
        assert!(graph.has_edge(85, 32, 3).is_some());

        // remove edge (not existing)
        graph.remove_edge(85, 32, 2);
        assert!(graph.has_edge(85, 32, 3).is_some());

        // remove edge
        graph.remove_edge(85, 32, 3);

        assert_eq!(graph.order(), 3);
        assert_eq!(graph.size(), 2);
        assert!(graph.has_edge(85, 32, 3).is_none());
        assert!(graph.has_edge(32, 85, 3).is_none());
        assert!(graph.has_edge(103, 25, 0).is_some());

        assert!(graph.check_size());
    }
}
