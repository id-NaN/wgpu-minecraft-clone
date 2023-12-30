struct CameraUniform {
    view_projection: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct LightUniform {
    direction: vec3<f32>,
    color: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) world_position: vec3<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(2) world_normal: vec3<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    out.clip_position = camera.view_projection * vec4<f32>(vertex.position, 1.0);
    out.world_position = vertex.position;
    out.world_normal = vertex.normal;
    return out;
}

@group(1) @binding(0)
var atlas: texture_2d<f32>;
@group(1) @binding(1)
var atlas_sampler: sampler;
@group(2) @binding(0)
var<uniform> light: LightUniform;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(atlas, atlas_sampler, vertex.tex_coords);
    let ambient_strength = 0.1;
    let diffuse_strength = max(dot(vertex.world_normal, light.direction), 0.0);
    let result = (ambient_strength + diffuse_strength) * light.color * object_color.xyz;
    return vec4<f32>(result, object_color.a);
}
