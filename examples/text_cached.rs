//! This example demonstrates how to use `TextCached` to draw TrueType font texts efficiently.
//! Powered by `gfx_glyph` crate.

extern crate ggez;

use ggez::conf;
use ggez::event;
use ggez::{Context, GameResult};
use ggez::graphics::{self, Point2};
use std::env;
use std::path;

struct MainState {
    text: graphics::TextCached,
    text_too: graphics::TextCached,
    frames: usize,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let font = graphics::Font::new_glyph_font(ctx, "/DejaVuSerif.ttf", 40)?;
        let font_too = graphics::Font::new_glyph_font(ctx, "/DejaVuSerif.ttf", 50)?;

        let text = graphics::TextCached::new(ctx, "Hello", &font)?;
        let text_too = graphics::TextCached::new(ctx, "World!", &font_too)?;

        let s = MainState {
            text,
            text_too,
            frames: 0,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        graphics::draw(ctx, &self.text, Point2::new(10.0, 20.0), 0.0)?;
        graphics::draw(ctx, &self.text_too, Point2::new(150.0, 20.0), 0.0)?;
        graphics::present(ctx);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        Ok(())
    }
}

pub fn main() {
    let conf = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("text_cached", "ggez", conf).unwrap();

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
