#[macro_use]
extern crate gfx;
extern crate ggez;

use ggez::*;
use ggez::graphics::{DrawMode, Point2};
use std::env;
use std::path;

gfx_defines!{
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
        let shader = graphics::Shader::new(ctx,
                                           "/basic_150.glslv",
                                           "/dimmer_150.glslf",
                                           dim,
                                           "Dim",
                                           None)?;
        Ok(MainState {
            dim: dim,
            shader: shader,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dim.rate = 0.5 + (((timer::get_ticks(ctx) as f32) / 100.0).cos() / 2.0);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        graphics::circle(ctx, DrawMode::Fill, Point2::new(100.0, 300.0), 100.0, 2.0)?;

        {
            let _lock = graphics::use_shader(ctx, &self.shader);
            self.shader.send(ctx, self.dim.clone())?;
            graphics::circle(ctx, DrawMode::Fill, Point2::new(400.0, 300.0), 100.0, 2.0)?;
        }

        graphics::circle(ctx, DrawMode::Fill, Point2::new(700.0, 300.0), 100.0, 2.0)?;

        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("shader", "ggez", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so
    // we we look in the cargo project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
