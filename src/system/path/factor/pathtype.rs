/// Represents a type of path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathType {
    Bridge,
    Normal,
    Impossible,
}

impl PathType {
    pub fn is_bridge(&self) -> bool {
        match self {
            PathType::Bridge => true,
            _ => false,
        }
    }

    pub fn is_normal(&self) -> bool {
        match self {
            PathType::Normal => true,
            _ => false,
        }
    }

    pub fn is_impossible(&self) -> bool {
        match self {
            PathType::Impossible => true,
            _ => false,
        }
    }

    pub fn can_create_intersection(&self) -> bool {
        !self.is_impossible() && !self.is_bridge()
    }
}
