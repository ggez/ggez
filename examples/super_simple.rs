//! The simplest possible example that does something.

extern crate ggez;

use ggez::conf;
use ggez::event;
use ggez::graphics::{self, DrawMode, Point2};
use ggez::{Context, GameResult};

struct MainState {
    pos_x: f32,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { pos_x: 0.0 };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx);
        graphics::circle(
            ctx,
            DrawMode::Fill,
            Point2::new(self.pos_x, 380.0),
            100.0,
            2.0,
        )?;
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let c = conf::Conf::new();
    let (ctx, events_loop) = &mut Context::load_from_conf("super_simple", "ggez", c)?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, events_loop, state)
}
