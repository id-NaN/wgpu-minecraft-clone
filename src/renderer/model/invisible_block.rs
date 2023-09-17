use super::BlockMeshGenerator;
use crate::renderer::Mesh;
use crate::util::BlockSide;
use crate::world::Block;

pub struct InvisibleBlock;

impl BlockMeshGenerator for InvisibleBlock {
    fn mesh_side(&self, _: glm::I16Vec3, _: Block, _: BlockSide, _: &mut Mesh) {}
    fn solid_side(&self, _: Block, _: crate::util::BlockSide) -> bool {
        false
    }
}
