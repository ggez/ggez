struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Uniforms {
    transform: mat4x4<f32>;
    color: vec4<f32>;
};

struct DrawParam {
    color: vec4<f32>;
    src_rect: vec4<f32>;
    transform: mat4x4<f32>;
};

struct InstanceArray {
    instances: [[stride(96)]] array<DrawParam>;
};

struct InstanceArrayIndices {
    indices: [[stride(4)]] array<u32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(1), binding(1)]]
var s: sampler;

[[group(2), binding(0)]]
var<storage, read> instances: InstanceArray;

[[group(2), binding(1)]]
var<storage, read> indices: InstanceArrayIndices;

[[stage(vertex)]]
fn vs_main(
    [[builtin(instance_index)]] in_instance_index: u32,
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] uv: vec2<f32>,
    [[location(2)]] color: vec4<f32>,
) -> VertexOutput {
    var index = indices.indices[in_instance_index];
    var instance = instances.instances[index];

    var out: VertexOutput;
    out.position = uniforms.transform
        * instance.transform
        * vec4<f32>(position, 0.0, 1.0);
    out.uv = mix(instance.src_rect.xy, instance.src_rect.zw, uv);
    out.color = uniforms.color * instance.color * color;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color * textureSample(t, s, in.uv);
}
