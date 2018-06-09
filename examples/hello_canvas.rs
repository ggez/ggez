//! Basic hello world example.

extern crate ggez;

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
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
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 48)?;
        let text = graphics::Text::new(ctx, "Hello world!", &font)?;
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

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let dest_point = graphics::Point2::new(10.0, 10.0);

        if self.draw_with_canvas {
            println!("Drawing with canvas");
            graphics::set_background_color(ctx, graphics::Color::from((64, 0, 0, 0)));
            graphics::clear(ctx);

            graphics::set_canvas(ctx, Some(&self.canvas));
            graphics::set_background_color(ctx, graphics::Color::from((255, 255, 255, 128)));
            graphics::clear(ctx);

            graphics::draw_ex(
                ctx,
                &self.text,
                graphics::DrawParam {
                    dest: dest_point,
                    color: Some(graphics::Color::from((0, 0, 0, 255))),
                    ..Default::default()
                },
            )?;
            graphics::set_canvas(ctx, None);

            // graphics::draw(ctx, &self.canvas, graphics::Point2::new(0.0, 0.0), 0.0)?;

            graphics::draw_ex(
                ctx,
                &self.canvas,
                graphics::DrawParam {
                    color: Some(graphics::Color::from((255, 255, 255, 128))),
                    ..Default::default()
                },
            )?;
        } else {
            println!("Drawing without canvas");
            graphics::set_canvas(ctx, None);
            graphics::set_background_color(ctx, graphics::Color::from((64, 0, 0, 255)));
            graphics::clear(ctx);

            graphics::draw_ex(
                ctx,
                &self.text,
                graphics::DrawParam {
                    dest: dest_point,
                    color: Some(graphics::Color::from((192, 128, 64, 255))),
                    ..Default::default()
                },
            )?;
        }

        graphics::present(ctx)?;

        // Drawables are drawn from their top-left corner.
        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
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
    let c = conf::Conf {
        window_setup: conf::WindowSetup {
            samples: conf::NumSamples::Two,
            ..Default::default()
        },
        ..Default::default()
    };
    let (ctx, event_loop) = &mut Context::load_from_conf("helloworld", "ggez", c)?;

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
