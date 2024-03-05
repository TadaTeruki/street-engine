use crate::geom::Site2D;

#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub site: Site2D,
    pub angle: f64,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            site: Site2D { x: 0.0, y: 0.0 },
            angle: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct NetworkTreeObject {
    pub site: Site2D,
    pub id: usize,
}

impl rstar::RTreeObject for NetworkTreeObject {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_point([self.site.x, self.site.y])
    }
}

impl rstar::PointDistance for NetworkTreeObject {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.site.distance(&Site2D::new(point[0], point[1]))
    }
}

#[derive(Debug, Clone)]
pub struct Network {
    nodes: Vec<Node>,
    connection: Vec<Vec<usize>>,
    search_tree: rstar::RTree<NetworkTreeObject>,
}

impl Network {
    pub(super) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connection: Vec::new(),
            search_tree: rstar::RTree::new(),
        }
    }

    pub(super) fn add_new_node(&mut self, node: Node) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        self.connection.push(Vec::new());
        self.search_tree.insert(NetworkTreeObject {
            site: node.site,
            id,
        });
        id
    }

    pub(super) fn connect_nodes(&mut self, from: usize, to: usize) {
        self.connection[from].push(to);
        self.connection[to].push(from);
    }

    pub(super) fn get_nearest_node(&self, site: Site2D) -> Option<usize> {
        self.search_tree
            .nearest_neighbor_iter(&[site.x, site.y])
            .next()
            .map(|nearest| nearest.id)
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn has_connection(&self, from: usize, to: usize) -> Option<bool> {
        if from >= self.connection.len() || to >= self.connection.len() {
            return None;
        }
        Some(self.connection[from].contains(&to))
    }

    pub fn for_each_neighbor<F>(&self, from: usize, mut f: F)
    where
        F: FnMut(usize),
    {
        if from >= self.connection.len() {
            return;
        }
        for &to in &self.connection[from] {
            f(to);
        }
    }
}
