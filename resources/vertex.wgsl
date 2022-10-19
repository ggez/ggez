struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct MyCustomDrawUniforms {
    rotation: mat4x4<f32>,
}

struct GgezDrawUniforms {
    color: vec4<f32>,
    src_rect: vec4<f32>,
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: GgezDrawUniforms;

@group(3) @binding(0)
var<uniform> my_uniforms: MyCustomDrawUniforms;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.transform * my_uniforms.rotation * vec4<f32>(position, 0.0, 1.0);
    out.uv = mix(uniforms.src_rect.xy, uniforms.src_rect.zw, uv);
    out.color = uniforms.color * color;
    return out;
}