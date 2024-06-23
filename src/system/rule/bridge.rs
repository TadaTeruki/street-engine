use crate::unit::Length;

/// Rules to create bridges.
///
/// Bridge is the path that connect two distant sites where the normal path cannot be constructed.
/// For this package, the meaning of bridges includes not only the bridges over rivers or valleys but also tunnels under mountains.
///
/// With `Default` values, the path will never create a bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRule {
    /// Maximum length of bridges.
    pub max_bridge_length: Length,

    /// Number of check steps to create a bridge.
    pub check_step: usize,
}

impl Default for BridgeRule {
    fn default() -> Self {
        Self {
            max_bridge_length: Length::new(0.0),
            check_step: 0,
        }
    }
}

impl BridgeRule {
    /// Set the maximum length of bridges.
    pub fn max_bridge_length(mut self, max_bridge_length: Length) -> Self {
        self.max_bridge_length = max_bridge_length;
        self
    }

    /// Set the number of check steps to create a bridge.
    pub fn check_step(mut self, check_step: usize) -> Self {
        self.check_step = check_step;
        self
    }
}
