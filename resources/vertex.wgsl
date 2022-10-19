struct DrawUniforms {
    transform: mat4x4<f32>,
    rotation: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: DrawUniforms;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> @builtin(position) vec4<f32> {
    //return uniforms.transform * uniforms.rotation * vec4<f32>(position, 0.0, 1.0);
    return uniforms.transform * vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}