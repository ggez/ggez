struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct DrawUniforms {
    color: vec4<f32>,
    src_rect: vec4<f32>,
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: DrawUniforms;

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.transform * vec4<f32>(position, 0.0, 1.0);
    out.uv = mix(uniforms.src_rect.xy, uniforms.src_rect.zw, uv);
    
    // convert to linear
    var threshold = uniforms.color.rgb < vec3<f32>(0.04045);
    var hi = pow((uniforms.color.rgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    var lo = uniforms.color.rgb * vec3<f32>(12.92);
    var linear_color = vec4<f32>(select(hi, lo, threshold), uniforms.color.a);
    out.color = linear_color * color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * textureSample(t, s, in.uv);
}
