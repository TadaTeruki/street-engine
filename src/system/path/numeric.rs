/// A numeric value that represents a stage in the network growth process.
///
/// Stage expresses the hierarchy of the path, for example, Highway (stage = 0) -> Street (stage = 1) -> Alley (stage = 2).
/// The specific meaning of the stage is determined by the context.
///
/// Stage are propagated to the next extended nodes, and can be incremented when the path is branched.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stage(usize);

impl Stage {
    /// Initial stage.
    pub fn initial() -> Self {
        Self(0)
    }

    /// Create a new stage from a number.
    pub fn from_num(stage: usize) -> Self {
        Self(stage)
    }

    /// Get the number of the stage.
    pub fn get_num(&self) -> usize {
        self.0
    }

    /// Increment the stage by one.
    pub fn incremented(self) -> Self {
        Self(self.0 + 1)
    }
}
