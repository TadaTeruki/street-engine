use rstar::{PointDistance, RTreeObject};

use crate::core::geometry::{line_segment::LineSegment, site::Site};

#[derive(Debug, Clone)]
pub struct PathTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    line_segment: LineSegment,
    node_ids: (ID, ID),
}

impl<ID> PathTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    pub fn new(line_segment: LineSegment, node_ids: (ID, ID)) -> Self {
        Self {
            line_segment,
            node_ids,
        }
    }

    pub fn line_segment(&self) -> &LineSegment {
        &self.line_segment
    }

    pub fn node_ids(&self) -> &(ID, ID) {
        &self.node_ids
    }
}

impl<ID> RTreeObject for PathTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_corners(
            [self.line_segment.0.x, self.line_segment.0.y],
            [self.line_segment.1.x, self.line_segment.1.y],
        )
    }
}

impl<ID> PointDistance for PathTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let site = Site::new(point[0], point[1]);
        let proj = self.line_segment.get_projection(&site);
        if let Some(proj) = proj {
            let dx = proj.x - site.x;
            let dy = proj.y - site.y;
            dx * dx + dy * dy
        } else {
            let start = &self.line_segment.0;
            let end = &self.line_segment.1;

            let d0 = start.distance(&Site::new(point[0], point[1]));
            let d1 = end.distance(&Site::new(point[0], point[1]));
            d0.min(d1)
        }
    }
}

impl<ID> PartialEq for PathTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ids == other.node_ids || self.node_ids == (other.node_ids.1, other.node_ids.0)
    }
}

#[derive(Debug, Clone)]
pub struct NodeTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    site: Site,
    node_id: ID,
}

impl<ID> NodeTreeObject<ID>
where
    ID: Copy + PartialEq,
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
    ID: Copy + PartialEq,
{
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_point([self.site.x, self.site.y])
    }
}

impl<ID> PointDistance for NodeTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.site.x - point[0];
        let dy = self.site.y - point[1];
        dx * dx + dy * dy
    }
}

impl<ID> PartialEq for NodeTreeObject<ID>
where
    ID: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}
