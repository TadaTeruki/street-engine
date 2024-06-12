pub mod growth_type;
pub mod node_stump;
pub mod transport_node;

#[cfg(test)]
mod tests {
    use crate::{
        core::{
            container::path_network::NodeId,
            geometry::{angle::Angle, site::Site},
        },
        transport::{
            params::{
                rules::{BridgeRules, TransportRules},
                PathParams,
            },
            path_network_repository::{PathNetworkGroup, PathNetworkId, RelatedNode},
            traits::TerrainProvider,
        },
    };

    use super::{
        growth_type::{GrowthTypes, NextNodeType},
        node_stump::NodeStump,
        transport_node::TransportNode,
    };

    macro_rules! assert_eq_f64 {
        ($a:expr, $b:expr) => {
            assert!(($a - $b).abs() < 1e-6);
        };
    }

    fn create_node(x: f64, y: f64) -> TransportNode {
        TransportNode {
            site: Site::new(x, y),
            stage: TransportNode::default().stage,
            creates_bridge: TransportNode::default().creates_bridge,
        }
    }

    fn create_node_detailed(x: f64, y: f64, creates_bridge: bool) -> TransportNode {
        TransportNode {
            site: Site::new(x, y),
            stage: TransportNode::default().stage,
            creates_bridge,
        }
    }

    struct SurfaceTerrain;

    impl TerrainProvider for SurfaceTerrain {
        fn get_elevation(&self, _site: &Site) -> Option<f64> {
            Some(0.0)
        }
    }

    macro_rules! bind_nodes {
        ($nodes:expr) => {
            $nodes
                .iter()
                .enumerate()
                .map(|(i, node)| RelatedNode {
                    node,
                    node_id: NodeId::new(i),
                    network_id: PathNetworkId::new(0),
                    group: PathNetworkGroup::new(0),
                })
                .collect::<Vec<_>>()
        };
    }
    macro_rules! bind_paths {
        ($paths:expr, $nodes:expr) => {
            $paths
                .iter()
                .map(|(start, end)| ($nodes[*start], $nodes[*end]))
                .collect::<Vec<_>>()
        };
    }

    fn create_node_start_end(
        site_start: Site,
        angle: Angle,
        params: PathParams,
        creates_bridge: bool,
    ) -> (TransportNode, TransportNode) {
        let node_start = TransportNode {
            site: site_start,
            stage: params.stage,
            creates_bridge,
        };
        let site_end = site_start.extend(angle, params.rules_start.path_normal_length);
        let node_end = TransportNode {
            site: site_end,
            stage: params.stage,
            creates_bridge,
        };

        (node_start, node_end)
    }

    #[test]
    fn test_next_node() {
        let related_nodes = vec![
            create_node(3.0, 0.0),
            create_node(1.0, 0.0),
            create_node(0.0, 1.0),
            create_node(0.0, 3.0),
        ];
        let related_nodes = bind_nodes!(&related_nodes);
        let related_paths = bind_paths!(&[(0, 1), (1, 2), (2, 3)], &related_nodes);

        let rules = TransportRules::default()
            .path_normal_length(1.0)
            .path_extra_length_for_intersection(0.25);

        let params = PathParams::default().rules_start(rules);
        let angle_expected_end = Angle::new(std::f64::consts::PI * 0.75);

        // New node
        let (node_start, node_expected_end) = create_node_start_end(
            Site::new(1.0, 1.0),
            angle_expected_end,
            params.clone(),
            false,
        );
        let new = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &node_expected_end,
                &related_nodes,
                &related_paths,
                &SurfaceTerrain,
            );

        if let NextNodeType::New(node) = new.next_node {
            assert_eq_f64!(
                node.site.distance(&Site::new(
                    1.0 + 1.0 / 2.0_f64.sqrt(),
                    1.0 + 1.0 / 2.0_f64.sqrt()
                )),
                0.0
            );
        } else {
            panic!("Unexpected node type");
        }

        // Intersect (Crossing Path)
        let angle_expected_end = Angle::new(-std::f64::consts::PI * 0.25);
        let (node_start, node_expected_end) = create_node_start_end(
            Site::new(1.0, 1.0),
            angle_expected_end,
            params.clone(),
            false,
        );
        let intersect = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &node_expected_end,
                &related_nodes,
                &related_paths,
                &SurfaceTerrain,
            );

        if let NextNodeType::Intersect(node, _) = intersect.next_node {
            assert_eq_f64!(node.site.distance(&Site::new(0.5, 0.5)), 0.0);
        } else {
            panic!("Unexpected node type");
        }

        // Existing node (close between two nodes)
        let angle_expected_end = Angle::new(std::f64::consts::PI * 0.05);
        let (node_start, node_expected_end) = create_node_start_end(
            Site::new(1.0, 1.0),
            angle_expected_end,
            params.clone(),
            false,
        );
        let existing = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &node_expected_end,
                &related_nodes,
                &related_paths,
                &SurfaceTerrain,
            );

        if let NextNodeType::Existing(node_id) = existing.next_node {
            assert_eq!(node_id, NodeId::new(1));
        } else {
            panic!("Unexpected node type");
        }

        // Existing node (close between an existing node and expected path)
        let angle_expected_end = Angle::new(std::f64::consts::PI * 0.05);
        let (node_start, node_expected_end) = create_node_start_end(
            Site::new(1.0, 0.5),
            angle_expected_end,
            params.clone(),
            false,
        );
        let existing = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &node_expected_end,
                &related_nodes,
                &related_paths,
                &SurfaceTerrain,
            );

        if let NextNodeType::Existing(node_id) = existing.next_node {
            assert_eq!(node_id, NodeId::new(1));
        } else {
            panic!("Unexpected node type");
        }
    }

    #[test]
    fn test_next_node_across_multiple_paths() {
        let nodes = vec![
            create_node(0.0, 0.0),
            create_node(0.3, 0.0),
            create_node(0.7, 0.0),
            create_node(1.0, 0.0),
            create_node(0.0, 10.0),
            create_node(0.3, 10.0),
            create_node(0.7, 10.0),
            create_node(1.0, 10.0),
        ];
        let nodes = bind_nodes!(&nodes);
        let paths = bind_paths!(
            &vec![(0, 5), (5, 2), (2, 7), (7, 3), (3, 6), (6, 1), (1, 4)],
            &nodes
        );

        let rules = TransportRules::default().path_normal_length(10000.0);
        let params = PathParams::default().rules_start(rules);
        let angle_expected_end = Angle::new(std::f64::consts::PI * 0.5);
        let (node_start, node_expected_end) = create_node_start_end(
            Site::new(-1.0, 1.0),
            angle_expected_end,
            params.clone(),
            false,
        );

        let next = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &node_expected_end,
                &nodes,
                &paths,
                &SurfaceTerrain,
            );

        println!("{:?}", next.next_node);

        assert!(matches!(next.next_node, NextNodeType::Intersect(_, _)));
        if let NextNodeType::Intersect(node, _) = next.next_node {
            assert!(
                (node.site.x >= 0.0 && node.site.x <= 0.3)
                    && (node.site.y >= 0.0 && node.site.y <= 5.0)
            );
        } else {
            panic!("Unexpected node type");
        }
    }

    struct SpotTerrain {
        spots: Vec<(Site, f64)>,
    }

    impl SpotTerrain {
        fn new(spots: Vec<(Site, f64)>) -> Self {
            Self { spots }
        }
    }

    impl TerrainProvider for SpotTerrain {
        fn get_elevation(&self, site: &Site) -> Option<f64> {
            self.spots
                .iter()
                .map(|(spot, elevation)| (spot.distance(site), elevation))
                .min_by(|(distance1, _), (distance2, _)| distance1.total_cmp(distance2))
                .map(|(_, elevation)| *elevation)
        }
    }

    #[test]
    fn test_bridge() {
        let situation =
            |path_elevation: f64, start_elevation: f64, path_is_bridge: bool| -> GrowthTypes {
                let related_nodes = vec![
                    create_node_detailed(0.0, 0.0, path_is_bridge),
                    create_node_detailed(1.0, 1.0, path_is_bridge),
                ];
                let related_nodes = bind_nodes!(&related_nodes);
                let related_paths = bind_paths!(&vec![(0, 1)], &related_nodes);

                let rules = TransportRules::default()
                    .path_normal_length(2.0_f64.sqrt())
                    .path_extra_length_for_intersection(0.25)
                    .path_elevation_diff_limit(Some(0.7));

                let params = PathParams::default().rules_start(rules);
                let angle_expected_end = Angle::new(std::f64::consts::PI * 0.25);
                let (node_start, node_expected_end) = create_node_start_end(
                    Site::new(0.0, 1.0),
                    angle_expected_end,
                    params.clone(),
                    false,
                );

                let terrain = SpotTerrain::new(vec![
                    (Site::new(0.0, 0.0), path_elevation),
                    (Site::new(1.0, 1.0), path_elevation),
                    (Site::new(0.0, 1.0), start_elevation),
                    (Site::new(1.0, 0.0), start_elevation),
                ]);

                NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
                    .determine_growth(
                        &node_start,
                        &node_expected_end,
                        &related_nodes,
                        &related_paths,
                        &terrain,
                    )
            };

        // --- on land ---

        // New node which passes between two existing paths
        let new = situation(0.0, 0.5, false);
        if let NextNodeType::New(node) = new.next_node {
            assert_eq_f64!(node.site.distance(&Site::new(1.0, 0.0)), 0.0);
        } else {
            panic!("Unexpected node type");
        }

        // Connect to the existing path (land)
        let intersect_on_land = situation(0.0, 0.2, false);
        if let NextNodeType::Intersect(node, _) = intersect_on_land.next_node {
            assert_eq_f64!(node.site.distance(&Site::new(0.5, 0.5)), 0.0);
        } else {
            panic!("Unexpected node type");
        }

        // --- across a bridge ---

        // New node which passes between two existing paths
        let new = situation(1.0, 0.5, true);
        if let NextNodeType::New(node) = new.next_node {
            assert_eq_f64!(node.site.distance(&Site::new(1.0, 0.0)), 0.0);
        } else {
            panic!("Unexpected node type");
        }

        // Connect to the existing path (bridge)
        // This connection will be ignored because creating intersection on bridge is not allowed.
        let intersect_on_bridge = situation(1.0, 0.8, true);
        assert!(matches!(intersect_on_bridge.next_node, NextNodeType::None));
    }

    #[test]
    fn test_custom_pattern_0() {
        let related_path_nodes = vec![
            create_node_detailed(-87.21831510702368, 1.140441704558744, true),
            create_node_detailed(-94.23498098118608, 1.1437765922601952, true),
        ];

        let related_nodes = bind_nodes!(&related_path_nodes);
        let related_paths = bind_paths!(&vec![(0, 1)], &related_nodes);

        let rules = TransportRules::default()
            .path_normal_length(0.45)
            .path_extra_length_for_intersection(0.315)
            .path_elevation_diff_limit(Some(5.0))
            .bridge_rules(BridgeRules {
                max_bridge_length: 25.0,
                check_step: 15,
            });

        let params = PathParams::default().rules_start(rules);

        let incoming_nodes = vec![
            create_node_detailed(-89.26258026851414, 4.955678286720349, false),
            create_node_detailed(-87.9230415089059, -5.408111363379038, true),
        ];

        let angle_expected_end = incoming_nodes[0].site.get_angle(&incoming_nodes[1].site);

        let stump = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone());

        let terrain = SpotTerrain::new(vec![
            (related_path_nodes[0].site, 0.11278313501817303),
            (related_path_nodes[1].site, 0.4537962059055101),
            (incoming_nodes[0].site, 0.26211488472154043),
            (incoming_nodes[1].site, 1.0838784525661787),
        ]);

        let next = stump.determine_growth(
            &incoming_nodes[0],
            &incoming_nodes[1],
            &vec![],
            &related_paths,
            &terrain,
        );

        assert!(matches!(next.next_node, NextNodeType::None));
    }
}
