struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2), interpolate(flat)]] idx: u32;
};

struct Uniforms {
    transform: mat4x4<f32>;
    pre_transform: mat4x4<f32>;
};

struct DrawParam {
    color: vec4<f32>;
    src_rect: vec4<f32>;
    transform: mat4x4<f32>;
};

struct InstanceArray {
    instances: [[stride(96)]] array<DrawParam>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(2), binding(0)]]
var s: sampler;

[[group(3), binding(0)]]
var<storage, read> instances: InstanceArray;

[[stage(vertex)]]
fn vs_main(
    [[builtin(instance_index)]] in_instance_index: u32,
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] uv: vec2<f32>,
    [[location(2)]] color: vec4<f32>,
) -> VertexOutput {
    var instance = instances.instances[in_instance_index];
    var identity = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );

    var out: VertexOutput;
    out.position = uniforms.transform * instance.transform * uniforms.pre_transform * vec4<f32>(position, 0.0, 1.0);;
    out.position = out.position / out.position.w;
    out.uv = mix(instance.src_rect.xy, instance.src_rect.zw, uv);
    out.color = color;
    out.idx = in_instance_index;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return instances.instances[in.idx].color * in.color * textureSample(t, s, in.uv);
}
