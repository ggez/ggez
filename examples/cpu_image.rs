//! Example that shows loading an image to cpu and back.
use std::{env, path};

use ggez::{
    event,
    glam::*,
    graphics::{self, Image},
    Context, GameResult,
};

struct MainState {
    image: Image,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let og_image =
            graphics::Image::from_bytes(ctx, include_bytes!("../resources/wabbit_alpha.png"))?;
        let cpu_image = og_image.to_pixels(ctx)?;
        let image = Image::from_pixels(
            ctx,
            cpu_image.as_slice(),
            og_image.format(),
            og_image.width(),
            og_image.height(),
        );
        Ok(MainState { image })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(&self.image, Vec2::new(380.0, 380.0));

        canvas.finish(ctx)?;

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

    let cb = ggez::ContextBuilder::new("cpu_image", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
