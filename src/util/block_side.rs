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

    pub fn direction<T: From<i8> + glm::Scalar>(&self) -> glm::TVec3<T> {
        match self {
            BlockSide::Top => {
                glm::vec3((0_i8).into(), (1_i8).into(), (0_i8).into())
            }
            BlockSide::Bottom => {
                glm::vec3((0_i8).into(), (-1_i8).into(), (0_i8).into())
            }
            BlockSide::North => {
                glm::vec3((0_i8).into(), (0_i8).into(), (1_i8).into())
            }
            BlockSide::East => {
                glm::vec3((1_i8).into(), (0_i8).into(), (0_i8).into())
            }
            BlockSide::South => {
                glm::vec3((0_i8).into(), (0_i8).into(), (-1_i8).into())
            }
            BlockSide::West => {
                glm::vec3((-1_i8).into(), (0_i8).into(), (0_i8).into())
            }
        }
    }

    pub fn flat_direction<T: From<i8> + glm::Scalar + Clone>(
        &self,
    ) -> glm::TVec2<T> {
        let direction = self.direction::<T>();
        glm::vec2(direction.x.clone(), direction.z.clone())
    }

    pub fn cardinal_sides() -> impl Iterator<Item = Self> {
        [Self::North, Self::East, Self::South, Self::West].into_iter()
    }

    pub fn cardinal_directions<T: From<i8> + glm::Scalar>(
    ) -> impl Iterator<Item = glm::TVec2<T>> {
        Self::cardinal_sides().map(|side| side.direction().xz())
    }
}
