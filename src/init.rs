use std::path::Path;

use color_eyre::Result;

use crate::{
    renderer::{
        model::{CubeBlock, InitBlockMeshGenerator},
        GameRenderData,
    },
    world::{BlockType, GameData},
};

mod init_texture_provider;

pub use init_texture_provider::InitTextureProvider;

pub fn initialize() -> Result<(GameData, GameRenderData)> {
    let mut block_types = Vec::new();
    let mut mesh_generators: Vec<Box<dyn InitBlockMeshGenerator>> = Vec::new();

    let mut texture_provider = InitTextureProvider::new();

    block_types.push(BlockType::new("cobblestone"));
    mesh_generators.push(Box::new(CubeBlock::new(
        Path::new("assets/textures/blocks/cobblestone.png"),
        &mut texture_provider,
    )));

    assert!(block_types.len() == mesh_generators.len());

    let atlas = texture_provider.atlas(16)?;
    let mesh_generators = mesh_generators
        .into_iter()
        .map(|generator| generator.build(&atlas))
        .collect();
    Ok((
        GameData::new(block_types),
        GameRenderData::new(mesh_generators, atlas),
    ))
}
