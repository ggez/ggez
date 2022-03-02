//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use ggez::graphics::{
    self,
    draw::DrawParam,
    image::Image,
    mesh::{Mesh, MeshBuilder},
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
    circle: Mesh,
    frame: ScreenImage,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let circle = Mesh::from_data(
            &ctx.gfx_context,
            MeshBuilder::new()
                .circle(
                    graphics::DrawMode::fill(),
                    vec2(0., 0.),
                    100.,
                    2.0,
                    Color::WHITE,
                )?
                .build(),
        );

        Ok(MainState {
            pos_x: 0.0,
            circle,
            frame: ScreenImage::new(&ctx.gfx_context, None, 1., 1., 1),
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

        let mut canvas = Canvas::from_image(
            &mut ctx.gfx_context,
            CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
            &frame,
            None,
        );

        canvas.draw_mesh(
            &self.circle,
            None,
            DrawParam::new().offset(vec2(self.pos_x, 300.)),
        );

        canvas.finish()?;

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
