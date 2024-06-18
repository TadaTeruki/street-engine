use rstar::{PointDistance, RTreeObject};

use crate::core::geometry::{path::PathTrait, site::Site};

pub trait PathTreeIDTrait: Copy + PartialEq {}
impl<T> PathTreeIDTrait for T where T: Copy + PartialEq {}

#[derive(Debug, Clone)]
pub struct PathTreeObject<ID, P>
where
    ID: PathTreeIDTrait,
    P: PathTrait,
{
    path: P,
    node_ids: (ID, ID),
}

impl<ID, P> PathTreeObject<ID, P>
where
    ID: PathTreeIDTrait,
    P: PathTrait,
{
    pub fn new(path: P, node_ids: (ID, ID)) -> Self {
        Self { path, node_ids }
    }

    pub fn node_ids(&self) -> &(ID, ID) {
        &self.node_ids
    }
}

impl<ID, P> RTreeObject for PathTreeObject<ID, P>
where
    ID: PathTreeIDTrait,
    P: PathTrait,
{
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let (start, end) = self.path.get_bounds();
        rstar::AABB::from_corners([start.x, start.y], [end.x, end.y])
    }
}

impl<ID, P> PointDistance for PathTreeObject<ID, P>
where
    ID: PathTreeIDTrait,
    P: PathTrait,
{
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.path
            .get_distance(&Site::new(point[0], point[1]))
            .powi(2)
    }
}

impl<ID, P> PartialEq for PathTreeObject<ID, P>
where
    ID: PathTreeIDTrait,
    P: PathTrait,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ids == other.node_ids || self.node_ids == (other.node_ids.1, other.node_ids.0)
    }
}

#[derive(Debug, Clone)]
pub struct NodeTreeObject<ID>
where
    ID: PathTreeIDTrait,
{
    site: Site,
    node_id: ID,
}

impl<ID> NodeTreeObject<ID>
where
    ID: PathTreeIDTrait,
{
    pub fn new(site: Site, node_id: ID) -> Self {
        Self { site, node_id }
    }

    pub fn site(&self) -> &Site {
        &self.site
    }

    pub fn node_id(&self) -> &ID {
        &self.node_id
    }
}

impl<ID> RTreeObject for NodeTreeObject<ID>
where
    ID: PathTreeIDTrait,
{
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_point([self.site.x, self.site.y])
    }
}

impl<ID> PointDistance for NodeTreeObject<ID>
where
    ID: PathTreeIDTrait,
{
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.site.x - point[0];
        let dy = self.site.y - point[1];
        dx * dx + dy * dy
    }
}

impl<ID> PartialEq for NodeTreeObject<ID>
where
    ID: PathTreeIDTrait,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}
