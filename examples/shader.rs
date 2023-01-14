//! A very simple shader example.

use crevice::std140::AsStd140;
use ggez::event;
use ggez::glam::Vec2;
use ggez::graphics::{self, Color, DrawMode};
use ggez::{Context, GameResult};
use std::env;
use std::path;

#[derive(AsStd140)]
struct Dim {
    rate: f32,
}

struct MainState {
    dim: Dim,
    shader: graphics::Shader,
    params: graphics::ShaderParams<Dim>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let dim = Dim { rate: 0.5 };
        let shader = graphics::ShaderBuilder::new_wgsl()
            .fragment_path("/dimmer.wgsl")
            .build(&ctx.gfx)?;
        let params = graphics::ShaderParamsBuilder::new(&dim).build(ctx);
        Ok(MainState {
            dim,
            shader,
            params,
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dim.rate = 0.5 + (((ctx.time.ticks() as f32) / 100.0).cos() / 2.0);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));

        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::new(100.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        canvas.draw(&circle, Vec2::new(0.0, 0.0));

        self.params.set_uniforms(ctx, &self.dim);
        canvas.set_shader(&self.shader);
        canvas.set_shader_params(&self.params);
        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::new(400.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        canvas.draw(&circle, Vec2::new(0.0, 0.0));

        canvas.set_default_shader();
        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::new(700.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        canvas.draw(&circle, Vec2::new(0.0, 0.0));

        canvas.finish(ctx)
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

    let cb = ggez::ContextBuilder::new("shader", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
