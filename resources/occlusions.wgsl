struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct Light {
    light_color: vec4<f32>,
    shadow_color: vec4<f32>,
    pos: vec2<f32>,
    screen_size: vec2<f32>,
    glow: f32,
    strength: f32,
}

@group(1) @binding(0)
var t: texture_2d<f32>;

@group(1) @binding(1)
var s: sampler;

@group(3) @binding(0)
var<uniform> light: Light;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var dist = 1.0;
    var theta = in.uv.x * 6.28318530718;
    var dir = vec2<f32>(cos(theta), sin(theta));
    for (var i: i32 = 0; i < 1024; i = i + 1) {
        var fi = f32(i);
        var r = fi / 1024.0;
        var rel = r * dir;
        var p = clamp(light.pos + rel, vec2<f32>(0.0), vec2<f32>(1.0));
        if (textureSample(t, s, p).a > 0.8) {
            dist = distance(light.pos, p) * 0.5;
            break;
        }
    }
    var others = select(dist, 0.0, dist == 1.0);
    return vec4<f32>(dist, others, others, 1.0);
}
