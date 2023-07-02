use ggez::graphics::{Camera3d, Canvas3d, InstanceArray3d, Mesh3dBuilder};
use std::f32::consts::TAU;
use std::{env, path};

use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};

struct MainState {
    camera: Camera3d,
    instances: InstanceArray3d,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut camera = Camera3d::default();
        camera.transform.yaw = 90.0;
        let cube = Mesh3dBuilder::new().cube(Vec3::splat(10.0)).build(ctx);

        let mut instances = graphics::InstanceArray3d::new(ctx, None, cube);
        instances.resize(ctx, 150 * 150);

        Ok(MainState { camera, instances })
    }
}

impl event::EventHandler for MainState {
    fn resize_event(&mut self, _: &mut Context, width: f32, height: f32) -> GameResult {
        self.camera.projection.resize(width as u32, height as u32);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let k_ctx = &ctx.keyboard.clone();
        let (yaw_sin, yaw_cos) = self.camera.transform.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        if k_ctx.is_key_pressed(KeyCode::Space) {
            self.camera.transform.position.y += 1.0;
        }
        if k_ctx.is_key_pressed(KeyCode::C) {
            self.camera.transform.position.y -= 1.0;
        }
        if k_ctx.is_key_pressed(KeyCode::W) {
            self.camera.transform.translate(forward);
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.camera.transform.translate(-forward);
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.camera.transform.translate(right);
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.camera.transform.translate(-right);
        }
        if k_ctx.is_key_pressed(KeyCode::Right) {
            self.camera.transform.yaw += 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Left) {
            self.camera.transform.yaw -= 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Up) {
            self.camera.transform.pitch += 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Down) {
            self.camera.transform.pitch -= 1.0_f32.to_radians();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas3d = Canvas3d::from_frame(ctx, Color::BLACK);
        canvas3d.set_projection(self.camera.calc_matrix());
        let time = (ctx.time.time_since_start().as_secs_f64() * 1000.0) as u32;
        let cycle = 10_000;
        // These are settings that apply per instance. These can be different per one
        self.instances.set((0..150).flat_map(|x| {
            (0..150).map(move |y| {
                let x = x as f32;
                let y = y as f32;
                graphics::DrawParam3d::default()
                    .position(Vec3::new(x * 10.0, y * 10.0, 0.0))
                    .scale(Vec3::new(
                        ((time % cycle * 2) as f32 / cycle as f32 * TAU).cos().abs() * 0.0625,
                        ((time % cycle * 2) as f32 / cycle as f32 * TAU).cos().abs() * 0.0625,
                        1.0,
                    ))
            })
        }));

        // These effect all instances. This is useful to for example scale up every instance by a given amount or offset them
        let param = graphics::DrawParam3d::default()
            .position(Vec3::splat(10.0))
            .scale(Vec3::splat(10.0));
        canvas3d.draw(&self.instances, param);

        canvas3d.finish(ctx)?;
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        // Do ggez drawing
        let dest_point1 = Vec2::new(10.0, 210.0);
        let dest_point2 = Vec2::new(10.0, 250.0);
        canvas.draw(
            &graphics::Text::new("You can mix 3d and 2d drawing;"),
            dest_point1,
        );
        canvas.draw(
            &graphics::Text::new(
                "
                WASD: Move
                Arrow Keys: Look
                C/Space: Up and Down
                ",
            ),
            dest_point2,
        );

        canvas.finish(ctx)?;

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

    let cb = ggez::ContextBuilder::new("3dshapes", "ggez")
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
