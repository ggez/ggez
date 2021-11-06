//! An example of how to use a `SpriteBatch`.
//!
//! You really want to run this one in release mode.
#![allow(clippy::unnecessary_wraps)]

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::timer;
use ggez::{Context, GameResult};
use glam::*;
use std::env;
use std::path;

struct MainState {
    spritebatch: graphics::spritebatch::SpriteBatch,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image = graphics::Image::new(ctx, "/tile.png").unwrap();
        let batch = graphics::spritebatch::SpriteBatch::new(image);
        let s = MainState { spritebatch: batch };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if timer::ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::delta(ctx));
            println!("Average FPS: {}", timer::fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        let time = (timer::duration_to_f64(timer::time_since_start(ctx)) * 1000.0) as u32;
        let cycle = 10_000;
        for x in 0..150 {
            for y in 0..150 {
                let x = x as f32;
                let y = y as f32;
                let p = graphics::DrawParam::new()
                    .dest(Vec2::new(x * 10.0, y * 10.0))
                    .scale(Vec2::new(
                        ((time % cycle * 2) as f32 / cycle as f32 * 6.28)
                            .cos()
                            .abs()
                            * 0.0625,
                        ((time % cycle * 2) as f32 / cycle as f32 * 6.28)
                            .cos()
                            .abs()
                            * 0.0625,
                    ))
                    .rotation(-2.0 * ((time % cycle) as f32 / cycle as f32 * 6.28));
                self.spritebatch.add(p);
            }
        }
        let param = graphics::DrawParam::new()
            .dest(Vec2::new(
                ((time % cycle) as f32 / cycle as f32 * 6.28).cos() * 50.0 + 100.0,
                ((time % cycle) as f32 / cycle as f32 * 6.28).sin() * 50.0 - 150.0,
            ))
            .scale(Vec2::new(
                ((time % cycle) as f32 / cycle as f32 * 6.28).sin().abs() * 2.0 + 1.0,
                ((time % cycle) as f32 / cycle as f32 * 6.28).sin().abs() * 2.0 + 1.0,
            ))
            .rotation((time % cycle) as f32 / cycle as f32 * 6.28)
            .offset(Vec2::new(750.0, 750.0))
            // src has no influence when applied globally to a spritebatch
            .src(graphics::Rect::new(0.005, 0.005, 0.005, 0.005));
        graphics::draw(ctx, &self.spritebatch, param)?;
        self.spritebatch.clear();

        graphics::present(ctx)?;
        Ok(())
    }
}

// Creating a gamestate depends on having an SDL context to load resources.
// Creating a context depends on loading a config file.
// Loading a config file depends on having FS (or we can just fake our way around it
// by creating an FS and then throwing it away; the costs are not huge.)
pub fn main() -> GameResult {
    if cfg!(debug_assertions) && env::var("yes_i_really_want_debug_mode").is_err() {
        eprintln!(
            "Note: Release mode will improve performance greatly.\n    \
             e.g. use `cargo run --example spritebatch --release`"
        );
    }

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("spritebatch", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
