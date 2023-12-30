use color_eyre::Result;
use image::GenericImageView;

fn compute_mip_level(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> (u32, u32, Vec<u8>) {
    let width = width / 2;
    let height = height / 2;
    let new_rgba = (0..height)
        .flat_map(|v| {
            (0..width).flat_map(move |u| (0..4_u32).map(move |c| (u, v, c)))
        })
        .map(|(u, v, c)| {
            ((rgba[(((v * 2) * width * 2 + u * 2) * 4 + c) as usize] as u16
                + rgba[(((v * 2) * width * 2 + u * 2 + 1) * 4 + c) as usize]
                    as u16
                + rgba[(((v * 2 + 1) * width * 2 + u * 2) * 4 + c) as usize]
                    as u16
                + rgba
                    [(((v * 2 + 1) * width * 2 + u * 2 + 1) * 4 + c) as usize]
                    as u16)
                / 4) as u8
        })
        .collect::<Vec<_>>();
    (width, height, new_rgba)
}

pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat =
        wgpu::TextureFormat::Depth32Float;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        file: &std::path::Path,
    ) -> Result<Self> {
        let file = std::fs::File::open(file)?;
        let reader = std::io::BufReader::new(file);
        let image = image::load(reader, image::ImageFormat::Png)?;
        Ok(Self::from_image(device, queue, image, label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: image::DynamicImage,
        label: Option<&str>,
    ) -> Self {
        let rgba = image.to_rgba8();
        let (width, height) = image.dimensions();

        Self::from_rgba(device, queue, &rgba, width, height, label, 1)
    }

    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        width: u32,
        height: u32,
        label: Option<&str>,
        mip_level_count: u32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            label,
            view_formats: &[],
        });
        if mip_level_count == 1 {
            queue.write_texture(
                wgpu::ImageCopyTextureBase {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                size,
            );
        } else {
            let mut width = width;
            let mut height = height;
            let mut rgba = rgba.to_vec();
            for mip_level in 0..mip_level_count {
                let size = wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                };
                queue.write_texture(
                    wgpu::ImageCopyTextureBase {
                        texture: &texture,
                        mip_level,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &rgba,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * width),
                        rows_per_image: Some(height),
                    },
                    size,
                );
                (width, height, rgba) = compute_mip_level(width, height, rgba);
            }
        }

        let view =
            texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            texture,
            sampler,
            view,
        }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view =
            texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
