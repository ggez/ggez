struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct Locals {
    transform: mat4x4<f32>,
    rotation: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> locals: Locals;

@group(0) @binding(1)
var t_color: texture_2d<f32>;

@group(0) @binding(2)
var s_sampler: sampler;

@vertex
fn vs_main(
    @location(0) pos: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = locals.transform * locals.rotation * pos;
    out.position = out.position / out.position.w;
    out.tex_coord = tex_coord;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex = textureSample(t_color, s_sampler, in.tex_coord);
    var blend = dot(in.tex_coord - vec2<f32>(0.5, 0.5), in.tex_coord - vec2<f32>(0.5, 0.5));
    return mix(tex, vec4<f32>(0.0, 0.0, 0.0, 0.0), blend);
}
