//! A more sophisticated example of how to use shaders
//! and canvas's to do 2D GPU shadows.

use crevice::std140::AsStd140;
use ggez::glam::Vec2;
use ggez::graphics::{
    self, BlendMode, Canvas, Color, DrawParam, Shader, ShaderBuilder, ShaderParamsBuilder,
};
use ggez::{event, graphics::ShaderParams};
use ggez::{Context, GameResult};
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
const OCCLUSIONS_SHADER_SOURCE: &str = include_str!("../resources/occlusions.wgsl");

/// Shader for drawing shadows based on a 1D shadow map. It takes current
/// fragment coordinates and converts them to polar coordinates centered
/// around the light source, using the angle to sample from the 1D shadow map.
/// If the distance from the light source is greater than the distance of the
/// closest reported shadow, then the output is the shadow color, else it calculates some
/// shadow based on the distance from light source based on strength and glow
/// uniform parameters.
const SHADOWS_SHADER_SOURCE: &str = include_str!("../resources/shadows.wgsl");

/// Shader for drawing lights based on a 1D shadow map. It takes current
/// fragment coordinates and converts them to polar coordinates centered
/// around the light source, using the angle to sample from the 1D shadow map.
/// If the distance from the light source is greater than the distance of the
/// closest reported shadow, then the output is black, else it calculates some
/// light based on the distance from light source based on strength and glow
/// uniform parameters. It is meant to be used additively for drawing multiple
/// lights.
const LIGHTS_SHADER_SOURCE: &str = include_str!("../resources/lights.wgsl");

struct MainState {
    background: graphics::Image,
    tile: graphics::Image,
    light_list: Vec<(Light, ShaderParams<Light>)>,
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
        let background = graphics::Image::from_path(ctx, "/bg_top.png")?;
        let tile = graphics::Image::from_path(ctx, "/tile.png")?;

        let screen_size = {
            let size = ctx.gfx.drawable_size();
            [size.0, size.1]
        };

        let torch = Light {
            pos: [0.0, 0.0].into(),
            light_color: TORCH_COLOR.into(),
            shadow_color: AMBIENT_COLOR.into(),
            screen_size: screen_size.into(),
            glow: 0.0,
            strength: LIGHT_STRENGTH,
        };
        let torch_params = ShaderParamsBuilder::new(&torch).build(ctx);

        let (w, h) = ctx.gfx.size();
        let (x, y) = (100.0 / w, 75.0 / h);

        let static_light = Light {
            pos: [x, y].into(),
            light_color: STATIC_LIGHT_COLOR.into(),
            shadow_color: AMBIENT_COLOR.into(),
            screen_size: screen_size.into(),
            glow: 0.0,
            strength: LIGHT_STRENGTH,
        };
        let static_light_params = ShaderParamsBuilder::new(&static_light).build(ctx);

        let light_list = vec![(torch, torch_params), (static_light, static_light_params)];

        let color_format = ctx.gfx.surface_format();
        let foreground = graphics::ScreenImage::new(ctx, None, 1., 1., 1);
        let occlusions =
            graphics::Image::new_canvas_image(ctx, color_format, LIGHT_RAY_COUNT.into(), 1, 1);
        let shadows = graphics::ScreenImage::new(ctx, None, 1., 1., 1);
        let lights = graphics::ScreenImage::new(ctx, None, 1., 1., 1);

        let occlusions_shader = ShaderBuilder::new_wgsl()
            .fragment_code(OCCLUSIONS_SHADER_SOURCE)
            .build(&ctx.gfx)?;
        let shadows_shader = ShaderBuilder::new_wgsl()
            .fragment_code(SHADOWS_SHADER_SOURCE)
            .build(&ctx.gfx)?;
        let lights_shader = ShaderBuilder::new_wgsl()
            .fragment_code(LIGHTS_SHADER_SOURCE)
            .build(&ctx.gfx)?;

        Ok(MainState {
            background,
            tile,
            light_list,
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
        light_idx: usize,
        origin: DrawParam,
        canvas_origin: DrawParam,
        clear: Option<graphics::Color>,
    ) -> GameResult {
        let foreground = self.foreground.image(ctx);

        let size = ctx.gfx.drawable_size();

        // Now we want to run the occlusions shader to calculate our 1D shadow
        // distances into the `occlusions` canvas.
        let mut canvas = Canvas::from_image(ctx, self.occlusions.clone(), None);
        canvas.set_screen_coordinates(graphics::Rect::new(0., 0., size.0, size.1));
        canvas.set_shader(&self.occlusions_shader);
        canvas.set_shader_params(&self.light_list[light_idx].1);
        canvas.draw(&foreground, canvas_origin);
        canvas.finish(ctx)?;

        // Now we render our shadow map and light map into their respective
        // canvases based on the occlusion map. These will then be drawn onto
        // the final render target using appropriate blending modes.
        let mut canvas = Canvas::from_screen_image(ctx, &mut self.shadows, clear);
        canvas.set_screen_coordinates(graphics::Rect::new(0., 0., size.0, size.1));
        canvas.set_shader(&self.shadows_shader);
        canvas.set_shader_params(&self.light_list[light_idx].1);
        canvas.draw(
            &self.occlusions,
            origin.scale([
                size.0 / self.occlusions.width() as f32,
                size.1 / self.occlusions.height() as f32,
            ]),
        );
        canvas.finish(ctx)?;

        let mut canvas = Canvas::from_screen_image(ctx, &mut self.lights, clear);
        canvas.set_screen_coordinates(graphics::Rect::new(0., 0., size.0, size.1));
        canvas.set_blend_mode(BlendMode::ADD);
        canvas.set_shader(&self.lights_shader);
        canvas.set_shader_params(&self.light_list[light_idx].1);
        canvas.draw(
            &self.occlusions,
            origin.scale([
                size.0 / self.occlusions.width() as f32,
                size.1 / self.occlusions.height() as f32,
            ]),
        );
        canvas.finish(ctx)?;

        Ok(())
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.time.ticks() % 100 == 0 {
            println!("Average FPS: {}", ctx.time.fps());
        }

        self.light_list[0].0.glow =
            LIGHT_GLOW_FACTOR * (ctx.time.time_since_start().as_secs_f32() * LIGHT_GLOW_RATE).cos();
        self.light_list[1].0.glow = LIGHT_GLOW_FACTOR
            * (ctx.time.time_since_start().as_secs_f32() * LIGHT_GLOW_RATE * 0.75).sin();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        for (light, light_params) in &mut self.light_list {
            light_params.set_uniforms(ctx, light);
        }

        let origin = DrawParam::new()
            .dest(Vec2::new(0.0, 0.0))
            .scale(Vec2::new(0.5, 0.5));
        let canvas_origin = DrawParam::new();

        // First thing we want to do it to render all the foreground items (that
        // will have shadows) onto their own Canvas (off-screen render). We will
        // use this canvas to:
        //  - run the occlusions shader to determine where the shadows are
        //  - render to screen once all the shadows are calculated and rendered
        let foreground = self.foreground.image(ctx);
        let mut canvas = Canvas::from_image(ctx, foreground, Color::new(0.0, 0.0, 0.0, 0.0));
        canvas.draw(&self.tile, DrawParam::new().dest(Vec2::new(598.0, 124.0)));
        canvas.draw(&self.tile, DrawParam::new().dest(Vec2::new(92.0, 350.0)));
        canvas.draw(
            &self.tile,
            DrawParam::new().dest(Vec2::new(442.0, 468.0)).rotation(0.5),
        );
        canvas.draw(
            graphics::Text::new("SHADOWS...").set_scale(48.),
            graphics::DrawParam::from([50., 200.]),
        );
        canvas.finish(ctx)?;

        // Then we draw our light and shadow maps
        for i in 0..self.light_list.len() {
            self.render_light(
                ctx,
                i,
                origin,
                canvas_origin,
                if i > 0 { None } else { Some(Color::BLACK) },
            )?;
        }

        // Now lets finally render to screen starting out with background, then
        // the shadows and lights overtop and finally our foreground.
        let shadows = self.shadows.image(ctx);
        let foreground = self.foreground.image(ctx);
        let lights = self.lights.image(ctx);
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);
        canvas.draw(&self.background, DrawParam::default());
        canvas.set_blend_mode(BlendMode::MULTIPLY);
        canvas.draw(&shadows, DrawParam::default());
        canvas.set_blend_mode(BlendMode::ALPHA);
        canvas.draw(&foreground, DrawParam::default());
        canvas.set_blend_mode(BlendMode::ADD);
        canvas.draw(&lights, DrawParam::default());
        // Uncomment following line to visualize the 1D occlusions canvas,
        // red pixels represent angles at which no shadows were found, and then
        // the greyscale pixels are the half distances of the nearest shadows to
        // the mouse position (equally encoded in all color channels).
        // canvas.draw(&self.occlusions, DrawParam::default());
        canvas.finish(ctx)?;

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
        let (x, y) = (x / w, y / h);
        self.light_list[0].0.pos = [x, y].into();
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
