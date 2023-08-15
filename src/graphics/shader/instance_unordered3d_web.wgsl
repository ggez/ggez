struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) vertex_color: vec4<f32>
}

struct Uniforms {
    color: vec4<f32>,
    model_transform: mat4x4<f32>,
    camera_transform: mat4x4<f32>,
}

struct DrawParam {
    color: vec4<f32>,
    model_transform: mat4x4<f32>,
    camera_transform: mat4x4<f32>,
}

struct InstanceArray {
    instances: array<DrawParam, 17>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

@group(2) @binding(0)
var<uniform> instances: InstanceArray;

@vertex
fn vs_main(
    @builtin(instance_index) in_instance_index: u32,
    model: VertexInput,
) -> VertexOutput {
    var index = in_instance_index;
    var instance = instances.instances[index];

    var out: VertexOutput;
    out.clip_position = uniforms.camera_transform * uniforms.model_transform * instance.model_transform * vec4<f32>(model.position, 1.0);
    out.tex_coord = model.tex_coords;
    out.vertex_color = model.color;
    out.color = uniforms.color * instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * in.vertex_color * textureSample(t, s, in.tex_coord);
}
