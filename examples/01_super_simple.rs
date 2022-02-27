//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use ggez::graphics::{self, draw::DrawParam, image::Image, Color, Rect};
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
        Ok(MainState {
            pos_x: 0.0,
            frame: ScreenImage::new(&ctx.gfx_context, ImageFormat::Rgba8Srgb, 1., 1., 1),
            depth: ScreenImage::new(&ctx.gfx_context, ImageFormat::Depth32, 1., 1., 1),
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

        canvas.finish();

        ctx.gfx_context.present(&frame)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    env_logger::init();
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
