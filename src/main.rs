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

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use color_eyre::Result;
use settings::SETTINGS;
use world::{ChunkEvent, GameData};

use crate::world::World;

fn main_loop(
    chunk_sender: Sender<ChunkEvent>,
    game_data: GameData,
    world: Arc<Mutex<World>>,
) {
    let generator = test_world_generator::ChunkGenerator::new();
    let generate_chunks = SETTINGS.graphics.render_distance as i32 / 16;
    for y in -generate_chunks..generate_chunks {
        for x in -generate_chunks..generate_chunks {
            let position = glm::vec2(x, y);
            world
                .lock()
                .unwrap()
                .set_chunk(generator.generate_chunk(position, &game_data));
        }
    }
    for y in -generate_chunks..generate_chunks {
        for x in -generate_chunks..generate_chunks {
            let position = glm::vec2(x, y);
            chunk_sender.send(ChunkEvent::Update(position)).unwrap();
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    logger::initialize_logger()?;
    log::info!("Hello, World!");

    let (game_data, game_render_data) = init::initialize()?;

    let (chunk_sender, chunk_receiver) = mpsc::channel();
    let world = Arc::new(Mutex::new(World::new()));
    thread::scope(|s| -> Result<()> {
        let clone_world = world.clone();
        s.spawn(move || main_loop(chunk_sender, game_data, clone_world));
        renderer::run(game_render_data, chunk_receiver, world)
    })?;
    Ok(())
}
