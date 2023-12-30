use super::{BlockMeshGenerator, InitBlockMeshGenerator};
use crate::renderer::Mesh;
use crate::util::BlockSide;
use crate::world::Block;

pub struct InvisibleBlock;

impl InvisibleBlock {
    pub fn new() -> InitInvisibleBlock {
        InitInvisibleBlock::new()
    }
}

impl BlockMeshGenerator for InvisibleBlock {
    fn mesh_side(&self, _: glm::IVec3, _: Block, _: BlockSide, _: &mut Mesh) {}

    fn solid_side(&self, _: Block, _: crate::util::BlockSide) -> bool {
        false
    }
}

pub struct InitInvisibleBlock;

impl InitInvisibleBlock {
    fn new() -> Self {
        Self {}
    }
}

impl InitBlockMeshGenerator for InitInvisibleBlock {
    fn build(
        &self,
        _atlas: &crate::renderer::TextureAtlas,
    ) -> Box<dyn BlockMeshGenerator> {
        return Box::new(InvisibleBlock {});
    }
}
