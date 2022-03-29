struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Uniforms {
    transform: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(1), binding(1)]]
var s: sampler;

[[stage(vertex)]]
fn vs_main(
    [[builtin(vertex_index)]] idx: u32,
    [[location(0)]] rect: vec4<f32>,
    [[location(1)]] uv: vec4<f32>,
    [[location(2)]] color: vec4<f32>,
    [[location(3)]] transform: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    var x = select(rect.x, rect.z, idx % 2u == 1u);
    var y = select(rect.y, rect.w, idx < 2u);

    var u = select(uv.x, uv.z, idx % 2u == 1u);
    var v = select(uv.y, uv.w, idx < 2u);

    var origin = vec4<f32>(transform.xy, 0.0, 0.0);
    var rotation = transform.z;
    // wgsl lacks sincos
    var rsin = sin(rotation);
    var rcos = cos(rotation);
    var rotmat = mat4x4<f32>(
        vec4<f32>(rcos, rsin, 0.0, 0.0),
        vec4<f32>(-rsin, rcos, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );

    out.position = uniforms.transform * (rotmat * (vec4<f32>(x, y, 0., 1.) - origin) + origin);
    out.uv = vec2<f32>(u, v);
    out.color = color;

    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color * textureSample(t, s, in.uv).rrrr;
}
