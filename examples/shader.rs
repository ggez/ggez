#[macro_use]
extern crate gfx;
extern crate ggez;

use ggez::*;
use ggez::graphics::{DrawMode, Point2};
use std::time::Duration;

gfx_defines!{
    constant Dim {
        rate: f32 = "u_Rate",
    }
}

struct MainState {
    dim: Dim,
    shader: graphics::PixelShader<Dim>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let dim = Dim { rate: 0.5 };
        let shader = graphics::PixelShader::new(ctx, "/dimmer_150.glslf", dim, "Dim")?;
        Ok(MainState { dim, shader })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        self.dim.rate = 0.5 + (((timer::get_ticks(ctx) as f32) / 100.0).cos() / 2.0);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        graphics::circle(ctx, DrawMode::Fill, Point2::new(100.0, 300.0), 100.0, 2.0)?;
        {
            let _lock = graphics::set_pixel_shader(ctx, &self.shader);
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
    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
