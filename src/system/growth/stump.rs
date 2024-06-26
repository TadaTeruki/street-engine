use crate::{
    core::{
        container::path_network::NodeId,
        geometry::{angle::Angle, path::bezier::PathBezier},
    },
    system::{
        node::TransportNode,
        path::{factor::PathConstructionFactors, TransportPath},
        rule::TransportRule,
    },
    traits::{PathPrioritizator, TerrainProvider},
    unit::{Elevation, Length},
};

/// Stump is a vector associated with a node which is used to determine the direction of the path to grow.
#[derive(Debug, Clone, PartialEq)]
pub struct Stump {
    /// The node id of the stump.
    node_id: NodeId,
    /// The direction of the stump to grow.
    direction: Angle,
    /// The factors of the path which are referenced when the path is created.
    ///
    /// This value will be moved into the attribute of the path in the path network.
    factor: PathConstructionFactors,
    /// The priority of the stump to grow.
    priority: f64,
    /// The flag to indicate if the path creates a bridge.
    creates_bridge: bool,
}

/// Checks if the elevation difference from a node to a site is not too steep.
fn check_elevation_diff(
    elevation0: Elevation,
    elevation1: Elevation,
    path_length: Length,
    rule: &TransportRule,
) -> bool {
    let allowed_elevation_diff = rule
        .path_slope_elevation_diff_limit
        .value(path_length)
        .value();
    let real_elevation_diff = (elevation0.value() - elevation1.value()).abs();
    real_elevation_diff <= allowed_elevation_diff
}

impl Stump {
    /// Create a new stump and determine the shape of the path to grow (not considering other paths yet).
    pub fn new<TP, PP>(
        node_tuple: (NodeId, TransportNode),
        normal_direction: Angle,
        factor: PathConstructionFactors,
        terrain_provider: &TP,
        path_prioritizator: &PP,
    ) -> Option<Self>
    where
        TP: TerrainProvider,
        PP: PathPrioritizator,
    {
        let (node_id, node) = node_tuple;
        let rule = factor.rule.clone();

        normal_direction
            .iter_range_around(
                rule.path_direction_rule.max_radian,
                rule.path_direction_rule.comparison_step,
            )
            .filter_map(|direction| {
                for i in 0..=rule.bridge_rule.check_step {
                    let bridge_path_length = if rule.bridge_rule.check_step == 0 {
                        Length::new(0.0)
                    } else {
                        Length::new(
                            rule.bridge_rule.max_bridge_length.value() * (i as f64)
                                / (rule.bridge_rule.check_step as f64),
                        )
                    };
                    let path_straight_length = rule.path_normal_length + bridge_path_length;
                    let site_end = node
                        .get_site()
                        .extend(direction, path_straight_length.value());
                    let path = TransportPath::from_curve(PathBezier::from_2d_vectors(
                        node.get_site(),
                        Some((direction, rule.path_normal_length.value())),
                        site_end,
                        None,
                    ));
                    // check terrain
                    let terrain_end = terrain_provider.get_terrain(&site_end);
                    if !terrain_end.0.is_land() {
                        continue;
                    }
                    // check elevation diff
                    if let (Some(elevation_node), Some(elevation_end)) = (
                        terrain_provider.get_terrain(&node.get_site()).1,
                        terrain_end.1,
                    ) {
                        if !check_elevation_diff(
                            elevation_node,
                            elevation_end,
                            path.get_length(),
                            &rule,
                        ) {
                            continue;
                        }
                    } else {
                        continue;
                    }

                    if let Some(priority) = path_prioritizator.prioritize(&node, path) {
                        return Some((priority, direction, i > 0));
                    }
                }
                None
            })
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(priority, direction, creates_bridge)| Self {
                node_id,
                direction,
                factor,
                priority,
                creates_bridge,
            })
    }

    /*
    fn determine_growth(
        &self,
        node_start: &TransportNode,
        encount_nodes: &[EncountNode],
        encount_paths: &[EncountPath],
    ) {

    }*/
}

impl Eq for Stump {}

impl PartialOrd for Stump {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for Stump {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.total_cmp(&other.priority)
    }
}

/*
struct EncountNode {
    node: TransportNode,
    //relation_type: NetworkRelationType,
}

struct EncountPath {
    start: TransportNode,
    end: TransportNode,
    path: PathBezier,
    //relation_type: NetworkRelationType,
}
*/

/*
impl NodeStump {
    /// Create a new node stump.
    pub fn new(node_id: NodeId, angle_expected: Angle, params: StumpParams) -> Self {
        Self {
            node_id,
            angle_expected,
            params,
        }
    }

    /// Get node id
    pub fn get_node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get the end site of the path.
    pub fn angle_expected(&self) -> Angle {
        self.angle_expected
    }

    pub fn get_stump_params(&self) -> &StumpParams {
        &self.params
    }

    /// Get the end site of the path with extra length.
    /// This is temporary used for searching intersections.
    fn get_expected_site_to_with_extra_length(
        &self,
        start_site: Site,
        site_expected_end: Site,
    ) -> Site {
        let path_length = site_expected_end.distance(&start_site);
        let scale =
            (path_length + self.params.rules.path_extra_length_for_intersection) / path_length;
        Site::new(
            start_site.x + (site_expected_end.x - start_site.x) * scale,
            start_site.y + (site_expected_end.y - start_site.y) * scale,
        )
    }

    /// Determine the next node type from related(close) nodes and paths.
    pub fn determine_growth<RP, TP>(
        &self,
        node_start: &TransportNode,
        node_expected_end: &TransportNode,
        related_nodes: &[RelatedNode],
        related_paths: &[(RelatedNode, RelatedNode)],
        terrain_provider: &TP,
        path_checker: PathChecker<RP, TP>,
    ) -> GrowthType
    where
        RP: GrowthRulesProvider,
        TP: TerrainProvider,
    {
        let search_start = node_start.site;

        // Existing Node
        // For this situation, path crosses are needed to be checked again because the direction of the path can be changed from original.
        {
            let existing_node_id = related_nodes
                .iter()
                .filter(|existing| {
                    // distance check for decreasing the number of candidates
                    LineSegment::new(search_start, node_expected_end.site)
                        .get_distance(&existing.node.site)
                        < self.params.rules.path_extra_length_for_intersection
                })
                .filter(|existing| {
                    // no intersection check
                    let has_intersection = related_paths.iter().any(|(path_start, path_end)| {
                        if existing.node_id == path_start.node_id
                            || existing.node_id == path_end.node_id
                        {
                            // ignore
                            return false;
                        }
                        let path_line = LineSegment::new(path_start.node.site, path_end.node.site);
                        let search_line = LineSegment::new(search_start, existing.node.site);
                        path_line.get_intersection(&search_line).is_some()
                    });
                    !has_intersection
                })
                .filter_map(|existing| {
                    // if the elevation difference is too large, the path cannot be connected.
                    let distance = existing.node.site.distance(&search_start);
                    self.params
                        .rules
                        .check_elevation_diff_to_create_path_on_land(
                            terrain_provider.get_elevation(&search_start)?,
                            terrain_provider.get_elevation(&existing.node.site)?,
                            distance,
                        )
                        .then_some(existing)
                })
                .min_by(|a, b| {
                    let distance_a = a.node.site.distance_2(&search_start);
                    let distance_b = b.node.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some(existing) = existing_node_id {
                return GrowthType::Existing(existing.node_id);
            }
        }

        // Crossing Paths
        {
            let search_end = self
                .get_expected_site_to_with_extra_length(node_start.site, node_expected_end.site);
            let search_line = LineSegment::new(search_start, search_end);

            let crossing_path = related_paths
                .iter()
                .filter_map(|(path_start, path_end)| {
                    let path_line = LineSegment::new(path_start.node.site, path_end.node.site);

                    if let Some(intersect) = path_line.get_intersection(&search_line) {
                        return Some((
                            TransportNode::new(
                                intersect,
                                path_start.node.path_stage(path_end.node),
                            ),
                            (path_start, path_end),
                        ));
                    }
                    None
                })
                .filter_map(|(crossing_node, path)| {
                    // calculate elevation of crossing_node
                    let elevation_crossing = {
                        let distance_0 = crossing_node.site.distance(&path.0.node.site);
                        let distance_1 = crossing_node.site.distance(&path.1.node.site);
                        let proportion_of_0 = distance_1 / (distance_0 + distance_1);
                        terrain_provider.get_elevation(&path.0.node.site)? * proportion_of_0
                            + terrain_provider.get_elevation(&path.1.node.site)?
                                * (1.0 - proportion_of_0)
                    };

                    // slope check
                    // if the elevation difference is too large, the path cannot be connected.
                    let distance = crossing_node.site.distance(&search_start);

                    self.params
                        .rules
                        .check_elevation_diff_to_create_path_on_land(
                            terrain_provider.get_elevation(&search_start)?,
                            elevation_crossing,
                            distance,
                        )
                        .then_some((crossing_node, path))
                })
                .min_by(|a, b| {
                    let distance_a = a.0.site.distance_2(&search_start);
                    let distance_b = b.0.site.distance_2(&search_start);
                    distance_a.total_cmp(&distance_b)
                });

            if let Some((crossing_node, (path_start, path_end))) = crossing_path {
                // if it cross the bridge, the path cannot be connected.
                let path_type = path_checker.check_path_type(*path_start.node, *path_end.node);
                if !path_type.can_create_intersection() {
                    return GrowthType::None;
                }

                return GrowthType::Intersect(
                    crossing_node,
                    (path_start.node_id, path_end.node_id),
                );
            }
        }

        // check slope
        let distance = search_start.distance(&node_expected_end.site);
        let slope_ok = {
            if let (Some(elevation_start), Some(elevation_end)) = (
                terrain_provider.get_elevation(&search_start),
                terrain_provider.get_elevation(&node_expected_end.site),
            ) {
                self.params
                    .rules
                    .check_elevation_diff_to_create_path_on_land(
                        elevation_start,
                        elevation_end,
                        distance,
                    )
            } else {
                false
            }
        };

        if !slope_ok {
            return GrowthType::None;
        }

        // New Node
        // Path crosses are already checked in the previous steps.
        GrowthType::New(*node_expected_end)
    }
}

 */
