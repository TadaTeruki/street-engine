use std::marker::PhantomData;

use crate::{
    core::geometry::{angle::Angle, site::Site},
    transport::{
        node::transport_node::TransportNode,
        params::rules::GrowthRules,
        traits::{GrowthRulesProvider, TerrainProvider},
    },
};

use super::pathtype::PathType;

pub struct PathChecker<'a, RP, TP>
where
    RP: GrowthRulesProvider,
    TP: TerrainProvider,
{
    rules_provider: &'a RP,
    terrain_provider: &'a TP,
}

/// Temporary struct to hold the data needed for path checking.
struct CheckNode {
    site: Site,
    rules: GrowthRules,
    elevation: f64,
}

impl<RP, TP> PathChecker<'_, RP, TP>
where
    RP: GrowthRulesProvider,
    TP: TerrainProvider,
{
    pub fn new<'a>(rules_provider: &'a RP, terrain_provider: &'a TP) -> PathChecker<'a, RP, TP> {
        PathChecker {
            rules_provider,
            terrain_provider,
        }
    }

    fn elevation_diff_is_proper(
        &self,
        node_from: &CheckNode,
        site_node_to: Site,
        elevation_node_to: f64,
    ) -> bool {
        let allowed_elevation_diff =
            if let Some(max_elevation_diff) = node_from.rules.path_elevation_diff_limit {
                let distance = node_from.site.distance(&site_node_to);
                max_elevation_diff * distance
            } else {
                // always allowed
                return true;
            };

        let real_elevation_diff = (node_from.elevation - elevation_node_to).abs();
        real_elevation_diff <= allowed_elevation_diff
    }

    fn check_elevation_diff(&self, node0: &CheckNode, node1: &CheckNode) -> bool {
        self.elevation_diff_is_proper(&node0, node1.site, node1.elevation)
            && self.elevation_diff_is_proper(&node1, node0.site, node0.elevation)
    }

    fn node_constructs_bridge(&self, node: &CheckNode, angle_to: Angle) -> bool {
        let normal_node_to = node.site.extend(angle_to, node.rules.path_normal_length);
        if let Some(elevation_node_to) = self.terrain_provider.get_elevation(&normal_node_to) {
            self.elevation_diff_is_proper(&node, normal_node_to, elevation_node_to)
        } else {
            // constructs bridge if the elevation is not available
            true
        }
    }

    fn check_path_is_bridge(&self, node0: &CheckNode, node1: &CheckNode) -> bool {
        let angle_from_node0_to_node1 = node0.site.get_angle(&node1.site);
        self.node_constructs_bridge(node0, angle_from_node0_to_node1)
            || self.node_constructs_bridge(node1, angle_from_node0_to_node1.opposite())
    }

    fn create_checknode(&self, node: TransportNode, other: TransportNode) -> Option<CheckNode> {
        let rules = self
            .rules_provider
            .get_rules(&node.into(), node.path_stage(&other))?;
        let elevation = self.terrain_provider.get_elevation(&node.into())?;
        Some(CheckNode {
            site: node.into(),
            rules,
            elevation,
        })
    }

    pub fn check_path_construction(&self, node0: TransportNode, node1: TransportNode) -> PathType {
        let (node0, node1) = if let (Some(node0), Some(node1)) = (
            self.create_checknode(node0, node1),
            self.create_checknode(node1, node0),
        ) {
            (node0, node1)
        } else {
            return PathType::Impossible;
        };

        if !self.check_elevation_diff(&node0, &node1) {
            return PathType::Impossible;
        }

        if self.check_path_is_bridge(&node0, &node1) {
            PathType::Bridge
        } else {
            PathType::Normal
        }
    }
}
