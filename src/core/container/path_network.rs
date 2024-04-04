use std::collections::{BTreeMap, BTreeSet};

use rstar::{PointDistance, RTree, RTreeObject};

use crate::core::geometry::{line_segment::LineSegment, site::Site};

#[derive(Debug, Clone)]
struct PathTreeObject {
    line_segment: LineSegment,
    node_ids: (usize, usize),
}

impl RTreeObject for PathTreeObject {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_corners(
            [self.line_segment.0.x, self.line_segment.0.y],
            [self.line_segment.1.x, self.line_segment.1.y],
        )
    }
}

impl PointDistance for PathTreeObject {
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

#[derive(Debug, Clone)]
struct PathNetwork<N>
where
    N: Eq + Into<Site>,
{
    nodes: BTreeMap<usize, N>,
    path_tree: RTree<PathTreeObject>,
    path_connections: BTreeMap<usize, BTreeSet<usize>>,
    last_node_id: usize,
}
