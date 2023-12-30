use super::Block;

pub struct Chunk {
    blocks: [[[Block; Self::SIZE_Z as usize]; Self::SIZE_X as usize];
        Self::SIZE_Y as usize],
}

impl Chunk {
    pub const LOWEST_HEIGHT: i16 = -64;
    pub const SIZE_X: u16 = 16;
    pub const SIZE_Y: u16 = 384;
    pub const SIZE_Z: u16 = 16;

    pub fn new() -> Self {
        Self {
            blocks: [[[Block::air(); Self::SIZE_Z as usize];
                Self::SIZE_X as usize];
                Self::SIZE_Y as usize],
        }
    }

    pub fn get_block(&self, position: glm::I16Vec3) -> Block {
        self.blocks[(position.y - Self::LOWEST_HEIGHT) as usize]
            [position.z as usize][position.x as usize]
    }

    pub fn position_in_chunk(
        chunk_position: glm::IVec2,
        position: glm::IVec3,
    ) -> bool {
        chunk_position.x * Self::SIZE_X as i32 <= position.x
            && position.x < (chunk_position.x + 1) * Self::SIZE_X as i32
            && Self::LOWEST_HEIGHT as i32 <= position.y
            && position.y < (Self::SIZE_Y as i32) + Self::LOWEST_HEIGHT as i32
            && chunk_position.y * Self::SIZE_Z as i32 <= position.z
            && position.z < (chunk_position.y + 1) * Self::SIZE_Z as i32
    }

    pub fn consider_position_solid(position: glm::I16Vec3) -> bool {
        position.y < 0 || position.y >= Self::SIZE_Y as i16
    }

    pub fn set_block(&mut self, position: glm::I16Vec3, block: Block) {
        assert!(
            position.y >= Self::LOWEST_HEIGHT
                && position.y < (Self::SIZE_Y as i16) + Self::LOWEST_HEIGHT
        );
        self.blocks[(position.y - Self::LOWEST_HEIGHT) as usize]
            [position.z as usize][position.x as usize] = block
    }
}
