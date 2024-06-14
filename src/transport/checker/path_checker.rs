use crate::{
    core::geometry::{angle::Angle, site::Site},
    transport::traits::{TerrainProvider, TransportRulesProvider},
};

use super::pathtype::PathType;

pub struct PathChecker<'a, RP, TP>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
{
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
}

impl<RP, TP> PathChecker<'_, RP, TP>
where
    RP: TransportRulesProvider,
    TP: TerrainProvider,
{
    pub fn new<'a>(rules_provider: &'a RP, terrain_provider: &'a TP) -> PathChecker<'a, RP, TP> {
        PathChecker {
            rules_provider,
            terrain_provider,
        }
    }

    fn node_constructs_bridge<N>(&self, node0: N, angle_to: Angle) -> bool
    where
        N: Into<Site>,
    {
        false
    }

    fn path_is_bridge<N>(&self, node0: N, node1: N) -> bool
    where
        N: Into<Site> + Copy,
    {
        let angle_from_node0_to_node1 = (node0.into() as Site).get_angle(&node1.into());
        self.node_constructs_bridge(node0, angle_from_node0_to_node1)
            || self.node_constructs_bridge(node1, angle_from_node0_to_node1)
    }

    pub fn check_path_construction<N>(&self, node0: N, node1: N) -> PathType
    where
        N: Into<Site>,
    {
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
