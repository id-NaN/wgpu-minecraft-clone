use std::path::{Path, PathBuf};

use color_eyre::eyre::Context;
use color_eyre::Result;

use crate::renderer::TextureAtlas;

pub struct InitTextureProvider {
    texture_names: Vec<PathBuf>,
}

impl InitTextureProvider {
    pub fn new() -> Self {
        Self {
            texture_names: Vec::new(),
        }
    }

    pub fn texture(&mut self, texture_path: &Path) -> u32 {
        if let Some(index) =
            self.texture_names.iter().position(|x| x == texture_path)
        {
            index as u32
        } else {
            let index = self.texture_names.len() as u32;
            self.texture_names.push(texture_path.to_path_buf());
            index
        }
    }

    pub fn atlas(&self, texture_size: u32) -> Result<TextureAtlas> {
        let mut atlas =
            TextureAtlas::new(texture_size, self.texture_names.len() as u32);
        for texture in &self.texture_names {
            atlas
                .add_path(texture)
                .wrap_err("Failed populating texture atlas!")?;
        }
        Ok(atlas)
    }
}
