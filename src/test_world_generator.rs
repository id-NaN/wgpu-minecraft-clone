use std::time::{SystemTime, UNIX_EPOCH};

use glm::IVec2;
use noise::{NoiseFn, Perlin};

use crate::world::{Block, Chunk, GameData};

pub struct ChunkGenerator {
    noise: Perlin,
}

impl ChunkGenerator {
    pub fn new() -> Self {
        Self {
            noise: Perlin::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u32,
            ),
        }
    }

    fn sample_height(&self, position: IVec2) -> i16 {
        let position = glm::vec2(position.x as f64, position.y as f64);
        (64.0
            + self.noise.get([position.x / 128.0, position.y / 128.0]) * 32.0
            + self.noise.get([position.x / 256.0, position.y / 256.0]) * 64.0
            + self.noise.get([position.x / 64.0, position.y / 64.0]) * 16.0
            + self.noise.get([position.x / 32.0, position.y / 32.0]) * 8.0
            + self.noise.get([position.x / 16.0, position.y / 16.0]) * 4.0
            + self.noise.get([position.x / 8.0, position.y / 8.0]) * 2.0)
            as i16
    }

    pub fn generate_chunk(
        &self,
        chunk_position: IVec2,
        game_data: &GameData,
    ) -> Chunk {
        let mut chunk = Chunk::new(chunk_position);
        for x in 0..16 {
            for z in 0..16 {
                for y in -64..self.sample_height(glm::vec2(
                    x + chunk_position.x * 16,
                    z + chunk_position.y * 16,
                )) {
                    chunk.set_block(
                        glm::vec3(
                            x.rem_euclid(16) as i16,
                            y,
                            z.rem_euclid(16) as i16,
                        ),
                        Block::new(game_data.block_id("dirt").unwrap()),
                    )
                }
            }
        }
        if chunk_position == glm::vec2(0, 0) {
            for x in 0..8 {
                chunk.set_block(
                    glm::vec3(x, 100, 0),
                    Block::new(game_data.block_id("dirt").unwrap()),
                )
            }
            for y in 100..108 {
                chunk.set_block(
                    glm::vec3(0, y, 0),
                    Block::new(game_data.block_id("cobblestone").unwrap()),
                )
            }
            for z in 0..8 {
                chunk.set_block(
                    glm::vec3(0, 100, z),
                    Block::new(game_data.block_id("cobblestone").unwrap()),
                )
            }
        }
        chunk
    }
}
