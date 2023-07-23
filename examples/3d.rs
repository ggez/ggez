use ggez::coroutine::Loading;
use ggez::graphics::{Camera3d, Canvas3d, DrawParam3d, Mesh3d, Mesh3dBuilder, Vertex3d};
use std::{env, path};

use ggez::graphics::Shader;
use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};

struct MainState {
    camera: Camera3d,
    meshes: Vec<(Mesh3d, Vec3, Vec3)>,
    custom_shader: Loading<Shader>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut camera = Camera3d::default();
        let vertex_data = vec![
            // front (0.0, 0.0, 1.0)
            Vertex3d::new([-1.0, -1.0, 1.0], [0.0, 0.0], Color::GREEN, [0.0, 1.0, 0.0]),
            Vertex3d::new([1.0, -1.0, 1.0], [1.0, 0.0], Color::GREEN, [0.0, 1.0, 0.0]),
            Vertex3d::new([1.0, 1.0, 1.0], [1.0, 1.0], Color::GREEN, [0.0, 1.0, 0.0]),
            Vertex3d::new(
                [-1.0, 1.0, 1.0],
                [0.0, 1.0],
                Color::new(0.0, 1.0, 0.0, 1.0),
                [0.0, 0.0, 1.0],
            ),
            // back (0.0, 0.0, -1.0)
            Vertex3d::new([-1.0, 1.0, -1.0], [1.0, 0.0], None, [0.0, 0.0, -1.0]),
            Vertex3d::new([1.0, 1.0, -1.0], [0.0, 0.0], None, [0.0, 0.0, -1.0]),
            Vertex3d::new([1.0, -1.0, -1.0], [0.0, 1.0], None, [0.0, 0.0, -1.0]),
            Vertex3d::new([-1.0, -1.0, -1.0], [1.0, 1.0], None, [0.0, 0.0, -1.0]),
            // right (1.0, 0.0, 0.0)
            Vertex3d::new([1.0, -1.0, -1.0], [0.0, 0.0], None, [1.0, 0.0, 0.0]),
            Vertex3d::new([1.0, 1.0, -1.0], [1.0, 0.0], None, [1.0, 0.0, 0.0]),
            Vertex3d::new([1.0, 1.0, 1.0], [1.0, 1.0], None, [1.0, 0.0, 0.0]),
            Vertex3d::new([1.0, -1.0, 1.0], [0.0, 1.0], None, [1.0, 0.0, 0.0]),
            // left (-1.0, 0.0, 0.0)
            Vertex3d::new([-1.0, -1.0, 1.0], [1.0, 0.0], None, [-1.0, 0.0, 0.0]),
            Vertex3d::new([-1.0, 1.0, 1.0], [0.0, 0.0], None, [-1.0, 0.0, 0.0]),
            Vertex3d::new([-1.0, 1.0, -1.0], [0.0, 1.0], None, [-1.0, 0.0, 0.0]),
            Vertex3d::new([-1.0, -1.0, -1.0], [1.0, 1.0], None, [-1.0, 0.0, 0.0]),
            // top (0.0, 1.0, 0.0)
            Vertex3d::new([1.0, 1.0, -1.0], [1.0, 0.0], None, [0.0, 1.0, 0.0]),
            Vertex3d::new([-1.0, 1.0, -1.0], [0.0, 0.0], None, [0.0, 1.0, 0.0]),
            Vertex3d::new([-1.0, 1.0, 1.0], [0.0, 1.0], None, [0.0, 1.0, 0.0]),
            Vertex3d::new([1.0, 1.0, 1.0], [1.0, 1.0], None, [0.0, 1.0, 0.0]),
            // bottom (0.0, -1.0, 0.0)
            Vertex3d::new([1.0, -1.0, 1.0], [0.0, 0.0], None, [0.0, -1.0, 0.0]),
            Vertex3d::new([-1.0, -1.0, 1.0], [1.0, 0.0], None, [0.0, -1.0, 0.0]),
            Vertex3d::new([-1.0, -1.0, -1.0], [1.0, 1.0], None, [0.0, -1.0, 0.0]),
            Vertex3d::new([1.0, -1.0, -1.0], [0.0, 1.0], None, [0.0, -1.0, 0.0]),
        ];
        #[rustfmt::skip]
        let index_data: Vec<u32> = vec![
             0,  1,  2,  2,  3,  0, // top
             4,  5,  6,  6,  7,  4, // bottom
             8,  9, 10, 10, 11,  8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        let image_two =
            graphics::Image::from_color(ctx, 1, 1, Some(graphics::Color::from_rgb(50, 50, 50)));
        let mesh = Mesh3dBuilder::new()
            .from_data(vertex_data, index_data, None)
            .build(ctx);
        let mesh_two = Mesh3dBuilder::new()
            .cube(Vec3::splat(5.0))
            .texture(image_two)
            .build(ctx);

        camera.transform.yaw = 90.0;
        Ok(MainState {
            camera,
            meshes: vec![
                (mesh, Vec3::new(10.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (mesh_two, Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
            ],
            custom_shader: graphics::ShaderBuilder::from_path("/fancy.wgsl").build_async(),
        })
    }
}

impl event::EventHandler for MainState {
    fn resize_event(&mut self, _: &mut Context, width: f32, height: f32) -> GameResult {
        self.camera.projection.resize(width as u32, height as u32);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.custom_shader.poll(ctx)?;
        let k_ctx = &ctx.keyboard.clone();
        let (yaw_sin, yaw_cos) = self.camera.transform.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        if k_ctx.is_key_pressed(KeyCode::Q) {
            self.meshes[1].1 += 0.1;
        }
        if k_ctx.is_key_pressed(KeyCode::E) {
            self.meshes[1].1 -= 0.1;
        }
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
        let mut canvas3d = Canvas3d::from_frame(ctx, Color::BLACK);
        canvas3d.set_projection(self.camera.to_matrix());
        for (i, mesh) in self.meshes.iter().enumerate() {
            if i == 0 {
                canvas3d.set_default_shader();
            } else if let Some(shader) = self.custom_shader.result() {
                canvas3d.set_shader(shader);
            }

            canvas3d.draw(
                &mesh.0,
                DrawParam3d::default()
                    .scale(mesh.1)
                    .color(Color::new(0.5, 0.5, 0.5, 1.0)),
            );
        }
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
                Q/E: Scale cube up and Down
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

    let cb = ggez::ContextBuilder::new("3d", "ggez")
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
