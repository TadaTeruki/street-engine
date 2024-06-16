#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stage(usize);

impl Stage {
    pub fn from_num(stage: usize) -> Self {
        Self(stage)
    }

    pub fn get_num(&self) -> usize {
        self.0
    }

    pub fn incremented(self) -> Self {
        Self(self.0 + 1)
    }
}
