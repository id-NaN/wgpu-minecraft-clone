mod block;
mod block_type;
mod chunk;
mod game_data;

use std::collections::HashMap;

pub use block::Block;
pub use block_type::BlockType;
pub use chunk::Chunk;
pub use game_data::GameData;
use glm::{I16Vec3, IVec2, IVec3};

pub struct World {
    chunks: HashMap<IVec2, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn set_chunk(&mut self, chunk: Chunk) -> Option<Chunk> {
        self.chunks.insert(chunk.position(), chunk)
    }

    pub fn get_chunk(&self, position: IVec2) -> Option<&Chunk> {
        self.chunks.get(&position)
    }

    pub fn remove_chunk(
        &mut self,
        position: IVec2,
        chunk: Chunk,
    ) -> Option<Chunk> {
        self.chunks.remove(&position)
    }

    pub fn get_block(&self, position: IVec3) -> Option<Block> {
        self.chunks
            .get(&glm::vec2(
                position.x.div_euclid(Chunk::SIZE_X as i32),
                position.z.div_euclid(Chunk::SIZE_Z as i32),
            ))
            .map(|chunk| chunk.get_block(Self::get_chunk_position(position)))
    }

    pub fn position_in_world(position: IVec3) -> bool {
        Chunk::LOWEST_HEIGHT as i32 <= position.y
            && position.y
                < (Chunk::SIZE_Y as i32) + Chunk::LOWEST_HEIGHT as i32
    }

    pub fn get_chunk_position(position: IVec3) -> I16Vec3 {
        glm::vec3(
            position.x.rem_euclid(Chunk::SIZE_X as i32) as i16,
            position.y as i16,
            position.z.rem_euclid(Chunk::SIZE_Z as i32) as i16,
        )
    }

    pub fn get_world_position(
        chunk_block_position: I16Vec3,
        chunk_position: IVec2,
    ) -> IVec3 {
        glm::vec3(
            chunk_block_position.x as i32 + chunk_position.x * 16,
            chunk_block_position.y as i32,
            chunk_block_position.z as i32 + chunk_position.y * 16,
        )
    }
}

pub enum ChunkEvent {
    Update(IVec2),
    Unload(IVec2),
}
