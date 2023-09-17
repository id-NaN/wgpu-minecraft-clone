use super::Block;

pub struct Chunk {
    blocks: [[[Block; Self::SIZE_Z as usize]; Self::SIZE_X as usize]; Self::SIZE_Y as usize],
}

impl Chunk {
    pub const SIZE_X: u16 = 16;
    pub const SIZE_Y: u16 = 384;
    pub const SIZE_Z: u16 = 16;

    pub fn new() -> Self {
        Self {
            blocks: [[[Block::air(); Self::SIZE_Z as usize]; Self::SIZE_X as usize];
                Self::SIZE_Y as usize],
        }
    }

    pub fn get_block(&self, position: glm::I16Vec3) -> Block {
        self.blocks[position.y as usize][position.z as usize][position.x as usize]
    }

    pub fn position_in_chunk(position: glm::I16Vec3) -> bool {
        0 <= position.x
            && position.x < Self::SIZE_X as i16
            && 0 <= position.y
            && position.y < Self::SIZE_Y as i16
            && 0 <= position.z
            && position.z < Self::SIZE_Z as i16
    }

    pub fn consider_position_solid(position: glm::I16Vec3) -> bool {
        position.y < 0 || position.y >= Self::SIZE_Y as i16
    }

    pub fn set_block(&mut self, position: glm::I16Vec3, block: Block) {
        self.blocks[position.y as usize][position.z as usize][position.x as usize] = block
    }
}
