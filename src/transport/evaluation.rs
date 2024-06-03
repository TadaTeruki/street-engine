use crate::core::{
    geometry::{angle::Angle, site::Site},
    Stage,
};

pub struct PathEvaluationFactors {
    pub site_start: Site,
    pub site_end: Site,
    pub angle: Angle,
    pub path_length: f64,
    pub stage: Stage,
    pub is_bridge: bool,
}
