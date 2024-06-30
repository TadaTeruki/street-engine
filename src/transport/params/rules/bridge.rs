/// Rules to create bridges.
///
/// Bridge is the path that connect two distant sites where the normal path cannot be constructed.
/// For this package, the meaning of bridges includes not only the bridges over rivers or valleys but also tunnels under mountains.
///
/// With `Default` values, the path will never create a bridge.

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeRules {
    /// Maximum length of bridges.
    pub max_bridge_length: f64,

    /// Number of check steps to create a bridge.
    pub check_step: usize,
}

impl Default for BridgeRules {
    fn default() -> Self {
        Self {
            max_bridge_length: 0.0,
            check_step: 0,
        }
    }
}
