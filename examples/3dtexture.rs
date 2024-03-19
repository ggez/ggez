use std::{env, path};

use ggez::graphics::{Camera3d, Canvas3d, DrawParam3d, Image, Mesh3d, Mesh3dBuilder};

use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};

struct MainState {
    camera: Camera3d,
    static_camera: Camera3d,
    cube_one: (Mesh3d, Quat),
    cube_two: (Mesh3d, Quat),
    canvas_image: Image,
    fancy_shader: graphics::Shader,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut camera = Camera3d::default();
        let mut static_camera = Camera3d::default();

        let mesh = Mesh3dBuilder::new().cube(Vec3::splat(10.0)).build(ctx);
        let canvas_image = Image::new_canvas_image(ctx, 320, 240, 1);
        let mesh_two = Mesh3dBuilder::new()
            .cube(Vec3::splat(10.0))
            .texture(canvas_image.clone())
            .build(ctx);

        camera.transform.yaw = 90.0;
        static_camera.transform.yaw = 90.0;
        Ok(MainState {
            static_camera,
            canvas_image,
            camera,
            cube_one: (mesh, Quat::IDENTITY),
            cube_two: (mesh_two, Quat::IDENTITY),
            fancy_shader: graphics::ShaderBuilder::from_path("/fancy.wgsl")
                .build(&ctx.gfx)
                .unwrap(),
        })
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
        let dt = ctx.time.delta().as_secs_f32();
        self.cube_one.1 *= Quat::from_rotation_x(50.0_f32.to_radians() * dt);
        self.cube_one.1 *= Quat::from_rotation_y(50.0_f32.to_radians() * dt);
        self.cube_two.1 *= Quat::from_rotation_y(-50.0_f32.to_radians() * dt);

        if k_ctx.is_key_pressed(KeyCode::Space) {
            self.camera.transform.position.y += 1.0;
        }
        if k_ctx.is_key_pressed(KeyCode::C) {
            self.camera.transform.position.y -= 1.0;
        }
        if k_ctx.is_key_pressed(KeyCode::W) {
            self.camera.transform = self.camera.transform.translate(forward);
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.camera.transform = self.camera.transform.translate(-forward);
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.camera.transform = self.camera.transform.translate(right);
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.camera.transform = self.camera.transform.translate(-right);
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
        let mut canvas3d = Canvas3d::from_image(ctx, self.canvas_image.clone(), Color::RED);
        canvas3d.set_projection(self.static_camera.to_matrix());
        canvas3d.set_shader(&self.fancy_shader);
        canvas3d.draw(
            &self.cube_one.0,
            DrawParam3d::default()
                .position(Vec3::new(-10.0, 0.0, 20.0))
                .rotation(self.cube_one.1),
        );
        canvas3d.finish(ctx)?;
        let mut canvas3d = Canvas3d::from_frame(ctx, Color::BLACK);
        canvas3d.set_projection(self.camera.to_matrix());
        canvas3d.set_shader(&self.fancy_shader);
        canvas3d.draw(
            &self.cube_two.0,
            DrawParam3d::default().rotation(self.cube_two.1),
        );
        canvas3d.finish(ctx)?;
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        let dest_point2 = Vec2::new(10.0, 250.0);
        canvas.draw(
            &graphics::Text::new(
                "
                WASD: Move
                Arrow Keys: Look
                K: Toggle default shader and custom shader
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

    let cb = ggez::ContextBuilder::new("3dtexture", "ggez")
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
