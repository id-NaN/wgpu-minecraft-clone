use std::path::Path;

use super::{InitBlockMeshGenerator, Mesh};
use crate::{
    init::InitTextureProvider,
    renderer::{texture_atlas::TextureAtlas, Vertex},
    util::BlockSide,
    world::Block,
};

use super::BlockMeshGenerator;

pub struct CubeBlock {
    texture_coords: (glm::Vec2, glm::Vec2),
}

impl CubeBlock {
    pub fn new(texture_path: &Path, texture_provider: &mut InitTextureProvider) -> InitCubeBlock {
        InitCubeBlock::new(texture_path, texture_provider)
    }
}

impl BlockMeshGenerator for CubeBlock {
    fn mesh_side(&self, position: glm::I16Vec3, _: Block, side: BlockSide, mesh: &mut Mesh) {
        let texture_top_right = glm::vec2(self.texture_coords.1.x, self.texture_coords.0.y);
        let texture_bottom_left = glm::vec2(self.texture_coords.0.x, self.texture_coords.1.y);
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
                    },
                    Vertex {
                        position: glm::vec3(
                            0.0 + position.x as f32,
                            0.0 + position.y as f32,
                            0.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.0,
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            0.0 + position.y as f32,
                            0.0 + position.z as f32,
                        ),
                        tex_coords: texture_top_right,
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            0.0 + position.y as f32,
                            1.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.1,
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
                    },
                    Vertex {
                        position: glm::vec3(
                            0.0 + position.x as f32,
                            1.0 + position.y as f32,
                            1.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.0,
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            1.0 + position.y as f32,
                            1.0 + position.z as f32,
                        ),
                        tex_coords: texture_top_right,
                    },
                    Vertex {
                        position: glm::vec3(
                            1.0 + position.x as f32,
                            1.0 + position.y as f32,
                            0.0 + position.z as f32,
                        ),
                        tex_coords: self.texture_coords.1,
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
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        0.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
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
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        0.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
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
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                },
                Vertex {
                    position: glm::vec3(
                        1.0 + position.x as f32,
                        0.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
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
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        1.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.0,
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        1.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: texture_top_right,
                },
                Vertex {
                    position: glm::vec3(
                        0.0 + position.x as f32,
                        0.0 + position.y as f32,
                        0.0 + position.z as f32,
                    ),
                    tex_coords: self.texture_coords.1,
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
    fn new(texture_path: &Path, texture_provider: &mut InitTextureProvider) -> Self {
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
