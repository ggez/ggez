struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

let vertices = array<vec2<f32>, 3>(
    vec2<f32>(-1, -1),
    vec2<f32>(3, -1),
    vec2<f32>(-1, 3)
);

@group(0) @binding(0) var t: texture_2d<f32>;
@group(1) @binding(0) var s: sampler;

@stage(vertex)
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertices[in_vertex_index], 0.0, 1.0);
    out.uv = 0.5 * out.position.xy + vec2<f32>(0.5);
    return out;
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t, s, in.uv);
}