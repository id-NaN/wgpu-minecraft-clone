use std::path::Path;

use color_eyre::eyre::Context;
use color_eyre::Result;
use lazy_static::lazy_static;
use regex::{Captures, Regex};

lazy_static! {
    static ref FIND_INTERPOLATION: Regex = Regex::new(r"\$\{(.+?)\}").unwrap();
}

pub fn load_shader_module(
    device: &wgpu::Device,
    shader_name: &str,
    interpolation: &[(&str, &str)],
) -> Result<wgpu::ShaderModule> {
    let path = Path::new("assets/shaders")
        .join(shader_name)
        .with_extension("wgsl");
    let raw_shader_source =
        std::fs::read_to_string(path).wrap_err_with(|| {
            format!("Failure loading shader \"{shader_name}\"")
        })?;
    let shader_source = FIND_INTERPOLATION.replace_all(
        &raw_shader_source,
        |captures: &Captures| {
            interpolation
                .iter()
                .filter_map(|(key, value)| {
                    (*key == &captures[1]).then_some(*value)
                })
                .next()
                .unwrap_or_else(|| {
                    panic!(
                        "Missing interpolation for key \"{}\"",
                        &captures[1]
                    )
                })
        },
    );
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_name),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    }))
}
