mod cube_block;
mod invisible_block;

use crate::{util::BlockSide, world::Block};

use super::{Mesh, TextureAtlas};
pub use cube_block::CubeBlock;
pub use invisible_block::InvisibleBlock;

pub trait BlockMeshGenerator {
    fn mesh_always(&self, _position: glm::I16Vec3, _block: Block, _mesh: &mut Mesh) {}

    fn mesh_side(&self, position: glm::I16Vec3, block: Block, side: BlockSide, mesh: &mut Mesh);
    fn solid_side(&self, block: Block, side: BlockSide) -> bool;
}

pub trait InitBlockMeshGenerator {
    fn build(&self, atlas: &TextureAtlas) -> Box<dyn BlockMeshGenerator>;
}
