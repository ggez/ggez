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

fn srgb_to_linear(color: vec4<f32>) -> vec4<f32> {
    let low = color.rgb / vec3<f32>(12.92);
    let high = pow((color.rgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    return vec4<f32>(select(high, low, color.rgb <= vec3<f32>(0.04045)), color.a);
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.transform * my_uniforms.rotation * vec4<f32>(position, 0.0, 1.0);
    out.uv = mix(uniforms.src_rect.xy, uniforms.src_rect.zw, uv);
    out.color = srgb_to_linear(uniforms.color) * color;
    return out;
}
