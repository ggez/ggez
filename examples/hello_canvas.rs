//! Basic hello world example, drawing
//! to a canvas.

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use glam::*;
use std::env;
use std::path;

struct MainState {
    canvas_image: graphics::ScreenImage,
    frames: usize,
    draw_with_canvas: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        ctx.gfx.add_font(
            "LiberationMono",
            graphics::FontData::from_path(&ctx.filesystem, "/LiberationMono-Regular.ttf")?,
        );
        let canvas_image = graphics::ScreenImage::new(&ctx.gfx, None, 1., 1., 1);

        let s = MainState {
            canvas_image,
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

        let text = graphics::Text::new()
            .font("LiberationMono")
            .text("Hello, world!")
            .size(48.);

        if self.draw_with_canvas {
            println!("Drawing with canvas");
            let canvas_image = self.canvas_image.image(&ctx.gfx);
            let mut canvas = graphics::Canvas::from_image(
                &ctx.gfx,
                canvas_image.clone(),
                graphics::Color::from((255, 255, 255, 128)),
            );

            canvas.draw_text(
                &[text.color(Color::from((0, 0, 0, 255)))],
                dest_point - vec2(15., 15.),
                0.0,
                graphics::TextLayout::tl_single_line(),
                0,
            );
            canvas.finish(&mut ctx.gfx)?;

            let mut canvas = graphics::Canvas::from_frame(&ctx.gfx, Color::from((64, 0, 0, 0)));
            canvas.draw(
                canvas_image.clone(),
                graphics::DrawParam::new().color(Color::from((255, 255, 255, 128))),
            );
            canvas.finish(&mut ctx.gfx)?;
        } else {
            println!("Drawing without canvas");
            let mut canvas =
                graphics::Canvas::from_frame(&ctx.gfx, Color::from([0.25, 0.0, 0.0, 1.0]));

            canvas.draw_text(
                &[text.color(Color::from((192, 128, 64, 255)))],
                dest_point,
                0.0,
                graphics::TextLayout::tl_single_line(),
                0,
            );

            canvas.finish(&mut ctx.gfx)?;
        }

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ctx.timer.fps());
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _keycode: ggez::event::KeyCode,
        _keymod: ggez::event::KeyMods,
        repeat: bool,
    ) -> GameResult {
        if !repeat {
            self.draw_with_canvas = !self.draw_with_canvas;
            println!("Canvas on: {}", self.draw_with_canvas);
        }
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

    let cb = ggez::ContextBuilder::new("hello_canvas", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
