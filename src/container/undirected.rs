use std::{
    collections::{hash_set::Iter, HashMap, HashSet},
    hash::Hash,
};

/// Undirected graph.
pub struct UndirectedGraph<T>
where
    T: Eq + Hash + Copy,
{
    edges: HashMap<T, HashSet<T>>,
}

impl<T> UndirectedGraph<T>
where
    T: Eq + Hash + Copy,
{
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, a: T, b: T) {
        self.edges.entry(a).or_insert_with(HashSet::new).insert(b);
        self.edges.entry(b).or_insert_with(HashSet::new).insert(a);
    }

    fn has_edge(&self, a: T, b: T) -> bool {
        self.edges.get(&a).map_or(false, |set| set.contains(&b))
    }

    pub fn remove_edge(&mut self, a: T, b: T) {
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

    fn order(&self) -> usize {
        self.edges.len()
    }

    fn size(&self) -> usize {
        self.edges.values().map(|set| set.len()).sum::<usize>() / 2
    }

    pub fn neighbors_iter(&self, node: T) -> Option<Iter<T>> {
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
        graph.add_edge(85, 32);
        graph.add_edge(67, 25);

        let mut iter = graph.neighbors_iter(103).unwrap();
        assert_eq!(iter.next(), Some(&25));
        assert_eq!(iter.next(), Some(&85));
        assert_eq!(iter.next(), None);

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
