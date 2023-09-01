//! An example of how to use an `InstanceArray`.
//!
//! You really want to run this one in release mode.
#![allow(clippy::unnecessary_wraps)]

use ggez::coroutine::Loading;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use std::env;
use std::f32::consts::TAU;
use std::path;

struct MainState {
    instances: graphics::InstanceArray,
    image: Loading<graphics::Image>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image = graphics::Image::from_path_async("/tile.png");
        let mut instances = graphics::InstanceArray::new(ctx, None);
        instances.resize(ctx, 10 * 10);
        Ok(MainState { instances, image })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(image) = self.image.poll(ctx)? {
            self.instances.set_image(image.clone());
        }

        if ctx.time.ticks() % 100 == 0 {
            println!("Delta frame time: {:?} ", ctx.time.delta());
            println!("Average FPS: {}", ctx.time.fps());
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let time = (ctx.time.time_since_start().as_secs_f64() * 1000.0) as u32;
        let cycle = 10_000;
        self.instances.set((0..10).flat_map(|x| {
            (0..10).map(move |y| {
                let x = x as f32;
                let y = y as f32;
                graphics::DrawParam::new()
                    .dest(Vec2::new(x * 10.0, y * 10.0))
                    .scale(Vec2::new(
                        ((time % cycle * 2) as f32 / cycle as f32 * TAU).cos().abs() * 0.0625,
                        ((time % cycle * 2) as f32 / cycle as f32 * TAU).cos().abs() * 0.0625,
                    ))
                    .rotation(-2.0 * ((time % cycle) as f32 / cycle as f32 * TAU))
            })
        }));

        let param = graphics::DrawParam::new()
            .dest(Vec2::new(
                ((time % cycle) as f32 / cycle as f32 * TAU).cos() * 50.0 + 100.0,
                ((time % cycle) as f32 / cycle as f32 * TAU).sin() * 50.0 - 150.0,
            ))
            .scale(Vec2::new(
                ((time % cycle) as f32 / cycle as f32 * TAU).sin().abs() * 2.0 + 1.0,
                ((time % cycle) as f32 / cycle as f32 * TAU).sin().abs() * 2.0 + 1.0,
            ))
            .rotation((time % cycle) as f32 / cycle as f32 * TAU)
            // src has no influence when applied globally to a spritebatch
            .src(graphics::Rect::new(0.005, 0.005, 0.005, 0.005));
        canvas.draw(&self.instances, param);

        canvas.finish(ctx)
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
