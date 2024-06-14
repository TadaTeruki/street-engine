pub mod path_checker;
pub mod pathtype;

#[cfg(test)]
mod tests {
    use crate::{
        core::geometry::site::Site,
        transport::{
            checker::{path_checker::PathChecker, pathtype::PathType},
            params::rules::TransportRules,
            traits::{SameRulesProvider, SurfaceTerrain, VoronoiTerrain},
        },
    };

    #[test]
    fn test_path_construction() {
        let rules_provider = SameRulesProvider::new(
            TransportRules::default()
                .path_normal_length(0.4)
                .path_elevation_diff_limit(Some(0.5)),
        );

        let node0 = Site::new(0.0, 0.0);
        let node1 = Site::new(1.0, 1.0);

        // normal path
        {
            let terrain_provider =
                VoronoiTerrain::new(vec![(Site::new(0.0, 0.0), 0.0), (Site::new(1.0, 1.0), 0.0)]);
            let checker = PathChecker::new(&rules_provider, &terrain_provider);

            assert_eq!(
                checker.check_path_construction(node0, node1),
                PathType::Normal
            );
        }

        // normal path with acceptable slope
        {
            let terrain_provider =
                VoronoiTerrain::new(vec![(Site::new(0.0, 0.0), 0.0), (Site::new(1.0, 1.0), 0.5)]);
            let checker = PathChecker::new(&rules_provider, &terrain_provider);

            assert_eq!(
                checker.check_path_construction(node0, node1),
                PathType::Normal
            );
        }

        // normal path with impossible slope
        {
            let terrain_provider =
                VoronoiTerrain::new(vec![(Site::new(0.0, 0.0), 0.0), (Site::new(1.0, 1.0), 1.0)]);
            let checker = PathChecker::new(&rules_provider, &terrain_provider);

            assert_eq!(
                checker.check_path_construction(node0, node1),
                PathType::Impossible
            );
        }

        // bridge path
        {
            let terrain_provider = VoronoiTerrain::new(vec![
                (Site::new(0.0, 0.0), 1.0),
                (Site::new(0.2, 0.2), 0.0),
                (Site::new(0.8, 0.8), 0.0),
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
