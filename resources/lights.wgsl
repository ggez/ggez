struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

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

fn degrees(x: f32) -> f32 {
    return x * 57.2957795130823208767981548141051703;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var rel = light.pos - in.uv;
    var theta = atan2(rel.y, rel.x);
    var ox = (theta + 3.1415926) / 6.2831853;
    var r = length(rel);
    var occl = step(r, textureSample(t, s, vec2<f32>(ox, 0.5)).r * 2.0);

    var g = light.screen_size / light.screen_size.y;
    var p = light.strength + light.glow;
    var d = distance(g * in.uv, g * light.pos);
    var intensity = clamp(p / (d * d), 0.0, 0.6);

    var blur = (2.5 / light.screen_size.x) * smoothstep(0.0, 1.0, r);
    var sum = 0.0;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox - 4.0 * blur, 0.5)).r * 2.0) * 0.05;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox - 3.0 * blur, 0.5)).r * 2.0) * 0.09;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox - 2.0 * blur, 0.5)).r * 2.0) * 0.12;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox - 1.0 * blur, 0.5)).r * 2.0) * 0.15;
    sum = sum + occl * 0.16;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox + 1.0 * blur, 0.5)).r * 2.0) * 0.15;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox + 2.0 * blur, 0.5)).r * 2.0) * 0.12;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox + 3.0 * blur, 0.5)).r * 2.0) * 0.09;
    sum = sum + step(r, textureSample(t, s, vec2<f32>(ox + 4.0 * blur, 0.5)).r * 2.0) * 0.05;

    return light.light_color * vec4<f32>(vec3<f32>(sum * intensity), 1.0);
}
