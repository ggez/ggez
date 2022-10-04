struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct Uniforms {
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

// text drawing works by submitting a draw with 4 vert count, and n (# glyphs) instances, thus 4 * n vertices.
// however, we only store 1 vertex per glyph (memory efficiency), so we repeat the same vertex data 4 times.
@vertex
fn vs_main(
    // 0-3 - the vertex ID (because we draw with a vert count of 4)
    @builtin(vertex_index) idx: u32,
    // these vertex parameters update *per instance*, not per vertex.
    // these will remain the same within a single glyph.
    @location(0) rect: vec4<f32>,
    @location(1) uv: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(4) transform_c1: vec4<f32>,
    @location(5) transform_c2: vec4<f32>,
    @location(6) transform_c3: vec4<f32>,
    @location(3) transform_c0: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    // this is to select the x,y (and also u,v) coordinates for each corner of the glyph rect,
    // based on the vertex ID
    var x = select(rect.x, rect.z, idx % 2u == 1u);
    var y = select(rect.y, rect.w, idx < 2u);

    var u = select(uv.x, uv.z, idx % 2u == 1u);
    var v = select(uv.y, uv.w, idx < 2u);

    var transform = mat4x4<f32>(
        transform_c0,
        transform_c1,
        transform_c2,
        transform_c3,
    );

    out.position = uniforms.transform * transform * vec4<f32>(x, y, 0., 1.);
    out.position = out.position / out.position.w;
    out.uv = vec2<f32>(u, v);
    out.color = color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * textureSample(t, s, in.uv).rrrr;
}
