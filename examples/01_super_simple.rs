//! The simplest possible example that does something.

use ggez;
use ggez::event;
use ggez::ggraphics as gg;
use ggez::graphics;
use ggez::{Context, GameResult};

struct MainState {
    pos_x: f32,
    batches: Vec<graphics::DrawBatch>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        use ggraphics::Pipeline;
        let particle_image = graphics::Image::new(ctx, "/player.png")?;
        let shader = graphics::default_shader(ctx);
        let mut batch = graphics::DrawBatch::new(ctx, shader);
        batch.add_quad(&particle_image, gg::QuadData::empty());

        let s = MainState {
            pos_x: 0.0,
            batches: vec![batch],
        };
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
        graphics::screen_pass(ctx).draw(self.batches.as_mut_slice());
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
