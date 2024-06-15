use crate::{
    core::geometry::site::Site,
    transport::{
        node::transport_node::TransportNode,
        params::rules::GrowthRules,
        traits::{GrowthRulesProvider, TerrainProvider},
    },
};

use super::pathtype::PathType;

/// A struct to check if a path can be constructed between two nodes and what type of path it is.
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

    /// Checks if the elevation difference from a node to a site is not too steep.
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

    /// Checks if the elevation difference between the two nodes is not too steep to construct a path.
    fn check_elevation_diff(&self, node0: &CheckNode, node1: &CheckNode) -> bool {
        self.elevation_diff_is_proper(&node0, node1.site, node1.elevation)
            && self.elevation_diff_is_proper(&node1, node0.site, node0.elevation)
    }

    /// Checks if a node constructs a bridge.
    fn node_constructs_bridge(&self, node: &CheckNode, site_to: Site) -> bool {
        if node.site.distance_2(&site_to) < node.rules.path_normal_length.powi(2) {
            return false;
        }

        let angle_to = node.site.get_angle(&site_to);
        let normal_node_to = node.site.extend(angle_to, node.rules.path_normal_length);
        if let Some(elevation_node_to) = self.terrain_provider.get_elevation(&normal_node_to) {
            // constructs bridge is the elevation is not proper (=too steep)
            !self.elevation_diff_is_proper(&node, normal_node_to, elevation_node_to)
        } else {
            // constructs bridge if the elevation is not available
            true
        }
    }

    /// Checks if the path between two nodes is a bridge.
    fn check_path_is_bridge(&self, node0: &CheckNode, node1: &CheckNode) -> bool {
        self.node_constructs_bridge(node0, node0.site)
            || self.node_constructs_bridge(node1, node1.site)
    }

    /// Creates a temporary node to check the path construction.
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

    /// Checks if a path can be constructed between two nodes.
    /// If it is possible, returns the type of the path.
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
