pub mod candidate;
pub mod node;

#[cfg(test)]
mod tests {
    use crate::{
        core::{
            container::path_network::NodeId,
            geometry::{angle::Angle, site::Site},
            Stage,
        },
        transport::rules::{BranchRules, BridgeRules, PathDirectionRules, TransportRules},
    };

    use super::{
        candidate::{NextTransportNode, PathCandidate},
        node::TransportNode,
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
            is_bridge: TransportNode::default().is_bridge,
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
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 1.0,
            path_extra_length_for_intersection: 0.25,
            path_max_elevation_diff: None,
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

        let static_stage = Stage::default();

        // New node
        let new = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            static_stage,
            rules.clone(),
        )
        .determine_next_node(
            site_expected_end,
            static_stage,
            false,
            &nodes_parsed,
            &paths_parsed,
        );

        if let (NextTransportNode::New(node), _) = new {
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
        let intersect = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            static_stage,
            rules.clone(),
        )
        .determine_next_node(
            site_expected_end,
            static_stage,
            false,
            &nodes_parsed,
            &paths_parsed,
        );

        if let (NextTransportNode::Intersect(node, _), _) = intersect {
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
        let existing = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            static_stage,
            rules.clone(),
        )
        .determine_next_node(
            site_expected_end,
            static_stage,
            false,
            &nodes_parsed,
            &paths_parsed,
        );

        if let (NextTransportNode::Existing(node_id), _) = existing {
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
        let existing = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            static_stage,
            rules.clone(),
        )
        .determine_next_node(
            site_expected_end,
            static_stage,
            false,
            &nodes_parsed,
            &paths_parsed,
        );

        if let (NextTransportNode::Existing(node_id), _) = existing {
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
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 10000.0,
            path_extra_length_for_intersection: 0.0,
            path_max_elevation_diff: None,
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

        let static_stage = Stage::default();

        let next = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            static_stage,
            rules.clone(),
        )
        .determine_next_node(
            site_expected_end,
            static_stage,
            false,
            &nodes_parsed,
            &paths_parsed,
        );

        println!("{:?}", next.0);

        assert!(matches!(next.0, NextTransportNode::Intersect(_, _)));
        if let (NextTransportNode::Intersect(node, _), _) = next {
            assert!(
                (node.site.x >= 0.0 && node.site.x <= 0.3)
                    && (node.site.y >= 0.0 && node.site.y <= 5.0)
            );
        } else {
            panic!("Unexpected node type");
        }
    }

    /*
    #[test]
    fn test_bridge() {
        let nodes = vec![
            create_node(0.0, 0.0),
            create_node(1.0, 1.0),
            create_node(0.0, 0.0),
            create_node(1.0, 1.0),
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

        let default_rules = TransportRules {
            path_priority: 0.0,
            elevation: 0.0,
            population_density: 0.0,
            path_normal_length: 1.0,
            path_extra_length_for_intersection: 0.25,
            path_max_elevation_diff: None,
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

        let static_stage = Stage::default();

        // New node
        let new = PathCandidate::new(
            node_start,
            NodeId::new(10000),
            angle_expected_end,
            static_stage,
            rules.clone(),
        )
        .determine_next_node(
            site_expected_end,
            rules.clone(),
            static_stage,
            &nodes_parsed,
            &paths_parsed,
        );

        if let NextTransportNode::New(node) = new {
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
    }
    */
}
