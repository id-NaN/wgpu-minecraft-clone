#![feature(let_chains)]

extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

mod init;
mod logger;
mod renderer;
mod settings;
mod test_world_generator;
mod util;
mod world;

use color_eyre::Result;
use settings::SETTINGS;

use crate::world::World;

fn main() -> Result<()> {
    color_eyre::install()?;
    logger::initialize_logger()?;
    log::info!("Hello, World!");

    let (game_data, game_render_data) = init::initialize()?;

    let mut world = World::new();
    let generate_chunks = SETTINGS.graphics.render_distance as i32 / 16;
    for y in -generate_chunks..generate_chunks {
        for x in -generate_chunks..generate_chunks {
            let position = glm::vec2(x, y);
            world.set_chunk(
                position,
                test_world_generator::generate_chunk(position, &game_data),
            );
        }
    }

    renderer::run(world, game_render_data)?;
    Ok(())
}
