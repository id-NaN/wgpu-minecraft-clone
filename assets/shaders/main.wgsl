struct CameraUniform {
    view_projection: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    out.clip_position = camera.view_projection * vec4<f32>(vertex.position, 1.0);
    return out;
}

// @group(1) @binding(0)
// var cube_texture: texture_2d<f32>;
// @group(1) @binding(1)
// var cube_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // textureSample(cube_texture, cube_sampler, vertex.tex_coords);
}
