//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use std::path::PathBuf;

use ggez::{
    coroutine::Loading,
    event,
    glam::*,
    graphics::{self, Image},
    Context, GameResult,
};

struct MainState {
    image: Loading<Image>,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        Ok(MainState {
            image: Image::from_path_async("@localhost:2020/dragon4.png"),
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.image.poll(ctx)?.is_some() {
            println!("Loaded image..");
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        if let Some(loaded_mesh) = &self.image.result() {
            canvas.draw(loaded_mesh, Vec2::new(100.0, 100.0));
        }

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("super_simple", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
