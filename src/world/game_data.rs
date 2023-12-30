use std::collections::HashMap;

use color_eyre::eyre::ContextCompat;
use color_eyre::Result;

use super::BlockType;

pub struct GameData {
    block_types: Vec<BlockType>,
    block_type_map: HashMap<String, u16>,
}

impl GameData {
    pub fn new(block_types: Vec<BlockType>) -> GameData {
        assert_eq!(block_types[0].name(), "air", "First block must be Air");
        Self {
            block_type_map: block_types
                .iter()
                .enumerate()
                .map(|(id, x)| (x.name().to_string(), id as u16))
                .collect(),
            block_types,
        }
    }

    pub fn block_id(&self, name: &str) -> Result<u16> {
        self.block_type_map
            .get(name)
            .map(|x| *x)
            .wrap_err("Unknown block name!")
    }
}
