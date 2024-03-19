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
struct NetworkSiteTreeObject {
    pub site: Site2D,
    pub id: usize,
}

impl rstar::RTreeObject for NetworkSiteTreeObject {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_point([self.site.x, self.site.y])
    }
}

impl rstar::PointDistance for NetworkSiteTreeObject {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.site.distance(&Site2D::new(point[0], point[1]))
    }
}

#[derive(Debug, Clone)]
pub struct Network {
    nodes: Vec<Node>,
    connection: Vec<Vec<usize>>,
    site_search_tree: rstar::RTree<NetworkSiteTreeObject>,
}

pub(super) enum IntersectionType {
    Cross,
    Nearest,
}

impl Network {
    pub(super) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connection: Vec::new(),
            site_search_tree: rstar::RTree::new(),
        }
    }

    pub(super) fn add_new_node(&mut self, node: Node) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        self.connection.push(Vec::new());
        self.site_search_tree.insert(NetworkSiteTreeObject {
            site: node.site,
            id,
        });
        id
    }

    pub(super) fn connect_nodes(&mut self, from: usize, to: usize) {
        self.remove_connection(from, to);
        self.connection[from].push(to);
        self.connection[to].push(from);

        println!("Connection: {} -> {}", from, to);
    }

    pub(super) fn remove_connection(&mut self, from: usize, to: usize) {
        self.connection[from].retain(|&id| id != to);
        self.connection[to].retain(|&id| id != from);

        println!("Remove connection: {} -> {}", from, to);
    }

    pub(super) fn get_nearest_node_in_distance(
        &self,
        backward_site: Site2D,
        forward_site: Site2D,
        distance: f64,
    ) -> Option<usize> {
        let envelope = rstar::AABB::from_corners(
            [forward_site.x - distance, forward_site.y - distance],
            [forward_site.x + distance, forward_site.y + distance],
        );

        self.site_search_tree
            .locate_in_envelope(&envelope)
            .filter_map(|node| {
                if node.site.distance(&forward_site) < distance {
                    Some(node.id)
                } else {
                    None
                }
            })
            .min_by(|&from, &to| {
                self.nodes[from]
                    .site
                    .distance(&backward_site)
                    .partial_cmp(&self.nodes[to].site.distance(&backward_site))
                    .unwrap()
            })
    }

    pub(super) fn get_intersection(
        &self,
        backward_site: Site2D,
        forward_site: Site2D,
        merge_distance: f64,
        connection_length: f64,
        exclude_id: Option<usize>,
        intersection_type: IntersectionType,
    ) -> Option<(Site2D, usize, usize)> {
        let half_search_range = 2.0_f64.sqrt();
        let envelope = rstar::AABB::from_corners(
            [
                forward_site.x - connection_length * half_search_range,
                forward_site.y - connection_length * half_search_range,
            ],
            [
                forward_site.x + connection_length * half_search_range,
                forward_site.y + connection_length * half_search_range,
            ],
        );

        self.site_search_tree
            .locate_in_envelope(&envelope)
            .flat_map(|node| {
                self.connection[node.id]
                    .iter()
                    .map(move |&to| (node.id, to))
            })
            .filter_map(|(from, to)| {
                if let Some(exclude_id) = exclude_id {
                    if from == exclude_id || to == exclude_id {
                        return None;
                    }
                }

                let from_site = &self.nodes[from].site;
                let to_site = &self.nodes[to].site;

                if let IntersectionType::Cross = intersection_type {
                    if let Some(cross_site) =
                        forward_site.get_intersection(&backward_site, from_site, to_site)
                    {
                        println!(
                            "Cross site: {:?}",
                            (cross_site, from, to, cross_site.distance(&forward_site))
                        );
                        return Some((cross_site, from, to, cross_site.distance(&forward_site)));
                    }
                }

                if let IntersectionType::Nearest = intersection_type {
                    if let Some(nearest_site) =
                        forward_site.get_nearest_point_on_line_segment(from_site, to_site)
                    {
                        let distance = nearest_site.distance(&forward_site);
                        if distance < merge_distance {
                            println!("Nearest site: {:?}", (nearest_site, from, to, distance));
                            return Some((nearest_site, from, to, distance));
                        }
                    }
                }
                None
            })
            .min_by(|(_, _, _, distance), (_, _, _, other_distance)| {
                distance.partial_cmp(other_distance).unwrap()
            })
            .map(|(cross_site, from, to, _)| (cross_site, from, to))
    }

    pub fn get_node(&self, id: usize) -> Option<Node> {
        self.nodes.get(id).cloned()
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
