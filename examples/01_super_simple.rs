//! The simplest possible example that does something.

use ggez;
use ggez::event;
use ggez::ggraphics as gg;
use ggez::graphics;
use ggez::{Context, GameResult};

struct MainState {
    pos_x: f32,
    passes: Vec<gg::RenderPass>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            pos_x: 0.0,
            passes: vec![],
        };

        use ggraphics::Pipeline;
        unsafe {
            // Render raw texture to the screen
            {
                let particle_image = graphics::Image::new(ctx, "/player.png")?;
                let shader = graphics::default_shader(ctx);
                let gl = graphics::gl_context(ctx);

                let mut screen_pass =
                    gg::RenderPass::new_screen(&*gl, 800, 600, Some((0.1, 0.2, 0.3, 1.0)));
                let mut pipeline = gg::QuadPipeline::new(gl.clone(), shader);
                let dc = pipeline.new_drawcall(particle_image.texture, gg::SamplerSpec::default());
                dc.add(gg::QuadData::empty());
                screen_pass.add_pipeline(pipeline);
                //s.passes.push(screen_pass);
            }

            let particle_image = graphics::Image::new(ctx, "/player.png")?;
            let shader = graphics::default_shader(ctx);
            let mut batch = graphics::DrawBatch::new(ctx, shader);
            batch.add_quad(&particle_image, gg::QuadData::empty());
            let gl = graphics::gl_context(ctx);
            let mut screen_pass =
                gg::RenderPass::new_screen(&*gl, 800, 600, Some((0.1, 0.2, 0.3, 1.0)));
            screen_pass.add_pipeline(batch.pipe);
            s.passes.push(screen_pass);

            /*
                         * I don't hate this:
            let particle_image = graphics::Image::new(ctx, "/player.png")?;
            let mut batch = graphics::DrawBatch::new(ctx, graphics::default_shader(ctx))
                .add_quad(&particle_image, gg::QuadData::empty());
            let screen_pass = graphics::screen_pass().draw(ctx, &[batch], Some((0.1, 0.2, 0.3, 1.0)));
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
        let gl = graphics::gl_context(ctx);
        gl.draw(&mut self.passes);
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    use std::env;
    use std::path;
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("super_simple", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
