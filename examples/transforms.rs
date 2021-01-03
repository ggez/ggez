//! Demonstrates various projection and matrix fiddling/testing.
use ggez;

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, DrawMode};
use ggez::timer;
use ggez::{Context, GameResult};
use glam::*;
use std::env;
use std::ops::Rem;
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
                let point = Vec2::new(fx * Self::GRID_INTERVAL, fy * Self::GRID_INTERVAL);
                let color = graphics::Color::new(fx / fsize, 0.0, fy / fsize, 1.0);
                gridmesh_builder.circle(
                    DrawMode::fill(),
                    point,
                    Self::GRID_POINT_RADIUS,
                    2.0,
                    color,
                )?;
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
                let point = Vec2::new(
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

        let origin = Vec2::zero();
        graphics::draw(ctx, &self.gridmesh, (origin, graphics::WHITE))?;

        let param = graphics::DrawParam::new()
            .dest(Vec2::new(400.0, 400.0))
            .rotation(self.pos_x / 100.0)
            .offset(Vec2::new(0.5, 0.5))
            .scale(Vec2::new(1.0, 1.0));

        self.draw_coord_labels(ctx)?;

        graphics::draw(ctx, &self.angle, param)?;

        /*
         * FIXME!
        let time = timer::time_since_start(ctx).as_secs_f64();

        let camera_zoom = time.sin().abs() as f32;
        let camera_transform = graphics::DrawParam::default()
            .scale(Vec2::new(camera_zoom, camera_zoom))
            .to_matrix();
        graphics::push_transform(ctx, Some(camera_transform));
        graphics::apply_transformations(ctx)?;

        let text = graphics::Text::new(String::from("Hello"));
        let text_rect = text.dimensions(ctx);
        let text_center = Vec2::new(text_rect.w, text_rect.h) / 2.0;
        let text_position = Vec2::new(100.0, 100.0);
        let border_rect = text_rect;
        let border = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(1.0),
            border_rect,
            graphics::WHITE,
        )?;
        let draw_params = graphics::DrawParam::default()
            .offset(text_center.clone())
            .dest(Vec2::from(text_position - text_center))
            .rotation(time.rem(std::f64::consts::TAU) as f32);
        graphics::draw(ctx, &text, draw_params)?;
        graphics::draw(ctx, &border, draw_params)?;
        */

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
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
