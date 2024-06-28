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
        transport::params::{
            metrics::PathMetrics,
            numeric::Stage,
            rules::{BranchRules, BridgeRules, PathDirectionRules, TransportRules},
            PathParams,
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
            elevation: TransportNode::default().elevation,
            stage: TransportNode::default().stage,
            is_bridge: TransportNode::default().is_bridge,
        }
    }

    fn create_node_detailed(x: f64, y: f64, elevation: f64, is_bridge: bool) -> TransportNode {
        TransportNode {
            site: Site::new(x, y),
            elevation,
            stage: TransportNode::default().stage,
            is_bridge,
        }
    }

    #[test]
    fn test_next_node() {
        let nodes = vec![
            create_node(3.0, 0.0),
            create_node(1.0, 0.0),
            create_node(0.0, 1.0),
            create_node(0.0, 3.0),
        ];

        let nodes_parsed = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node, NodeId::new(i)))
            .collect::<Vec<_>>();

        let paths = vec![(0, 1), (1, 2), (2, 3)];

        let paths_parsed = paths
            .iter()
            .map(|(start, end)| (nodes_parsed[*start], nodes_parsed[*end]))
            .collect::<Vec<_>>();

        let rules = TransportRules {
            path_normal_length: 1.0,
            path_extra_length_for_intersection: 0.25,
            path_elevation_diff_limit: None,
            branch_rules: BranchRules::default(),
            path_direction_rules: PathDirectionRules::default(),
            bridge_rules: BridgeRules::default(),
        };

        let (node_start, angle_expected_end) = (
            create_node(1.0, 1.0),
            Angle::new(std::f64::consts::PI * 0.75),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);

        let params = PathParams {
            stage: Stage::default(),
            rules_start: rules.clone(),
            metrics: PathMetrics::default(),
            priority: 0.0,
        };

        // New node
        let new = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &TransportNode {
                    site: site_expected_end,
                    elevation: 0.0,
                    stage: params.stage,
                    is_bridge: false,
                },
                &nodes_parsed,
                &paths_parsed,
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
        let (node_start, angle_expected_end) = (
            create_node(1.0, 1.0),
            Angle::new(-std::f64::consts::PI * 0.25),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        let intersect = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &TransportNode {
                    site: site_expected_end,
                    elevation: 0.0,
                    stage: params.stage,
                    is_bridge: false,
                },
                &nodes_parsed,
                &paths_parsed,
            );

        if let NextNodeType::Intersect(node, _) = intersect.next_node {
            assert_eq_f64!(node.site.distance(&Site::new(0.5, 0.5)), 0.0);
        } else {
            panic!("Unexpected node type");
        }

        // Existing node (close between two nodes)
        let (node_start, angle_expected_end) = (
            create_node(1.0, 1.0),
            Angle::new(std::f64::consts::PI * 0.05),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        let existing = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &TransportNode {
                    site: site_expected_end,
                    elevation: 0.0,
                    stage: params.stage,
                    is_bridge: false,
                },
                &nodes_parsed,
                &paths_parsed,
            );

        if let NextNodeType::Existing(node_id) = existing.next_node {
            assert_eq!(node_id, NodeId::new(1));
        } else {
            panic!("Unexpected node type");
        }

        // Existing node (close between an existing node and expected path)
        let (node_start, angle_expected_end) = (
            create_node(1.0, 0.5),
            Angle::new(std::f64::consts::PI * 0.05),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);
        let existing = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &TransportNode {
                    site: site_expected_end,
                    elevation: 0.0,
                    stage: params.stage,
                    is_bridge: false,
                },
                &nodes_parsed,
                &paths_parsed,
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

        let nodes_parsed = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node, NodeId::new(i)))
            .collect::<Vec<_>>();

        let paths = vec![(0, 5), (5, 2), (2, 7), (7, 3), (3, 6), (6, 1), (1, 4)];

        let paths_parsed = paths
            .iter()
            .map(|(start, end)| (nodes_parsed[*start], nodes_parsed[*end]))
            .collect::<Vec<_>>();

        let rules = TransportRules {
            path_normal_length: 10000.0,
            path_extra_length_for_intersection: 0.0,
            path_elevation_diff_limit: None,
            branch_rules: BranchRules::default(),
            path_direction_rules: PathDirectionRules::default(),
            bridge_rules: BridgeRules::default(),
        };

        let (node_start, angle_expected_end) = (
            create_node(-1.0, 1.0),
            Angle::new(std::f64::consts::PI * 0.5),
        );
        let site_expected_end = node_start
            .site
            .extend(angle_expected_end, rules.path_normal_length);

        let params = PathParams {
            stage: Stage::default(),
            rules_start: rules.clone(),
            metrics: PathMetrics::default(),
            priority: 0.0,
        };

        let next = NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone())
            .determine_growth(
                &node_start,
                &TransportNode {
                    site: site_expected_end,
                    elevation: 0.0,
                    stage: params.stage,
                    is_bridge: false,
                },
                &nodes_parsed,
                &paths_parsed,
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

    #[test]
    fn test_bridge() {
        let nodes = vec![
            create_node_detailed(0.0, 0.0, 0.0, false),
            create_node_detailed(1.0, 1.0, 0.0, false),
            create_node_detailed(0.0, 0.0, 1.0, true),
            create_node_detailed(1.0, 1.0, 1.0, true),
        ];

        let nodes_parsed = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node, NodeId::new(i)))
            .collect::<Vec<_>>();

        let paths = vec![(0, 1), (2, 3)];

        let paths_parsed = paths
            .iter()
            .map(|(start, end)| (nodes_parsed[*start], nodes_parsed[*end]))
            .collect::<Vec<_>>();

        let rules = TransportRules {
            path_normal_length: 2.0_f64.sqrt(),
            path_extra_length_for_intersection: 0.25,
            path_elevation_diff_limit: Some(0.7),
            branch_rules: BranchRules::default(),
            path_direction_rules: PathDirectionRules::default(),
            bridge_rules: BridgeRules::default(),
        };

        let check = |elevation_start: f64, elevation_end: f64| -> GrowthTypes {
            let (node_start, angle_expected_end) = (
                create_node_detailed(0.0, 1.0, elevation_start, false),
                Angle::new(std::f64::consts::PI * 0.25),
            );
            let site_expected_end = node_start
                .site
                .extend(angle_expected_end, rules.path_normal_length);

            let params = PathParams {
                stage: Stage::default(),
                rules_start: rules.clone(),
                metrics: PathMetrics::default(),
                priority: 0.0,
            };

            NodeStump::new(NodeId::new(10000), angle_expected_end, params.clone()).determine_growth(
                &node_start,
                &TransportNode {
                    site: site_expected_end,
                    elevation: elevation_end,
                    stage: params.stage,
                    is_bridge: false,
                },
                &nodes_parsed,
                &paths_parsed,
            )
        };

        // New node which passes between two existing paths
        let new = check(0.5, 0.5);

        if let (NextNodeType::New(node), is_bridge) = (new.next_node, new.bridge_node) {
            assert_eq_f64!(node.site.distance(&Site::new(1.0, 0.0)), 0.0);
            assert!(is_bridge.is_none());
        } else {
            panic!("Unexpected node type");
        }

        // Connect to the existing path (land)
        let land = check(0.2, 0.2);

        if let (NextNodeType::Intersect(node, _), is_bridge) = (land.next_node, land.bridge_node) {
            assert_eq_f64!(node.site.distance(&Site::new(0.5, 0.5)), 0.0);
            assert!(is_bridge.is_none());
        } else {
            panic!("Unexpected node type");
        }

        // Connect to the existing path (bridge)
        // This connection will be ignored because creating intersection on bridge is not allowed.
        let bridge = check(0.8, 0.8);

        if let (NextNodeType::None, is_bridge) = (bridge.next_node, bridge.bridge_node) {
            assert!(is_bridge.is_none());
        } else {
            panic!("Unexpected node type");
        }
    }
}
