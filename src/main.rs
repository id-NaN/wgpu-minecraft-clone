#![feature(let_chains)]

extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

mod init;
mod logger;
mod renderer;
mod settings;
mod util;
mod world;

use color_eyre::Result;
use settings::SETTINGS;

fn main() -> Result<()> {
    color_eyre::install()?;
    logger::initialize_logger()?;
    log::info!("Hello, World!");

    let (game_data, game_render_data) = init::initialize()?;

    let mut chunk = world::Chunk::new();
    for z in 0..16 {
        for x in 0..16 {
            chunk.set_block(
                glm::vec3(x, 0, z),
                world::Block::new(game_data.block_id("cobblestone")?),
            );
        }
    }
    for i in 0..384 {
        chunk.set_block(
            glm::vec3(0, i, 0),
            world::Block::new(game_data.block_id("cobblestone")?),
        );
    }

    renderer::run(chunk, game_render_data)?;
    Ok(())
}
