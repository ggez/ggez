//! Demonstrates various projection and matrix fiddling/testing.
use ggez::event;
use ggez::glam::*;
use ggez::graphics::{self, Color, DrawMode, DrawParam};
use ggez::input::keyboard;
use ggez::{Context, GameResult};
use std::env;
use std::path;

const GRID_INTERVAL: f32 = 100.0;
const GRID_SIZE: usize = 10;
const GRID_POINT_RADIUS: f32 = 5.0;

struct MainState {
    pos_x: f32,
    gridmesh: graphics::Mesh,
    angle: graphics::Image,
    screen_bounds: Vec<graphics::Rect>,
    screen_bounds_idx: usize,
    screen_coords: graphics::Rect,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let angle = graphics::Image::from_path(ctx, "/angle.png")?;
        let gridmesh_builder = &mut graphics::MeshBuilder::new();
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                let fx = x as f32;
                let fy = y as f32;
                let fsize = GRID_SIZE as f32;
                let point = Vec2::new(fx * GRID_INTERVAL, fy * GRID_INTERVAL);
                let color = graphics::Color::new(fx / fsize, 0.0, fy / fsize, 1.0);
                gridmesh_builder.circle(DrawMode::fill(), point, GRID_POINT_RADIUS, 2.0, color)?;
            }
        }
        let gridmesh = graphics::Mesh::from_data(ctx, gridmesh_builder.build());
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
            screen_coords: graphics::Rect::new(
                0.,
                0.,
                ctx.gfx.drawable_size().0,
                ctx.gfx.drawable_size().1,
            ),
        };
        Ok(s)
    }
}

fn draw_coord_labels(canvas: &mut graphics::Canvas) {
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let point = Vec2::new(x as f32 * GRID_INTERVAL, y as f32 * GRID_INTERVAL);
            let s = format!("({}, {})", point.x, point.y);
            canvas.draw(&graphics::Text::new(s), point);
        }
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));
        canvas.set_screen_coordinates(self.screen_coords);

        let origin = Vec2::ZERO;
        canvas.draw(
            &self.gridmesh,
            DrawParam::new().dest(origin).color(Color::WHITE),
        );

        draw_coord_labels(&mut canvas);

        canvas.draw(
            &self.angle,
            graphics::DrawParam::new()
                .dest(Vec2::new(400.0, 400.0))
                .rotation(self.pos_x / 100.0)
                .offset(Vec2::new(0.5, 0.5))
                .scale(Vec2::new(1.0, 1.0)),
        );

        canvas.finish(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: keyboard::KeyInput,
        _repeat: bool,
    ) -> GameResult {
        if let Some(keyboard::KeyCode::Space) = input.keycode {
            self.screen_bounds_idx = (self.screen_bounds_idx + 1) % self.screen_bounds.len();
            self.screen_coords = self.screen_bounds[self.screen_bounds_idx];
        }
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
        .window_setup(
            ggez::conf::WindowSetup::default()
                .title("transforms -- Press spacebar to cycle projection!"),
        )
        .add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
