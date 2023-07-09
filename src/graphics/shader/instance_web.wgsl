struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct Uniforms {
    transform: mat4x4<f32>,
    color: vec4<f32>,
    scale: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

// Ordered draws on web sort their instance buffer on the CPU before upload, so this
// shader matches instance_unordered_web.wgsl. See notes there for per-instance attribute layout.
@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) inst_color: vec4<f32>,
    @location(4) inst_src_rect: vec4<f32>,
    @location(5) inst_t0: vec4<f32>,
    @location(6) inst_t1: vec4<f32>,
    @location(7) inst_t2: vec4<f32>,
    @location(8) inst_t3: vec4<f32>,
) -> VertexOutput {
    var inst_transform = mat4x4<f32>(inst_t0, inst_t1, inst_t2, inst_t3);

    var scale_x = select(1.0, uniforms.scale.x * (inst_src_rect.z - inst_src_rect.x), uniforms.scale.x > 0.0);
    var scale_y = select(1.0, uniforms.scale.y * (inst_src_rect.w - inst_src_rect.y), uniforms.scale.x > 0.0);
    var scale_mat = mat4x4<f32>(
        scale_x,
        0.0,
        0.0,
        0.0,
        0.0,
        scale_y,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );

    var out: VertexOutput;
    out.position = uniforms.transform * inst_transform * scale_mat * vec4<f32>(position, 0.0, 1.0);
    out.uv = mix(inst_src_rect.xy, inst_src_rect.zw, uv);
    out.color = uniforms.color * inst_color * color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * textureSample(t, s, in.uv);
}
