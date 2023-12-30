use glm::IVec2;
use strum::IntoEnumIterator;

use super::{GameRenderData, Mesh};
use crate::util::BlockSide;
use crate::world::{Chunk, World};

pub fn mesh_chunk(
    chunk_position: IVec2,
    chunk: &Chunk,
    render_data: &GameRenderData,
    world: &World,
) -> Mesh {
    let mut mesh = Mesh::new();

    for z in 0..Chunk::SIZE_Z {
        for y in 0..Chunk::SIZE_Y {
            for x in 0..Chunk::SIZE_X {
                let chunk_block_position =
                    glm::vec3(x as i16, (y as i16) - 64, z as i16);
                let block = chunk.get_block(chunk_block_position);
                let model_generator =
                    render_data.mesh_generator(block.block_type());

                model_generator.mesh_always(
                    World::get_world_position(
                        chunk_block_position,
                        chunk_position,
                    ),
                    block,
                    &mut mesh,
                );

                for side in BlockSide::iter() {
                    let side_position = World::get_world_position(
                        chunk_block_position,
                        chunk_position,
                    ) + side.direction();
                    if World::position_in_world(side_position.cast()) {
                        let side_block = if Chunk::position_in_chunk(
                            chunk_position,
                            side_position,
                        ) {
                            Some(chunk.get_block(World::get_chunk_position(
                                side_position,
                            )))
                        } else {
                            world.get_block(side_position.cast())
                        };
                        let side_solid = match side_block {
                            Some(side_block) => render_data
                                .mesh_generator(side_block.block_type())
                                .solid_side(side_block, side.opposite()),
                            None => true,
                        };
                        if !side_solid {
                            model_generator.mesh_side(
                                World::get_world_position(
                                    chunk_block_position,
                                    chunk_position,
                                ),
                                block,
                                side,
                                &mut mesh,
                            );
                        }
                    } else {
                        model_generator.mesh_side(
                            World::get_world_position(
                                chunk_block_position,
                                chunk_position,
                            ),
                            block,
                            side,
                            &mut mesh,
                        );
                    }
                }
            }
        }
    }

    mesh
}

pub fn mesh_world(world: &World, render_data: &GameRenderData) -> Vec<Mesh> {
    world
        .iter()
        .map(|(position, chunk)| {
            mesh_chunk(*position, chunk, render_data, world)
        })
        .collect()
}
