pub mod container;
pub mod geometry;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stage(usize);

impl Stage {
    pub fn new(stage: usize) -> Self {
        Self(stage)
    }

    pub fn get(&self) -> usize {
        self.0
    }

    pub fn incremented(mut self) -> Self {
        self.0 += 1;
        self
    }
}
