use color_eyre::{eyre::Context, Result};
use std::path::Path;

pub fn load_shader_module(device: &wgpu::Device, shader_name: &str) -> Result<wgpu::ShaderModule> {
    let path = Path::new("assets/shaders")
        .join(shader_name)
        .with_extension("wgsl");
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_name),
        source: wgpu::ShaderSource::Wgsl(
            std::fs::read_to_string(path)
                .wrap_err_with(|| format!("Failure loading shader \"{shader_name}\""))?
                .into(),
        ),
    }))
}
