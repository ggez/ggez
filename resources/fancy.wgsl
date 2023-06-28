struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec4<f32>,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) vertex_color: vec4<f32>,
}


@group(1) @binding(0)
var<uniform> camera: CameraUniform;


@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coord = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.color = instance.color;
    out.vertex_color = model.color;
    return out;
}

@group(0) @binding(0)
var t_color: texture_2d<f32>;

@group(0) @binding(1)
var s_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex = textureSample(t_color, s_sampler, in.tex_coord);
    let tex_col = mix(mix(tex, vec4<f32>(in.color.xyz, 1.0), in.color.w), vec4<f32>(in.vertex_color.xyz, 1.0), in.vertex_color.w);
    var blend = dot(in.tex_coord - vec2<f32>(0.5, 0.5), in.tex_coord - vec2<f32>(0.5, 0.5));
    return mix(tex_col, vec4<f32>(0.0, 0.0, 0.0, 0.0), blend);
}
