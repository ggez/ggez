//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use std::{cell::Cell, path::PathBuf, sync::Arc};

use ggez::{
    coroutine::{yield_now, Loading},
    event,
    glam::*,
    graphics::{self, Color, Image},
    Context, Coroutine, GameResult,
};

struct MainState {
    pos_x: Arc<Cell<f32>>,
    circle: graphics::Mesh,
    coroutine: Coroutine,
    slow_coroutine: Coroutine<String>,
    image: Loading<Image>,
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
            coroutine: Coroutine::new(move |_ctx| async move {
                loop {
                    pos_x.set(pos_x.get() % 800.0 + 1.0);
                    yield_now().await
                }
            }),
            slow_coroutine: Coroutine::new(move |_ctx| async move {
                // wait 100 frames
                for _ in 0..100 {
                    yield_now().await
                }

                String::from("I came from a coroutine!")
            }),
            image: Image::from_path_async("/dragon4.png"),
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.coroutine.poll(ctx);
        if let Some(val) = self.slow_coroutine.poll(ctx) {
            println!("Coroutine says: \"{val}\"");
        }
        if let Some(_) = self.image.poll(ctx)? {
            println!("Loaded image..");
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(&self.circle, Vec2::new(self.pos_x.get(), 380.0));
        if let Some(loaded_mesh) = &self.image.result() {
            canvas.draw(loaded_mesh, Vec2::new(self.pos_x.get(), 100.0));
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
