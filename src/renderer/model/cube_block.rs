use std::path::Path;

use super::{BlockMeshGenerator, InitBlockMeshGenerator, Mesh};
use crate::init::InitTextureProvider;
use crate::renderer::texture_atlas::TextureAtlas;
use crate::renderer::Vertex;
use crate::util::BlockSide;
use crate::world::Block;

pub struct CubeBlock {
    texture_coords: (glm::Vec2, glm::Vec2),
}

impl CubeBlock {
    pub fn new(
        texture_path: &Path,
        texture_provider: &mut InitTextureProvider,
    ) -> InitCubeBlock {
        InitCubeBlock::new(texture_path, texture_provider)
    }
}

impl BlockMeshGenerator for CubeBlock {
    fn mesh_side(
        &self,
        position: glm::IVec3,
        _: Block,
        side: BlockSide,
        mesh: &mut Mesh,
    ) {
        let texture_top_right =
            glm::vec2(self.texture_coords.1.x, self.texture_coords.0.y);
        let texture_bottom_left =
            glm::vec2(self.texture_coords.0.x, self.texture_coords.1.y);
        match side {
            BlockSide::Bottom => {
                mesh.add_quad(
                    Vertex {
                        position: glm::vec3(
                            0.0 + position.x as f32,
                            0.0 + position.y as f32,
                            1.0 + position.z as f32,
                        ),
                        tex_coords: texture_bottom_left,
                        normal: BlockSide::Bottom.direction(),
                    },
                    Vertex {
                        position: glm::vec3(
                            0.0 + position.x as f32,
                            0.0 + position.y as f32,
                            0.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.0,
                        normal: BlockSide::Bottom.direction(),
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            0.0 + position.y as f32,
                            0.0 + position.z as f32,
                        ),
                        tex_coords: texture_top_right,
                        normal: BlockSide::Bottom.direction(),
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            0.0 + position.y as f32,
                            1.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.1,
                        normal: BlockSide::Bottom.direction(),
                    },
                );
            }
            BlockSide::Top => {
                mesh.add_quad(
                    Vertex {
                        position: glm::vec3(
                            0.0 + position.x as f32,
                            1.0 + position.y as f32,
                            0.0 + position.z as f32,
                        ),
                        tex_coords: texture_bottom_left,
                        normal: BlockSide::Top.direction(),
                    },
                    Vertex {
                        position: glm::vec3(
                            0.0 + position.x as f32,
                            1.0 + position.y as f32,
                            1.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.0,
                        normal: BlockSide::Top.direction(),
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            1.0 + position.y as f32,
                            1.0 + position.z as f32,
                        ),
                        tex_coords: texture_top_right,
                        normal: BlockSide::Top.direction(),
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            1.0 + position.y as f32,
                            0.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.1,
                        normal: BlockSide::Top.direction(),
                    },
                );
            }
            BlockSide::North => mesh.add_quad(
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        0.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: texture_bottom_left,
                    normal: BlockSide::North.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                    normal: BlockSide::North.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                    normal: BlockSide::North.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        0.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
                    normal: BlockSide::North.direction(),
                },
            ),
            BlockSide::East => mesh.add_quad(
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        0.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: texture_bottom_left,
                    normal: BlockSide::East.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                    normal: BlockSide::East.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                    normal: BlockSide::East.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        0.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
                    normal: BlockSide::East.direction(),
                },
            ),
            BlockSide::South => mesh.add_quad(
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        0.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: texture_bottom_left,
                    normal: BlockSide::South.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                    normal: BlockSide::South.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                    normal: BlockSide::South.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        0.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
                    normal: BlockSide::South.direction(),
                },
            ),
            BlockSide::West => mesh.add_quad(
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        0.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: texture_bottom_left,
                    normal: BlockSide::West.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                    normal: BlockSide::West.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                    normal: BlockSide::West.direction(),
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        0.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
                    normal: BlockSide::West.direction(),
                },
            ),
        }
    }

    fn solid_side(&self, _: Block, _: crate::util::BlockSide) -> bool {
        true
    }
}

pub struct InitCubeBlock {
    face_texture: u32,
}

impl InitCubeBlock {
    fn new(
        texture_path: &Path,
        texture_provider: &mut InitTextureProvider,
    ) -> Self {
        Self {
            face_texture: texture_provider.texture(texture_path),
        }
    }
}

impl InitBlockMeshGenerator for InitCubeBlock {
    fn build(&self, atlas: &TextureAtlas) -> Box<dyn BlockMeshGenerator> {
        Box::new(CubeBlock {
            texture_coords: atlas.get_texture_coordinates(self.face_texture),
        })
    }
}
