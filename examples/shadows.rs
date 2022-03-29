//! A more sophisticated example of how to use shaders
//! and canvas's to do 2D GPU shadows.

use ggez::graphics::{self, AsStd140, BlendMode, Canvas, Color, DrawParam, Shader};
use ggez::{event, graphics::ShaderParams};
use ggez::{Context, GameResult};
use glam::Vec2;
use std::env;
use std::path;

#[derive(AsStd140)]
struct Light {
    light_color: mint::Vector4<f32>,
    shadow_color: mint::Vector4<f32>,
    pos: mint::Vector2<f32>,
    screen_size: mint::Vector2<f32>,
    glow: f32,
    strength: f32,
}

/// Shader source for calculating a 1D shadow map that encodes half distances
/// in the red channel. The idea is that we scan X rays (X is the horizontal
/// size of the output) and calculate the distance to the nearest pixel at that
/// angle that has transparency above a threshold. The distance gets halved
/// and encoded in the red channel (it is halved because if the distance can be
/// greater than 1.0 - think bottom left to top right corner, that sqrt(1) and
/// will not get properly encoded).
const OCCLUSIONS_SHADER_SOURCE: &str = "
struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Light {
    light_color: vec4<f32>;
    shadow_color: vec4<f32>;
    pos: vec2<f32>;
    screen_size: vec2<f32>;
    glow: f32;
    strength: f32;
};

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(1), binding(1)]]
var s: sampler;

[[group(3), binding(0)]]
var<uniform> light: Light;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
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
";

/// Shader for drawing shadows based on a 1D shadow map. It takes current
/// fragment coordinates and converts them to polar coordinates centered
/// around the light source, using the angle to sample from the 1D shadow map.
/// If the distance from the light source is greater than the distance of the
/// closest reported shadow, then the output is the shadow color, else it calculates some
/// shadow based on the distance from light source based on strength and glow
/// uniform parameters.
const SHADOWS_SHADER_SOURCE: &str = "
struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Light {
    light_color: vec4<f32>;
    shadow_color: vec4<f32>;
    pos: vec2<f32>;
    screen_size: vec2<f32>;
    glow: f32;
    strength: f32;
};

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(1), binding(1)]]
var s: sampler;

[[group(3), binding(0)]]
var<uniform> light: Light;

fn degrees(x: f32) -> f32 {
    return x * 57.2957795130823208767981548141051703;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var rel = light.pos - in.uv;
    var theta = atan2(rel.y, rel.x);
    var ox = (theta + 3.1415926) / 6.2831853;
    var r = length(rel);
    var occl = 1.0 - step(r, textureSample(t, s, vec2<f32>(ox, 0.5)).r * 2.0);

    var g = light.screen_size / light.screen_size.y;
    var p = light.strength + light.glow;
    var d = distance(g * in.uv, g * light.pos);
    var intensity = 1.0 - clamp(p/(d*d), 0.0, 1.0);

    return light.shadow_color * vec4<f32>(vec3<f32>(mix(intensity, 1.0, occl)), 1.0);
}
";

/// Shader for drawing lights based on a 1D shadow map. It takes current
/// fragment coordinates and converts them to polar coordinates centered
/// around the light source, using the angle to sample from the 1D shadow map.
/// If the distance from the light source is greater than the distance of the
/// closest reported shadow, then the output is black, else it calculates some
/// light based on the distance from light source based on strength and glow
/// uniform parameters. It is meant to be used additively for drawing multiple
/// lights.
const LIGHTS_SHADER_SOURCE: &str = "
struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct Light {
    light_color: vec4<f32>;
    shadow_color: vec4<f32>;
    pos: vec2<f32>;
    screen_size: vec2<f32>;
    glow: f32;
    strength: f32;
};

[[group(1), binding(0)]]
var t: texture_2d<f32>;

[[group(1), binding(1)]]
var s: sampler;

[[group(3), binding(0)]]
var<uniform> light: Light;

fn degrees(x: f32) -> f32 {
    return x * 57.2957795130823208767981548141051703;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var rel = light.pos - in.uv;
    var theta = atan2(rel.y, rel.x);
    var ox = (theta + 3.1415926) / 6.2831853;
    var r = length(rel);
    var occl = step(r, textureSample(t, s, vec2<f32>(ox, 0.5)).r * 2.0);

    var g = light.screen_size / light.screen_size.y;
    var p = light.strength + light.glow;
    var d = distance(g * in.uv, g * light.pos);
    var intensity = clamp(p/(d*d), 0.0, 0.6);

    var blur = (2.5 / light.screen_size.x) * smoothStep(0.0, 1.0, r);
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
";

struct MainState {
    background: graphics::Image,
    tile: graphics::Image,
    torch: Light,
    torch_params: ShaderParams<Light>,
    static_light: Light,
    static_light_params: ShaderParams<Light>,
    foreground: graphics::ScreenImage,
    occlusions: graphics::Image,
    shadows: graphics::ScreenImage,
    lights: graphics::ScreenImage,
    occlusions_shader: Shader,
    shadows_shader: Shader,
    lights_shader: Shader,
}

/// The color cast things take when not illuminated
const AMBIENT_COLOR: [f32; 4] = [0.15, 0.12, 0.24, 1.0];
/// The default color for the static light
const STATIC_LIGHT_COLOR: [f32; 4] = [0.37, 0.69, 0.75, 1.0];
/// The default color for the mouse-controlled torch
const TORCH_COLOR: [f32; 4] = [0.80, 0.73, 0.44, 1.0];
/// The number of rays to cast to. Increasing this number will result in better
/// quality shadows. If you increase too much you might hit some GPU shader
/// hardware limits.
const LIGHT_RAY_COUNT: u16 = 1440;
/// The strength of the light - how far it shines
const LIGHT_STRENGTH: f32 = 0.0035;
/// The factor at which the light glows - just for fun
const LIGHT_GLOW_FACTOR: f32 = 0.00065;
/// The rate at which the glow effect oscillates
const LIGHT_GLOW_RATE: f32 = 0.9;

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let background = graphics::Image::from_path(&ctx.fs, &ctx.gfx, "/bg_top.png", true)?;
        let tile = graphics::Image::from_path(&ctx.fs, &ctx.gfx, "/tile.png", true)?;

        ctx.gfx.add_font(
            "LiberationMono",
            graphics::FontData::from_path(&ctx.fs, "/LiberationMono-Regular.ttf")?,
        );

        let screen_size = {
            let size = ctx.gfx.drawable_size();
            [size.0 as f32, size.1 as f32]
        };

        let torch = Light {
            pos: [0.0, 0.0].into(),
            light_color: TORCH_COLOR.into(),
            shadow_color: AMBIENT_COLOR.into(),
            screen_size: screen_size.into(),
            glow: 0.0,
            strength: LIGHT_STRENGTH,
        };
        let torch_params = ShaderParams::new(&mut ctx.gfx, &torch, &[], &[]);

        let (w, h) = ctx.gfx.size();
        let (x, y) = (100.0 / w as f32, 75.0 / h as f32);

        let static_light = Light {
            pos: [x, y].into(),
            light_color: STATIC_LIGHT_COLOR.into(),
            shadow_color: AMBIENT_COLOR.into(),
            screen_size: screen_size.into(),
            glow: 0.0,
            strength: LIGHT_STRENGTH,
        };
        let static_light_params = ShaderParams::new(&mut ctx.gfx, &static_light, &[], &[]);

        let color_format = ctx.gfx.surface_format();
        let foreground = graphics::ScreenImage::new(&ctx.gfx, None, 1., 1., 1);
        let occlusions =
            graphics::Image::new_canvas_image(&ctx.gfx, color_format, LIGHT_RAY_COUNT.into(), 1, 1);
        let shadows = graphics::ScreenImage::new(&ctx.gfx, None, 1., 1., 1);
        let lights = graphics::ScreenImage::new(&ctx.gfx, None, 1., 1., 1);

        let occlusions_shader = Shader::from_wgsl(&ctx.gfx, OCCLUSIONS_SHADER_SOURCE, "main");
        let shadows_shader = Shader::from_wgsl(&ctx.gfx, SHADOWS_SHADER_SOURCE, "main");
        let lights_shader = Shader::from_wgsl(&ctx.gfx, LIGHTS_SHADER_SOURCE, "main");

        Ok(MainState {
            background,
            tile,
            torch,
            torch_params,
            static_light,
            static_light_params,
            foreground,
            occlusions,
            shadows,
            lights,
            occlusions_shader,
            shadows_shader,
            lights_shader,
        })
    }
    fn render_light(
        &mut self,
        ctx: &mut Context,
        light: ShaderParams<Light>,
        origin: DrawParam,
        canvas_origin: DrawParam,
        clear: Option<graphics::Color>,
    ) -> GameResult {
        let foreground = self.foreground.image(&ctx.gfx);

        let size = ctx.gfx.drawable_size();
        // Now we want to run the occlusions shader to calculate our 1D shadow
        // distances into the `occlusions` canvas.
        let mut canvas = Canvas::from_image(&ctx.gfx, self.occlusions.clone(), None);
        canvas.set_shader(self.occlusions_shader.clone());
        canvas.set_shader_params(light.clone());
        canvas.draw(foreground, canvas_origin);
        canvas.finish(&mut ctx.gfx)?;

        // Now we render our shadow map and light map into their respective
        // canvases based on the occlusion map. These will then be drawn onto
        // the final render target using appropriate blending modes.
        let mut canvas = Canvas::from_screen_image(&ctx.gfx, &mut self.shadows, clear);
        canvas.set_shader(self.shadows_shader.clone());
        canvas.set_shader_params(light.clone());
        canvas.draw(
            self.occlusions.clone(),
            origin.image_scale(false).scale([size.0, size.1]),
        );
        canvas.finish(&mut ctx.gfx)?;

        let mut canvas = Canvas::from_screen_image(&ctx.gfx, &mut self.lights, clear);
        canvas.set_blend_mode(BlendMode::ADD);
        canvas.set_shader(self.lights_shader.clone());
        canvas.set_shader_params(light);
        canvas.draw(
            self.occlusions.clone(),
            origin.image_scale(false).scale([size.0, size.1]),
        );
        canvas.finish(&mut ctx.gfx)?;

        Ok(())
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.time.ticks() % 100 == 0 {
            println!("Average FPS: {}", ctx.time.fps());
        }

        self.torch.glow =
            LIGHT_GLOW_FACTOR * (ctx.time.time_since_start().as_secs_f32() * LIGHT_GLOW_RATE).cos();
        self.static_light.glow = LIGHT_GLOW_FACTOR
            * (ctx.time.time_since_start().as_secs_f32() * LIGHT_GLOW_RATE * 0.75).sin();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.torch_params.set_uniforms(&ctx.gfx, &self.torch);
        self.static_light_params
            .set_uniforms(&ctx.gfx, &self.static_light);

        let origin = DrawParam::new()
            .dest(Vec2::new(0.0, 0.0))
            .scale(Vec2::new(0.5, 0.5));
        let canvas_origin = DrawParam::new();

        // First thing we want to do it to render all the foreground items (that
        // will have shadows) onto their own Canvas (off-screen render). We will
        // use this canvas to:
        //  - run the occlusions shader to determine where the shadows are
        //  - render to screen once all the shadows are calculated and rendered
        let foreground = self.foreground.image(&ctx.gfx);
        let mut canvas = Canvas::from_image(&ctx.gfx, foreground, Color::new(0.0, 0.0, 0.0, 0.0));
        canvas.draw(
            self.tile.clone(),
            DrawParam::new().dest(Vec2::new(598.0, 124.0)),
        );
        canvas.draw(
            self.tile.clone(),
            DrawParam::new().dest(Vec2::new(92.0, 350.0)),
        );
        canvas.draw(
            self.tile.clone(),
            DrawParam::new().dest(Vec2::new(442.0, 468.0)).rotation(0.5),
        );
        canvas.draw_text(
            &[graphics::Text::new()
                .text("SHADOWS...")
                .size(48.0)
                .font("LiberationMono")],
            Vec2::new(50.0, 200.0),
            0.0,
            graphics::TextLayout::tl_single_line(),
            0,
        );
        canvas.finish(&mut ctx.gfx)?;

        // Then we draw our light and shadow maps
        self.render_light(
            ctx,
            self.torch_params.clone(),
            origin,
            canvas_origin,
            Some(Color::BLACK),
        )?;
        self.render_light(
            ctx,
            self.static_light_params.clone(),
            origin,
            canvas_origin,
            None,
        )?;

        // Now lets finally render to screen starting out with background, then
        // the shadows and lights overtop and finally our foreground.
        let shadows = self.shadows.image(&ctx.gfx);
        let foreground = self.foreground.image(&ctx.gfx);
        let lights = self.lights.image(&ctx.gfx);
        let mut canvas = Canvas::from_frame(&ctx.gfx, Color::WHITE);
        canvas.draw(self.background.clone(), DrawParam::default());
        canvas.set_blend_mode(BlendMode::MULTIPLY);
        canvas.draw(shadows, DrawParam::default());
        canvas.set_blend_mode(BlendMode::ALPHA);
        canvas.draw(foreground, DrawParam::default());
        canvas.set_blend_mode(BlendMode::ADD);
        canvas.draw(lights, DrawParam::default());
        // Uncomment following line to visualize the 1D occlusions canvas,
        // red pixels represent angles at which no shadows were found, and then
        // the greyscale pixels are the half distances of the nearest shadows to
        // the mouse position (equally encoded in all color channels).
        // canvas.draw(&self.occlusions, DrawParam::default());
        canvas.finish(&mut ctx.gfx)?;

        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        _xrel: f32,
        _yrel: f32,
    ) -> GameResult {
        let (w, h) = ctx.gfx.drawable_size();
        let (x, y) = (x / w as f32, y / h as f32);
        self.torch.pos = [x, y].into();
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("shadows", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
