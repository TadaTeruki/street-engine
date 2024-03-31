use std::collections::{btree_set::Iter, BTreeMap, BTreeSet};

/// Undirected graph.
pub struct UndirectedGraph<T>
where
    T: Eq + Ord + Copy,
{
    edges: BTreeMap<T, BTreeSet<T>>,
}

impl<T> UndirectedGraph<T>
where
    T: Eq + Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }

    pub fn add_edge(&mut self, a: T, b: T) {
        if self.has_edge(a, b) {
            return;
        }
        self.edges.entry(a).or_insert_with(BTreeSet::new).insert(b);
        self.edges.entry(b).or_insert_with(BTreeSet::new).insert(a);
    }

    pub fn has_edge(&self, a: T, b: T) -> bool {
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

    pub fn size(&self) -> usize {
        self.edges.values().map(|set| set.len()).sum::<usize>() / 2
    }

    pub fn neighbors_iter(&self, node: T) -> Option<Iter<T>> {
        self.edges.get(&node).map(|set| set.iter())
    }

    pub fn remove_node(&mut self, node: T) {
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

        let iter = graph.neighbors_iter(103).unwrap();
        let mut neighbors = iter.map(|&x| x).collect::<Vec<_>>();
        neighbors.sort();
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
