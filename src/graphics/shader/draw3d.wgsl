struct DrawUniforms {
    color: vec4<f32>,
    // src_rect: vec4<f32>,
    model_transform: mat4x4<f32>,
    camera_transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: DrawUniforms;

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) vertex_color: vec4<f32>
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = model.tex_coords;
    out.clip_position = uniforms.camera_transform * uniforms.model_transform * vec4<f32>(model.position, 1.0);

    // convert to linear
    var threshold = uniforms.color.rgb < vec3<f32>(0.04045);
    var hi = pow((uniforms.color.rgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    var lo = uniforms.color.rgb * vec3<f32>(1.0 / 12.92);
    var linear_color = vec4<f32>(select(hi, lo, threshold), uniforms.color.a);
    out.color = linear_color;
    
    out.vertex_color = model.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex = textureSample(t, s, in.tex_coord);
    return tex;
    // return mix(tex, vec4<f32>(in.color.xyz, 1.0), in.color.w) * in.vertex_color;
}
