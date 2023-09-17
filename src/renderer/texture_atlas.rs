use color_eyre::{eyre::Context, Result};

use super::Texture;

pub struct TextureAtlas {
    texture_data: Vec<u8>,
    texture_size: u32,
    level: u8,
    texture_index: u32,
}

impl TextureAtlas {
    pub fn new(texture_size: u32, texture_count: u32) -> Self {
        let level = (texture_count as f32).log(4.0).ceil() as u8;
        dbg!(level);
        Self {
            texture_index: 0,
            texture_data: vec![0; ((texture_size * 2_u32.pow(level as u32)).pow(2) * 4) as usize],
            texture_size,
            level,
        }
    }

    fn textures_per_side(&self) -> u32 {
        2_u32.pow(self.level as u32)
    }

    fn side_length(&self) -> u32 {
        self.textures_per_side() * self.texture_size
    }

    pub fn get_texture_coordinates(&self, index: u32) -> (glm::Vec2, glm::Vec2) {
        assert!(index < self.texture_index);
        let textures_per_side = self.textures_per_side() as f32;
        let tile_x = index as f32 % textures_per_side;
        let tile_y = index as f32 / textures_per_side;
        (
            glm::vec2(tile_x / textures_per_side, tile_y / textures_per_side),
            glm::vec2(
                (tile_x + 1.0) / textures_per_side,
                (tile_y + 1.0) / textures_per_side,
            ),
        )
    }

    pub fn add_texture(&mut self, data: &[u8]) -> u32 {
        let expected_len = self.texture_size.pow(2) * 4;
        if data.len() != expected_len as usize {
            panic!(
                "Texture has wrong size: {} instead of {expected_len}!",
                data.len()
            )
        }
        let index = self.texture_index;
        let textures_per_side = self.textures_per_side();
        if index >= textures_per_side.pow(2) {
            panic!(
                "Atlas may only contain {} textures!",
                textures_per_side.pow(2)
            )
        }

        let side_length = self.side_length();

        let x_offset = (index % textures_per_side) * self.texture_size;
        let y_offset = (index / textures_per_side) * self.texture_size;
        let atlas_offset = (y_offset * side_length + x_offset) * 4;

        for y in 0..self.texture_size {
            for x in 0..self.texture_size * 4 {
                self.texture_data[(atlas_offset + y * side_length * 4 + x) as usize] =
                    data[(y * self.texture_size * 4 + x) as usize];
            }
        }
        self.texture_index += 1;
        index
    }

    pub fn add_path(&mut self, path: &std::path::Path) -> Result<u32> {
        let file = std::fs::File::open(path)
            .wrap_err_with(|| format!("Failed to add file {path:?} to atlas!"))?;
        let reader = std::io::BufReader::new(file);
        let image = image::load(reader, image::ImageFormat::Png)
            .wrap_err_with(|| format!("Failed to add file {path:?} to atlas!"))?;
        if image.width() != self.texture_size || image.height() != self.texture_size {
            panic!(
                "Atlas expects image of size {}x{} pixels, got {}x{} pixels!",
                self.texture_size,
                self.texture_size,
                image.width(),
                image.height()
            )
        }
        Ok(self.add_texture(&image.to_rgba8()))
    }

    pub fn texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
    ) -> Texture {
        Texture::from_rgba(
            device,
            queue,
            &self.texture_data,
            self.side_length(),
            self.side_length(),
            label,
        )
    }
}
