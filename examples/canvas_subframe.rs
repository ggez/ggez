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
use ggez::nalgebra as na;
use std::env;
use std::path;

struct MainState {
    spritebatch: graphics::spritebatch::SpriteBatch,
    canvas: graphics::Canvas,
    draw_pt: graphics::Point2,
    draw_vec: na::Vector2<f32>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image = graphics::Image::new(ctx, "/tile.png").unwrap();
        let spritebatch = graphics::spritebatch::SpriteBatch::new(image);
        let canvas = graphics::Canvas::with_window_size(ctx)?;
        let draw_pt = na::origin();
        let draw_vec = na::Vector2::new(1.0, 1.0);
        let s = MainState { 
            spritebatch, 
            canvas ,
            draw_pt,
            draw_vec,
        };
        Ok(s)
    }
}

impl MainState {
    fn draw_spritebatch(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_canvas(ctx, Some(&self.canvas));
        graphics::set_background_color(ctx, graphics::WHITE);
        graphics::clear(ctx);

        // Freeze the animation so things are easier to see.
        // let time = (timer::duration_to_f64(timer::get_time_since_start(ctx)) * 1000.0) as u32;
        let time = 2000;
        let cycle = 10_000;
        for x in 0..150 {
            for y in 0..150 {
                let x = x as f32;
                let y = y as f32;
                let p = graphics::DrawParam {
                    dest: graphics::Point2::new(x * 10.0, y * 10.0),
                    // scale: graphics::Point::new(0.0625, 0.0625),
                    scale: graphics::Point2::new(
                        ((time % cycle * 2) as f32 / cycle as f32 * 6.28)
                            .cos()
                            .abs() * 0.0625,
                        ((time % cycle * 2) as f32 / cycle as f32 * 6.28)
                            .cos()
                            .abs() * 0.0625,
                    ),
                    rotation: -2.0 * ((time % cycle) as f32 / cycle as f32 * 6.28),
                    ..Default::default()
                };
                self.spritebatch.add(p);
            }
        }
        let param = graphics::DrawParam {
            dest: graphics::Point2::new(
                ((time % cycle) as f32 / cycle as f32 * 6.28).cos() * 50.0 - 350.0,
                ((time % cycle) as f32 / cycle as f32 * 6.28).sin() * 50.0 - 450.0,
            ),
            scale: graphics::Point2::new(
                ((time % cycle) as f32 / cycle as f32 * 6.28).sin().abs() * 2.0 + 1.0,
                ((time % cycle) as f32 / cycle as f32 * 6.28).sin().abs() * 2.0 + 1.0,
            ),
            rotation: ((time % cycle) as f32 / cycle as f32 * 6.28),
            offset: graphics::Point2::new(750.0, 750.0),
            ..Default::default()
        };
        graphics::draw_ex(ctx, &self.spritebatch, param)?;
        self.spritebatch.clear();
        graphics::set_canvas(ctx, None);
        Ok(())
    }

}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if timer::get_ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::get_delta(ctx));
            println!("Average FPS: {}", timer::get_fps(ctx));
        }

        // Bounce the rect if necessary
        let (w, h) = graphics::get_size(ctx); 
        if self.draw_pt.x + (w as f32 / 2.0) > (w as f32) || self.draw_pt.x < 0.0 {
            self.draw_vec.x *= -1.0;
        }
        // println!("{:?}", self.draw_pt);
        // BUGGO: The height bounds are hella wrong!
        if self.draw_pt.y + (h as f32 / 2.0) > (h as f32 / 2.0) || self.draw_pt.y < -(h as f32 / 2.0) {
            self.draw_vec.y *= -1.0;
        }
        self.draw_pt += self.draw_vec;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_background_color(ctx, [0.1, 0.2, 0.3, 1.0].into());
        graphics::clear(ctx);
        self.draw_spritebatch(ctx)?;
        let dims = self.canvas.get_image().get_dimensions();
        let src_x = self.draw_pt.x / dims.w;
        let src_y = self.draw_pt.y / dims.h;
        graphics::draw_ex(ctx, &self.canvas, 
            graphics::DrawParam {
                dest: self.draw_pt,
                src: graphics::Rect::new(src_x, -src_y, 0.5, 0.5),
                .. Default::default()
                })?;
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
