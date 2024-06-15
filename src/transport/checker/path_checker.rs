use std::marker::PhantomData;

use crate::{
    core::geometry::{angle::Angle, site::Site},
    transport::{params::rules::TransportRules, traits::{TerrainProvider, TransportRulesProvider}},
};

use super::pathtype::PathType;

pub struct PathChecker<'a, N, RP, TP>
where
    N: Into<Site> + Copy,
    RP: TransportRulesProvider,
    TP: TerrainProvider,
{
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
    _phantom: PhantomData<N>,
}

struct CheckPathNode<'a> {
    site: Site,
    rules: &'a TransportRules,
    elevation: f64,
}

impl<N, RP, TP> PathChecker<'_, N, RP, TP>
where
    N: Into<Site> + Copy,
    RP: TransportRulesProvider,
    TP: TerrainProvider,
{
    pub fn new<'a>(rules_provider: &'a RP, terrain_provider: &'a TP) -> PathChecker<'a, N, RP, TP> {
        PathChecker {
            rules_provider,
            terrain_provider,
            _phantom: PhantomData,
        }
    }

    fn check_elevation_diff(&self, node0: CheckPathNode, node1: CheckPathNode) -> bool {
        let allowed_elevation_diff = if let Some(max_elevation_diff) = node0.rules.path_elevation_diff_limit {
            let distance = node0.site.distance(&node1.site);
            max_elevation_diff * distance
        } else {
            // always allowed
            return true;
        };

        let real_elevation_diff = (node1.elevation - node0.elevation).abs();
        real_elevation_diff <= allowed_elevation_diff
    }

    fn node_constructs_bridge(&self, node: CheckPathNode, angle_to: Angle) -> bool {
        false
    }

    fn path_is_bridge(&self, node0: CheckPathNode, node1: CheckPathNode) -> bool {
        let angle_from_node0_to_node1 = node0.site.get_angle(&node1.site);
        self.node_constructs_bridge(node0, angle_from_node0_to_node1)
            || self.node_constructs_bridge(node1, angle_from_node0_to_node1.opposite())
    }

    pub fn check_path_construction(&self, node0: N, node1: N) -> PathType {
        let angle = (node0.into() as Site).get_angle(&(node1.into() as Site));
        let node0 = CheckPathNode {
            site: node0.into(),
            rules: self.rules_provider.get_rules(&node0.into(), , 0, &Default::default()).unwrap(),
            elevation: self.terrain_provider.get_elevation(&node0.into()).unwrap(),
        };
        
        PathType::Impossible
    }
}

/*
   pub fn check_elevation_diff_to_create_path_on_land(
       &self,
       elevation_start: f64,
       elevation_end: f64,
       distance: f64,
   ) -> bool {
       let allowed_elevation_diff =
           if let Some(max_elevation_diff) = self.path_elevation_diff_limit {
               max_elevation_diff
           } else {
               // always allowed
               return true;
           };
       let real_elevation_diff = (elevation_end - elevation_start).abs();

       real_elevation_diff <= allowed_elevation_diff * distance
   }
*/
