//! The simplest possible example that does something.

use ggez::graphics::{self, Color, Rect};
use ggez::physics::{dynamics::RigidBodyBuilder, Physics};
use ggez::{event, physics::BodyHandle};
use ggez::{Context, GameResult};
use graphics::DrawParam;
use rapier2d::{dynamics::BodyStatus, geometry::ColliderBuilder};

struct MainState {
    physics: Physics,

    square: BodyHandle,
}

impl MainState {
    fn new() -> GameResult<Self> {
        let mut physics = Physics::new(None);

        let square_body = RigidBodyBuilder::new(BodyStatus::Dynamic)
            .translation(0.0, 0.0)
            .mass(10.0)
            .build();

        let square_collider = ColliderBuilder::cuboid(50.0 / 2.0 - 0.01, 50.0 / 2.0 - 0.01).build();

        let square = physics.insert_body(square_body, square_collider);

        let state = Self { physics, square };

        Ok(state)
    }
}

impl event::EventHandler for MainState {
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let square_body = self.physics.get_body(self.square).unwrap();
        let square_pos = square_body.position().translation;

        let square_rect = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(square_pos.x, square_pos.y, 50.0, 50.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
        )?;

        graphics::draw(ctx, &square_rect, DrawParam::default())?;

        graphics::present(ctx)
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.physics.step();

        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("some_wouw_physics_xD", "ggez");
    let (ctx, event_loop) = cb.build()?;
    let state = MainState::new()?;

    event::run(ctx, event_loop, state)
}
