use crate::unit::Length;

/// Rules to make other paths avoid the path as an obstacle.
///
/// This parameter is particularly useful for treating rivers as obstacles.
#[derive(Debug, Clone, PartialEq)]
pub struct PathAvoidanceRule {
    /// The radius of the path as an obstacle.
    pub path_radius_as_obstacle: Length,
}
