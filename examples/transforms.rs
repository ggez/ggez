//! Demonstrates various projection and matrix fiddling/testing.
//! 
extern crate ggez;
extern crate nalgebra;

use ggez::event;
use ggez::graphics::{self, DrawMode};
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::path;

struct MainState {
    pos_x: f32,
    gridmesh: graphics::Mesh,
    angle: graphics::Image,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let angle = graphics::Image::new(ctx, "/angle.png")?;
        let grid_interval = 100.0;
        let grid_size = 10;
        let grid_point_radius = 5.0;
        let gridmesh_builder = &mut graphics::MeshBuilder::new();
        for x in 0..grid_size {
            for y in 0..grid_size {
                let point = na::Point2::new(x as f32 * grid_interval, y as f32 * grid_interval);
                gridmesh_builder
                    .circle(DrawMode::Fill, point, grid_point_radius, 2.0);
            }
        }
        let gridmesh = gridmesh_builder.build(ctx)?;
        let s = MainState { pos_x: 0.0, gridmesh, angle };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let origin: na::Point2<f32> = na::origin();
        graphics::draw(ctx, &self.gridmesh, (origin, graphics::BLACK))?;

        let param = graphics::DrawParam::new()
            .dest(na::Point2::new(300.0, 300.0))
            .rotation(self.pos_x / 100.0)
            .offset(na::Point2::new(64.0, 64.0))
        ;
        graphics::draw(
            ctx,
            &self.angle,
            param
        )?;
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };
    let cb = ggez::ContextBuilder::new("transforms", "ggez")
        .add_resource_path(resource_dir)
        // .window_setup(ggez::conf::WindowSetup::default().srgb(false))
    ;
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
