//! A very simple shader example.

use cgmath;
use gfx::{self, *};
use ggez;

use ggez::event;
use ggez::graphics::{self, DrawMode};
use ggez::timer;
use ggez::{Context, GameResult};
use std::env;
use std::path;

// Define the input struct for our shader.
gfx_defines! {
    constant Dim {
        rate: f32 = "u_Rate",
    }
}

struct MainState {
    dim: Dim,
    shader: graphics::Shader<Dim>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let dim = Dim { rate: 0.5 };
        let shader = graphics::Shader::new(
            ctx,
            "/basic_150.glslv",
            "/dimmer_150.glslf",
            dim,
            "Dim",
            None,
        )?;
        Ok(MainState { dim, shader })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dim.rate = 0.5 + (((timer::ticks(ctx) as f32) / 100.0).cos() / 2.0);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            cgmath::Point2::new(100.0, 300.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (cgmath::Point2::new(0.0, 0.0),))?;

        {
            let _lock = graphics::use_shader(ctx, &self.shader);
            self.shader.send(ctx, self.dim)?;
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                cgmath::Point2::new(400.0, 300.0),
                100.0,
                2.0,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &circle, (cgmath::Point2::new(0.0, 0.0),))?;
        }

        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            cgmath::Point2::new(700.0, 300.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (cgmath::Point2::new(0.0, 0.0),))?;

        graphics::present(ctx)?;
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

    let cb = ggez::ContextBuilder::new("shader", "ggez").add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
