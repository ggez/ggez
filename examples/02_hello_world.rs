//! Basic hello world example.

use ggez::graphics::image::ImageFormat;
use ggez::{
    event,
    graphics::{
        self,
        canvas::{Canvas, CanvasLoadOp},
        image::ScreenImage,
        text::{FontData, Text, TextLayout},
        Color,
    },
    Context, GameResult,
};
use std::{env, path};

// First we make a structure to contain the game's state
struct MainState {
    frame: ScreenImage,
    depth: ScreenImage,
    frames: usize,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let frame = ScreenImage::new(&ctx.gfx, None, 1., 1., 1);
        let depth = ScreenImage::new(&ctx.gfx, ImageFormat::Depth32Float, 1., 1., 1);

        ctx.gfx.add_font(
            "LiberationMono",
            FontData::from_path(&ctx.filesystem, "/LiberationMono-Regular.ttf")?,
        );

        let s = MainState {
            frame,
            depth,
            frames: 0,
        };
        Ok(s)
    }
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let frame = self.frame.image(&ctx.gfx);
        let depth = self.depth.image(&ctx.gfx);

        let mut canvas = Canvas::from_image(
            &mut ctx.gfx,
            CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
            &frame,
            Some(&depth),
        );

        // Drawables are drawn from their top-left corner.
        let offset = self.frames as f32 / 10.0;
        let dest_point = glam::Vec2::new(offset, offset);
        canvas.draw_text(
            &[Text::new()
                .text("Hello, world!")
                .font("LiberationMono")
                .size(48.)],
            dest_point,
            TextLayout::tl_single_line(),
        );

        canvas.finish();
        ctx.gfx.present(&frame);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ctx.timer.fps());
        }

        Ok(())
    }
}

// Now our main function, which does three things:
//
// * First, create a new `ggez::ContextBuilder`
// object which contains configuration info on things such
// as screen resolution and window title.
// * Second, create a `ggez::game::Game` object which will
// do the work of creating our MainState and running our game.
// * Then, just call `game.run()` which runs the `Game` mainloop.
pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("helloworld", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
