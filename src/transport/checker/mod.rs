pub mod path_checker;
pub mod pathtype;

#[cfg(test)]
mod tests {
    use crate::{
        core::geometry::site::Site,
        transport::{
            checker::{path_checker::PathChecker, pathtype::PathType},
            node::transport_node::TransportNode,
            params::{numeric::Stage, rules::GrowthRules},
            traits::{MockVoronoiTerrain, SameRulesProvider},
        },
    };

    #[test]
    fn test_path_construction() {
        let rules_provider = SameRulesProvider::new(
            GrowthRules::default()
                .path_normal_length(0.4)
                .path_elevation_diff_limit(Some(0.5)),
        );

        let node0 = TransportNode::new(Site::new(0.0, 0.0), Stage::from_num(0));
        let node1 = TransportNode::new(Site::new(1.0, 1.0), Stage::from_num(0));

        // normal path
        {
            let terrain_provider = MockVoronoiTerrain::new(vec![
                (Site::new(0.0, 0.0), 0.0),
                (Site::new(1.0, 1.0), 0.5),
            ]);
            let checker = PathChecker::new(&rules_provider, &terrain_provider);

            assert_eq!(
                checker.check_path_construction(node0, node1),
                PathType::Normal
            );
        }

        // normal path with acceptable slope
        {
            let terrain_provider = MockVoronoiTerrain::new(vec![
                (Site::new(0.0, 0.0), 0.0),
                (Site::new(1.0, 1.0), 0.5),
            ]);
            let checker = PathChecker::new(&rules_provider, &terrain_provider);

            assert_eq!(
                checker.check_path_construction(node0, node1),
                PathType::Normal
            );
        }

        // normal path with impossible slope
        {
            let terrain_provider = MockVoronoiTerrain::new(vec![
                (Site::new(0.0, 0.0), 0.8),
                (Site::new(1.0, 1.0), 0.0),
            ]);
            let checker = PathChecker::new(&rules_provider, &terrain_provider);

            assert_eq!(
                checker.check_path_construction(node0, node1),
                PathType::Impossible
            );
        }

        // bridge path
        {
            let terrain_provider = MockVoronoiTerrain::new(vec![
                (Site::new(0.0, 0.0), 1.0),
                (Site::new(0.2, 0.2), 1.1),
                (Site::new(0.8, 0.8), 1.3),
                (Site::new(1.0, 1.0), 1.0),
            ]);
            let checker = PathChecker::new(&rules_provider, &terrain_provider);

            assert_eq!(
                checker.check_path_construction(node0, node1),
                PathType::Bridge
            );
        }
    }
}
