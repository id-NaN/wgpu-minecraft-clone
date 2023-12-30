mod cube_block;
mod invisible_block;

pub use cube_block::CubeBlock;
pub use invisible_block::InvisibleBlock;

use super::{Mesh, TextureAtlas};
use crate::util::BlockSide;
use crate::world::Block;

pub trait BlockMeshGenerator {
    fn mesh_always(
        &self,
        _position: glm::IVec3,
        _block: Block,
        _mesh: &mut Mesh,
    ) {
    }

    fn mesh_side(
        &self,
        position: glm::IVec3,
        block: Block,
        side: BlockSide,
        mesh: &mut Mesh,
    );
    fn solid_side(&self, block: Block, side: BlockSide) -> bool;
}

pub trait InitBlockMeshGenerator {
    fn build(&self, atlas: &TextureAtlas) -> Box<dyn BlockMeshGenerator>;
}
