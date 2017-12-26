//! An example of how to use a `SpriteBatch`.
//! 
//! You really want to run this one in release mode.

extern crate ggez;
extern crate rand;

use ggez::conf;
use ggez::event;
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::timer;
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

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {

        if timer::get_ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::get_delta(ctx));
            println!("Average FPS: {}", timer::get_fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let time = (timer::duration_to_f64(timer::get_time_since_start(ctx)) * 1000.0) as u32;
        let cycle = 10_000;
        for x in 0..150 {
            for y in 0..150 {
                let x = x as f32;
                let y = y as f32;
                let p = graphics::DrawParam {
                    dest: graphics::Point2::new(x * 10.0, y * 10.0),
                    // scale: graphics::Point::new(0.0625, 0.0625),
                    scale: graphics::Point2::new(((time % cycle * 2) as f32 / cycle as f32 * 6.28)
                                                     .cos()
                                                     .abs() *
                                                 0.0625,
                                                 ((time % cycle * 2) as f32 / cycle as f32 * 6.28)
                                                     .cos()
                                                     .abs() *
                                                 0.0625),
                    rotation: -2.0 * ((time % cycle) as f32 / cycle as f32 * 6.28),
                    ..Default::default()
                };
                self.spritebatch.add(p);
            }
        }
        let param = graphics::DrawParam {
            dest: graphics::Point2::new(((time % cycle) as f32 / cycle as f32 * 6.28).cos() *
                                        50.0 - 350.0,
                                        ((time % cycle) as f32 / cycle as f32 * 6.28).sin() *
                                        50.0 - 450.0),
            scale: graphics::Point2::new(((time % cycle) as f32 / cycle as f32 * 6.28).sin().abs() *
                                         2.0 + 1.0,
                                         ((time % cycle) as f32 / cycle as f32 * 6.28).sin().abs() *
                                         2.0 + 1.0),
            rotation: ((time % cycle) as f32 / cycle as f32 * 6.28),
            offset: graphics::Point2::new(750.0, 750.0),
            ..Default::default()
        };
        graphics::draw_ex(ctx, &self.spritebatch, param)?;
        self.spritebatch.clear();

        graphics::present(ctx);
        Ok(())
    }
}

// Creating a gamestate depends on having an SDL context to load resources.
// Creating a context depends on loading a config file.
// Loading a config file depends on having FS (or we can just fake our way around it
// by creating an FS and then throwing it away; the costs are not huge.)
pub fn main() {
    let c = conf::Conf::new();
    println!("Starting with default config: {:#?}", c);
    let ctx = &mut Context::load_from_conf("spritebatch", "ggez", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so
    // we we look in the cargo project for files.
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
