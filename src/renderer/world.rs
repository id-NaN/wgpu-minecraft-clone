use strum::IntoEnumIterator;

use crate::{util::BlockSide, world::Chunk};

use super::{GameRenderData, Mesh};

pub fn mesh_chunk(chunk: &Chunk, render_data: &GameRenderData) -> Mesh {
    let mut mesh = Mesh::new();

    for z in 0..Chunk::SIZE_Z {
        for y in 0..Chunk::SIZE_Y {
            for x in 0..Chunk::SIZE_X {
                let position = glm::vec3(x as i16, y as i16, z as i16);
                let block = chunk.get_block(position);
                let model_generator = render_data.mesh_generator(block.block_type());

                model_generator.mesh_always(position, block, &mut mesh);

                for side in BlockSide::iter() {
                    let side_position = position + side.direction();
                    if Chunk::position_in_chunk(side_position) {
                        let side_block = chunk.get_block(side_position);
                        let side_solid = render_data
                            .mesh_generator(side_block.block_type())
                            .solid_side(side_block, side.opposite());
                        if !side_solid {
                            model_generator.mesh_side(position, block, side, &mut mesh);
                        }
                    } else if Chunk::consider_position_solid(side_position) {
                        model_generator.mesh_side(position, block, side, &mut mesh);
                    }
                }
            }
        }
    }

    mesh
}
