//! The simplest possible example that does something.

use ggez;
use ggez::event;
use ggez::ggraphics as gg;
use ggez::graphics;
use ggez::{Context, GameResult};

struct MainState {
    pos_x: f32,
    batches: Vec<graphics::QuadBatch>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let particle_image = graphics::Image::new(ctx, "/player.png")?;
        let projection = graphics::default_projection(ctx);
        let mut batch = graphics::QuadBatch::new(ctx, projection);
        batch.add_quad(
            &particle_image,
            gg::QuadData {
                dst_rect: [0.0, 0.0, 32.0, 32.0],
                ..gg::QuadData::empty()
            },
        );
        batch.add_quad(
            &particle_image,
            gg::QuadData {
                color: [1.0, 0.0, 0.0, 1.0],
                dst_rect: [4.0, 0.0, 32.0, 32.0],
                ..gg::QuadData::empty()
            },
        );
        batch.add_quad(
            &particle_image,
            gg::QuadData {
                color: [0.0, 1.0, 0.0, 1.0],
                dst_rect: [0.0, 4.0, 32.0, 32.0],
                ..gg::QuadData::empty()
            },
        );

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
        graphics::screen_pass(ctx).clear_draw(
            graphics::Color::new(0.1, 0.2, 0.3, 1.0),
            self.batches.as_mut_slice(),
        );
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
