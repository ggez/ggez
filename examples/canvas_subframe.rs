//! An example of how to use an `InstanceArray` with a `Canvas`.
//!
//! You really want to run this one in release mode.

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use std::env;
use std::f32::consts::TAU;
use std::path;

type Point2 = ggez::glam::Vec2;
type Vector2 = ggez::glam::Vec2;

struct MainState {
    instances: graphics::InstanceArray,
    canvas_image: graphics::ScreenImage,
    draw_pt: Point2,
    draw_vec: Vector2,
}

impl MainState {
    fn new(ctx: &mut Context) -> MainState {
        let image = graphics::Image::from_path(ctx, "/tile.png").unwrap();
        let mut instances = graphics::InstanceArray::new(ctx, image);
        instances.resize(ctx, 150 * 150);
        let canvas_image = graphics::ScreenImage::new(ctx, None, 1., 1., 1);
        let draw_pt = Point2::new(0.0, 0.0);
        let draw_vec = Vector2::new(1.0, 1.0);
        MainState {
            instances,
            canvas_image,
            draw_pt,
            draw_vec,
        }
    }
}

impl MainState {
    fn draw_spritebatch(&mut self, ctx: &mut Context) -> GameResult {
        // Freeze the animation so things are easier to see.
        let time = 2000;
        // let time = (ctx.timer.time_since_start().as_secs_f64() * 1000.0) as u32;
        let cycle = 10_000;
        self.instances.set((0..150).flat_map(|x| {
            (0..150).map(move |y| {
                let x = x as f32;
                let y = y as f32;
                graphics::DrawParam::new()
                    .dest(Point2::new(x * 10.0, y * 10.0))
                    // scale: graphics::Point::new(0.0625, 0.0625),
                    .scale(Vector2::new(
                        ((time % cycle * 2) as f32 / cycle as f32 * TAU).cos().abs() * 0.0625,
                        ((time % cycle * 2) as f32 / cycle as f32 * TAU).cos().abs() * 0.0625,
                    ))
                    .rotation(-2.0 * ((time % cycle) as f32 / cycle as f32 * TAU))
            })
        }));

        let mut canvas =
            graphics::Canvas::from_screen_image(ctx, &mut self.canvas_image, Color::WHITE);

        let param = graphics::DrawParam::new()
            .dest(Point2::new(
                ((time % cycle) as f32 / cycle as f32 * TAU).cos() * 50.0 + 150.0,
                ((time % cycle) as f32 / cycle as f32 * TAU).sin() * 50.0 + 250.0,
            ))
            .scale(Vector2::new(
                ((time % cycle) as f32 / cycle as f32 * TAU).sin().abs() * 2.0 + 1.0,
                ((time % cycle) as f32 / cycle as f32 * TAU).sin().abs() * 2.0 + 1.0,
            ))
            .rotation((time % cycle) as f32 / cycle as f32 * TAU)
            .offset(Point2::new(750., 750.));

        canvas.draw(&self.instances, param);
        canvas.finish(ctx)?;

        Ok(())
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.time.ticks() % 100 == 0 {
            println!("Delta frame time: {:?} ", ctx.time.delta());
            println!("Average FPS: {}", ctx.time.fps());
        }

        // Bounce the rect if necessary
        let (w, h) = ctx.gfx.drawable_size();
        if self.draw_pt.x + w / 2.0 > w || self.draw_pt.x < 0.0 {
            self.draw_vec.x *= -1.0;
        }
        if self.draw_pt.y + h / 2.0 > h || self.draw_pt.y < 0.0 {
            self.draw_vec.y *= -1.0;
        }
        self.draw_pt += self.draw_vec;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.draw_spritebatch(ctx)?;

        let canvas_image = self.canvas_image.image(ctx);
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));

        let src_x = self.draw_pt.x / canvas_image.width() as f32;
        let src_y = self.draw_pt.y / canvas_image.height() as f32;

        canvas.draw(
            &canvas_image,
            graphics::DrawParam::new()
                .dest(self.draw_pt)
                .src(graphics::Rect::new(src_x, src_y, 0.5, 0.5)),
        );

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    if cfg!(debug_assertions) && env::var("yes_i_really_want_debug_mode").is_err() {
        eprintln!(
            "Note: Release mode will improve performance greatly.\n    \
             e.g. use `cargo run --example canvas_subframe --release`"
        );
    }
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("canvas_subframe", "ggez").add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx);
    event::run(ctx, events_loop, state)
}
