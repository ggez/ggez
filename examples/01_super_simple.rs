//! The simplest possible example that does something.

use ggez;
use ggez::event;
use ggez::ggraphics;
use ggez::graphics;
use ggez::{Context, GameResult};
use image;

struct MainState {
    pos_x: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { pos_x: 0.0 };

        use ggraphics::Pipeline;
        let gl = graphics::gl_context_mut(ctx);
        unsafe {
            let particle_texture = {
                let image_bytes = include_bytes!("../resources/player.png");
                let image_rgba = image::load_from_memory(image_bytes).unwrap().to_rgba();
                let (w, h) = image_rgba.dimensions();
                let image_rgba_bytes = image_rgba.into_raw();
                ggraphics::TextureHandle::new(gl, &image_rgba_bytes, w as usize, h as usize)
                    .into_shared()
            };
            // Render that texture to the screen
            let shader = gl.default_shader();
            graphics::screen_render_pass(ctx, |mut pass| {
                pass.quad_pipeline(ctx, shader, |mut pipe| {
                    /*
                    let dc =
                        pipe.new_drawcall(gl, particle_texture, ggraphics::SamplerSpec::default());
                    dc.add(ggraphics::QuadData::empty());
                    */
                });
            });
            /*
            let mut screen_pass = ggraphics::RenderPass::new_screen(gl, 800, 600, (0.1, 0.2, 0.3, 1.0));
            let mut pipeline = ggraphics::QuadPipeline::new(&gl, shader);
            let dc = pipeline.new_drawcall(gl, particle_texture, ggraphics::SamplerSpec::default());
            dc.add(ggraphics::QuadData::empty());
            screen_pass.add_pipeline(pipeline);
            gl.passes.push(screen_pass);
            */
        }
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        /*
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (na::Point2::new(self.pos_x, 380.0),))?;

        */
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
