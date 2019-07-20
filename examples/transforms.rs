//! Demonstrates various projection and matrix fiddling/testing.
use ggez;
use nalgebra;

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, DrawMode};
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::path;

struct MainState {
    pos_x: f32,
    gridmesh: graphics::Mesh,
    angle: graphics::Image,
    screen_bounds: Vec<graphics::Rect>,
    screen_bounds_idx: usize,
}

impl MainState {
    const GRID_INTERVAL: f32 = 100.0;
    const GRID_SIZE: usize = 10;
    const GRID_POINT_RADIUS: f32 = 5.0;

    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let angle = graphics::Image::new(ctx, "/angle.png")?;
        let gridmesh_builder = &mut graphics::MeshBuilder::new();
        for x in 0..Self::GRID_SIZE {
            for y in 0..Self::GRID_SIZE {
                let fx = x as f32;
                let fy = y as f32;
                let fsize = Self::GRID_SIZE as f32;
                let point = na::Point2::new(fx * Self::GRID_INTERVAL, fy * Self::GRID_INTERVAL);
                let color = graphics::Color::new(fx / fsize, 0.0, fy / fsize, 1.0);
                gridmesh_builder.circle(
                    DrawMode::fill(),
                    point,
                    Self::GRID_POINT_RADIUS,
                    2.0,
                    color,
                );
            }
        }
        let gridmesh = gridmesh_builder.build(ctx)?;
        // An array of rects to cycle the screen coordinates through.
        let screen_bounds = vec![
            graphics::Rect::new(0.0, 0.0, 800.0, 600.0),
            graphics::Rect::new(0.0, 600.0, 800.0, -600.0),
        ];
        let screen_bounds_idx = 0;
        let s = MainState {
            pos_x: 0.0,
            gridmesh,
            angle,
            screen_bounds,
            screen_bounds_idx,
        };
        Ok(s)
    }

    fn draw_coord_labels(&self, ctx: &mut Context) -> GameResult {
        for x in 0..Self::GRID_SIZE {
            for y in 0..Self::GRID_SIZE {
                let point = na::Point2::new(
                    x as f32 * Self::GRID_INTERVAL,
                    y as f32 * Self::GRID_INTERVAL,
                );
                let s = format!("({}, {})", point.x, point.y);
                let t = graphics::Text::new(s);
                graphics::queue_text(ctx, &t, point, None);
            }
        }
        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::default(),
            None,
            graphics::FilterMode::Linear,
        )
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let origin: na::Point2<f32> = na::Point2::origin();
        graphics::draw(ctx, &self.gridmesh, (origin, graphics::WHITE))?;

        let param = graphics::DrawParam::new()
            .dest(na::Point2::new(400.0, 400.0))
            .rotation(self.pos_x / 100.0)
            .offset(na::Point2::new(0.5, 0.5))
            .scale(na::Vector2::new(1.0, 1.0));

        self.draw_coord_labels(ctx)?;

        graphics::draw(ctx, &self.angle, param)?;
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            event::KeyCode::Space => {
                self.screen_bounds_idx = (self.screen_bounds_idx + 1) % self.screen_bounds.len();
                graphics::set_screen_coordinates(ctx, self.screen_bounds[self.screen_bounds_idx])
                    .unwrap();
            }
            _ => (),
        }
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
        .window_setup(
            ggez::conf::WindowSetup::default()
                .title("transforms -- Press spacebar to cycle projection!"),
        )
        .add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
