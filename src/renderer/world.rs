use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use glm::{I16Vec3, IVec2, IVec3};
use strum::IntoEnumIterator;
use winit::dpi::Position;

use super::{GameRenderData, Mesh};
use crate::util::BlockSide;
use crate::world::{Block, Chunk, ChunkEvent, World};

pub enum ChunkMeshEvent {
    Update { position: glm::IVec2, mesh: Mesh },
    Unload(glm::IVec2),
}

pub struct RenderWorld {
    chunks: HashSet<glm::IVec2>,
    mesh_queue: Arc<Mutex<VecDeque<ChunkMeshEvent>>>,
    world: Arc<Mutex<World>>,
    render_data: GameRenderData,
}

impl RenderWorld {
    pub fn new(
        world: Arc<Mutex<World>>,
        render_data: GameRenderData,
    ) -> (Self, Arc<Mutex<VecDeque<ChunkMeshEvent>>>) {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        (
            Self {
                world,
                chunks: HashSet::new(),
                mesh_queue: queue.clone(),
                render_data,
            },
            queue,
        )
    }

    pub fn handle_chunk_event(&mut self, event: ChunkEvent) {
        let mut update_chunks: Vec<glm::IVec2> = vec![];
        match event {
            ChunkEvent::Update(position) => {
                self.chunks.insert(position);
                update_chunks.push(position);
            }
            ChunkEvent::Unload(position) => {
                self.chunks.remove(&position);
                BlockSide::cardinal_directions().for_each(|direction| {
                    update_chunks.push(position + direction)
                });
                self.mesh_queue
                    .lock()
                    .unwrap()
                    .push_back(ChunkMeshEvent::Unload(position));
            }
        };
        for chunk_position in update_chunks {
            let world = self.world.lock().unwrap();
            if let Some(chunk) = world.get_chunk(chunk_position) {
                let mesh = mesh_chunk(chunk, &self.render_data, &world);
                self.mesh_queue.lock().unwrap().push_back(
                    ChunkMeshEvent::Update {
                        position: chunk_position,
                        mesh,
                    },
                )
            }
        }
    }
}

pub fn mesh_chunk(
    chunk: &Chunk,
    render_data: &GameRenderData,
    world: &World,
) -> Mesh {
    let mut mesh = Mesh::new();

    let extra_chunks = [
        world.get_chunk(chunk.position() + BlockSide::North.flat_direction()),
        world.get_chunk(chunk.position() + BlockSide::East.flat_direction()),
        world.get_chunk(chunk.position() + BlockSide::South.flat_direction()),
        world.get_chunk(chunk.position() + BlockSide::West.flat_direction()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

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
                        chunk.position(),
                    ),
                    block,
                    &mut mesh,
                );

                for side in BlockSide::iter() {
                    let side_position = World::get_world_position(
                        chunk_block_position,
                        chunk.position(),
                    ) + side.direction();
                    if World::position_in_world(side_position.cast()) {
                        let side_block =
                            if chunk.position_in_chunk(side_position) {
                                Some(chunk.get_block(
                                    World::get_chunk_position(side_position),
                                ))
                            } else {
                                extra_chunks
                                    .iter()
                                    .flat_map(|chunk| {
                                        chunk
                                            .position_in_chunk(side_position)
                                            .then(|| {
                                                chunk.get_block(
                                                    World::get_chunk_position(
                                                        side_position,
                                                    ),
                                                )
                                            })
                                    })
                                    .collect::<Vec<_>>()
                                    .into_iter()
                                    .next()
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
                                    chunk.position(),
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
                                chunk.position(),
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
