use super::model::BlockMeshGenerator;
use super::TextureAtlas;

pub struct GameRenderData {
    mesh_generators: Vec<Box<dyn BlockMeshGenerator>>,
    texture_atlas: TextureAtlas,
}

impl GameRenderData {
    pub fn new(
        mesh_generators: Vec<Box<dyn BlockMeshGenerator>>,
        texture_atlas: TextureAtlas,
    ) -> Self {
        Self {
            mesh_generators,
            texture_atlas,
        }
    }

    pub fn mesh_generator(&self, block_type: u16) -> &dyn BlockMeshGenerator {
        self.mesh_generators[block_type as usize].as_ref()
    }

    pub fn texture_atlas(&self) -> &TextureAtlas {
        &self.texture_atlas
    }
}
