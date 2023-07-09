//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use std::{cell::Cell, sync::Arc};

use ggez::{
    coroutine::yield_now,
    event,
    glam::*,
    graphics::{self, Color},
    Context, Coroutine, GameResult,
};

struct MainState {
    pos_x: Arc<Cell<f32>>,
    circle: graphics::Mesh,
    coroutine: Coroutine,
    slow_coroutine: Coroutine<String>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            100.0,
            2.0,
            Color::WHITE,
        )?;

        let pos_x = Arc::new(Cell::new(0.0));

        Ok(MainState {
            pos_x: Arc::clone(&pos_x),
            circle,
            coroutine: Coroutine::new(async move {
                loop {
                    pos_x.set(pos_x.get() % 800.0 + 1.0);
                    yield_now().await
                }
            }),
            slow_coroutine: Coroutine::new(async move {
                // wait 100 frames
                for _ in 0..100 {
                    yield_now().await
                }

                String::from("I came from a coroutine!")
            }),
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.coroutine.poll();
        if let Some(val) = self.slow_coroutine.poll() {
            println!("Coroutine says: \"{val}\"");
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(&self.circle, Vec2::new(self.pos_x.get(), 380.0));

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
