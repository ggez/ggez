//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use ggez::graphics::{
    self,
    draw::DrawParam,
    image::Image,
    text::{FontData, Text, TextLayout},
    Color, Rect,
};
use ggez::{
    event, filesystem,
    graphics::{
        canvas::{Canvas, CanvasLoadOp},
        image::{ImageFormat, ScreenImage},
        sampler::Sampler,
    },
};
use ggez::{Context, GameResult};
use glam::*;

struct MainState {
    pos_x: f32,
    frame: ScreenImage,
    depth: ScreenImage,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        ctx.gfx_context.add_font(
            "monospace",
            FontData::from_slice(include_bytes!("../resources/LiberationMono-Regular.ttf"))?,
        );

        Ok(MainState {
            pos_x: 0.0,
            frame: ScreenImage::new(&ctx.gfx_context, None, 1., 1., 1),
            depth: ScreenImage::new(&ctx.gfx_context, ImageFormat::Depth32Float, 1., 1., 1),
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let frame = self.frame.image(&ctx.gfx_context);
        let depth = self.depth.image(&ctx.gfx_context);

        let mut canvas = Canvas::from_image(
            &mut ctx.gfx_context,
            CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
            &frame,
            Some(&depth),
        );

        canvas.draw(
            None,
            DrawParam::new()
                .color(Color::WHITE)
                .dst_rect(Rect::new(30., 10., 50., 70.))
                .rotation_deg(30.),
        );

        canvas.draw_text(
            &[Text::new()
                .text("Hello, world!")
                .font("monospace")
                .size(16.)
                .color(Color::RED)],
            glam::vec2(30., 70.),
            TextLayout::tl_single_line(),
        );

        canvas.draw(
            None,
            DrawParam::new()
                .color(Color::BLUE)
                .dst_rect(Rect::new(40., 40., 10., 100.))
                .rotation_deg(-10.),
        );

        canvas.finish()?;

        ctx.gfx_context.present(&frame)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    env_logger::init();

    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("super_simple", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
