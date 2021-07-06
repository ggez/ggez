//! Basic hello world example, drawing
//! to a canvas.

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use glam::*;
use std::env;
use std::path;

struct MainState {
    text: graphics::Text,
    canvas: graphics::Canvas,
    frames: usize,
    draw_with_canvas: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        let font = graphics::Font::new(ctx, "/LiberationMono-Regular.ttf")?;
        let text = graphics::Text::new(("Hello world!", font, 48.0));
        let canvas = graphics::Canvas::with_window_size(ctx)?;

        let s = MainState {
            text,
            canvas,
            draw_with_canvas: false,
            frames: 0,
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let dest_point = Vec2::new(10.0, 10.0);

        if self.draw_with_canvas {
            println!("Drawing with canvas");
            graphics::clear(ctx, graphics::Color::from((64, 0, 0, 0)));

            graphics::set_canvas(ctx, Some(&self.canvas));
            graphics::clear(ctx, graphics::Color::from((255, 255, 255, 128)));

            graphics::draw(
                ctx,
                &self.text,
                graphics::DrawParam::new()
                    .dest(dest_point)
                    .color(Color::from((0, 0, 0, 255))),
            )?;
            graphics::set_canvas(ctx, None);

            graphics::draw(
                ctx,
                &self.canvas,
                graphics::DrawParam::new().color(Color::from((255, 255, 255, 128))),
            )?;
        } else {
            println!("Drawing without canvas");
            graphics::set_canvas(ctx, None);
            graphics::clear(ctx, [0.25, 0.0, 0.0, 1.0].into());

            graphics::draw(
                ctx,
                &self.text,
                graphics::DrawParam::new()
                    .dest(dest_point)
                    .color(Color::from((192, 128, 64, 255))),
            )?;
        }

        graphics::present(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _keycode: ggez::event::KeyCode,
        _keymod: ggez::event::KeyMods,
        repeat: bool,
    ) {
        if !repeat {
            self.draw_with_canvas = !self.draw_with_canvas;
            println!("Canvas on: {}", self.draw_with_canvas);
        }
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

    let cb = ggez::ContextBuilder::new("hello_canvas", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
