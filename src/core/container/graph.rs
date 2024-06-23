use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct GraphKey<N>
where
    N: Eq + Ord + Copy,
{
    smaller: N,
    larger: N,
}

impl<N> GraphKey<N>
where
    N: Eq + Ord + Copy,
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

/// Undirected edge-attributed multigraph.
/// 
/// This is a graph where the edges are undirected, have attributes
/// and can have multiple paths between two nodes.
/// 
/// N: Node.
/// 
/// H: Handle. This is the identifier of the edge.
/// 
/// A: Attribute. This is the attribute of the edge.
#[derive(Debug, Clone)]
pub struct UeamGraph<N, H, A>
where
    N: Eq + Ord + Copy,
    H: Eq,
    A: Eq,
{
    edges: BTreeMap<N, BTreeSet<N>>,
    handles: BTreeMap<GraphKey<N>, Vec<(H, A)>>,
}

impl<N, H, A> UeamGraph<N, H, A>
where
    N: Eq + Ord + Copy,
    H: Eq,
    A: Eq,
{
    /// Create a new undirected graph.
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
            handles: BTreeMap::new(),
        }
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, a: N, b: N, handle: H, attr: A) -> Option<(N, N)> {
        self.edges.entry(a).or_default().insert(b);
        self.edges.entry(b).or_default().insert(a);
        self.handles
            .entry(GraphKey::new(a, b))
            .or_default()
            .push((handle, attr));
        Some((a, b))
    }

    /// Check if the two nodes are connected.
    pub fn has_connection(&self, a: N, b: N) -> Option<&Vec<(H, A)>> {
        self.edges.get(&a).and_then(|set| {
            if set.contains(&b) {
                self.handles.get(&GraphKey::new(a, b))
            } else {
                None
            }
        })
    }

    /// Remove all connections between two nodes.
    pub fn remove_connection(&mut self, a: N, b: N) -> Option<(N, N)> {
        self.handles.remove(&GraphKey::new(a, b));
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

    pub fn has_edge(&self, a: N, b: N, handle: &H) -> Option<&(H, A)> {
        self.handles
            .get(&GraphKey::new(a, b))
            .and_then(|vec| vec.iter().find(|x| &x.0 == handle))
    }

    /// Remove an edge from the graph.
    pub fn remove_edge(&mut self, a: N, b: N, handle: H) -> Option<(N, N, H)> {
        let key = GraphKey::new(a, b);
        self.handles.entry(key).and_modify(|vec| {
            vec.retain(|x| x.0 != handle);
        });
        // if there is no handleibution, remove the connection
        if self.handles.get(&key).map_or(true, |vec| vec.is_empty()) {
            self.remove_connection(a, b);
        }

        Some((a, b, handle))
    }

    /// Get the number of nodes in the graph.
    /// This function is now only for testing, but it may be useful in the future.
    #[allow(dead_code)]
    fn order(&self) -> usize {
        self.edges.len()
    }

    /// Get the number of edges in the graph from the edges.
    ///
    /// This is only for testing as it is slower than `size_from_handles`.
    #[allow(dead_code)]
    fn size_from_edges(&self) -> usize {
        self.edges.values().map(|set| set.len()).sum::<usize>() / 2
    }

    /// Get the number of edges in the graph from the handles.
    fn size_from_handles(&self) -> usize {
        self.handles.values().map(|vec| vec.len()).sum::<usize>()
    }

    /// Check if the size of the graph is consistent.
    #[allow(dead_code)]
    fn check_size(&self) -> bool {
        self.size_from_edges() == self.size_from_handles()
    }

    /// Get the number of edges in the graph.
    pub fn size(&self) -> usize {
        self.size_from_handles()
    }

    /// Get the neighbors of a node as an iterator.
    pub fn neighbors_iter(&self, node: N) -> impl Iterator<Item = &N> + '_ {
        self.edges
            .get(&node)
            .map(|set| set.iter())
            .into_iter()
            .flatten()
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
            for x in $a.iter() {
                assert!($b.contains(&x));
            }
        };
    }

    #[test]
    fn test_undirected_graph() {
        let mut graph = UeamGraph::new();

        graph.add_edge(103, 25, 0, "a");
        graph.add_edge(85, 103, 1, "b");
        graph.add_edge(85, 103, 2, "a");
        graph.add_edge(85, 32, 3, "d");
        graph.add_edge(67, 25, 4, "e");

        let neighbors = graph.neighbors_iter(103).map(|&x| x).collect::<Vec<i32>>();
        assert_eq_vec!(neighbors, vec![25, 85]);

        assert_eq!(graph.order(), 5);
        assert_eq!(graph.size(), 5);
        assert_eq!(graph.has_edge(103, 25, &0), Some(&(0, "a")));
        assert!(graph.has_edge(25, 103, &0).is_some());
        assert!(graph.has_edge(103, 25, &1).is_none());

        assert_eq!(graph.has_edge(85, 103, &1), Some(&(1, "b")));
        assert!(graph.has_edge(103, 85, &1).is_some());
        assert!(graph.has_edge(85, 103, &2).is_some());

        assert_eq_vec!(
            graph.has_connection(85, 103).unwrap(),
            vec![&(1, "b"), &(2, "a")]
        );

        assert!(graph.has_edge(85, 32, &3).is_some());
        assert!(graph.has_edge(32, 85, &3).is_some());

        // remove connection
        graph.remove_connection(103, 85);

        assert_eq!(graph.order(), 5);
        assert_eq!(graph.size(), 3);
        assert!(graph.has_connection(103, 85).is_none());
        assert!(graph.has_connection(85, 103).is_none());
        assert!(graph.has_edge(103, 85, &1).is_none());
        assert!(graph.has_edge(85, 103, &1).is_none());
        assert!(graph.has_edge(85, 32, &3).is_some());

        // remove edge (not existing)
        graph.remove_edge(85, 32, 2);
        assert!(graph.has_edge(85, 32, &3).is_some());

        // remove edge
        graph.remove_edge(85, 32, 3);

        assert_eq!(graph.order(), 3);
        assert_eq!(graph.size(), 2);
        assert!(graph.has_edge(85, 32, &3).is_none());
        assert!(graph.has_edge(32, 85, &3).is_none());
        assert!(graph.has_edge(103, 25, &0).is_some());

        assert!(graph.check_size());
    }
}
