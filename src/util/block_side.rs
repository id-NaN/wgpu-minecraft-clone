use strum::EnumIter;

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum BlockSide {
    Top,
    Bottom,
    North,
    East,
    South,
    West,
}

impl BlockSide {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub fn direction(&self) -> glm::I16Vec3 {
        match self {
            BlockSide::Top => glm::vec3(0, 1, 0),
            BlockSide::Bottom => glm::vec3(0, -1, 0),
            BlockSide::North => glm::vec3(0, 0, 1),
            BlockSide::East => glm::vec3(1, 0, 0),
            BlockSide::South => glm::vec3(0, 0, -1),
            BlockSide::West => glm::vec3(-1, 0, 0),
        }
    }
}
