//! A very simple shader example.

use ggez::graphics::{self, Color, DrawMode};
use ggez::{event, graphics::AsStd140};
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
        let shader =
            graphics::Shader::from_wgsl(&ctx.gfx, include_str!("../resources/dimmer.wgsl"), "main");
        let params = graphics::ShaderParams::new(&mut ctx.gfx, &dim, &[], &[]);
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
        let mut canvas = graphics::Canvas::from_frame(&ctx.gfx, Color::from([0.1, 0.2, 0.3, 1.0]));

        let circle = graphics::Mesh::new_circle(
            &ctx.gfx,
            DrawMode::fill(),
            glam::Vec2::new(100.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        canvas.draw(&circle, glam::Vec2::new(0.0, 0.0));

        self.params.set_uniforms(&ctx.gfx, &self.dim);
        canvas.set_shader(self.shader.clone());
        canvas.set_shader_params(self.params.clone());
        let circle = graphics::Mesh::new_circle(
            &ctx.gfx,
            DrawMode::fill(),
            glam::Vec2::new(400.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        canvas.draw(&circle, glam::Vec2::new(0.0, 0.0));

        canvas.set_default_shader();
        let circle = graphics::Mesh::new_circle(
            &ctx.gfx,
            DrawMode::fill(),
            glam::Vec2::new(700.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        canvas.draw(&circle, glam::Vec2::new(0.0, 0.0));

        canvas.finish(&mut ctx.gfx)
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
