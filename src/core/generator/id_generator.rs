/// IdGenerator is a simple struct that generates random ids.
///
/// This struct doesn't provide any methods to check the uniqueness of the generated ids.
#[derive(Debug, Clone)]
pub struct IdGenerator {
    next_id: usize,
}

/// Implement the IdGenerator struct.
impl IdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
}

impl IdGenerator {
    pub fn generate_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
