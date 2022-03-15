//! A collection of semi-random shape and image drawing examples.

use ggez::{
    event,
    graphics::{self, Color},
    Context, GameResult,
};
use glam::*;
use std::{env, path};

struct MainState {
    image1: graphics::Image,
    image2: graphics::Image,
    meshes: Vec<(Option<graphics::Image>, graphics::Mesh)>,
    rect: graphics::Mesh,
    rotation: f32,
}

impl MainState {
    /// Load images and create meshes.
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image1 = graphics::Image::from_path(ctx, "/dragon1.png", true)?;
        let image2 = graphics::Image::from_path(ctx, "/shot.png", true)?;

        let mb = &mut graphics::MeshBuilder::new();
        mb.rectangle(
            graphics::DrawMode::stroke(1.0),
            graphics::Rect::new(450.0, 450.0, 50.0, 50.0),
            graphics::Color::new(1.0, 0.0, 0.0, 1.0),
        )?;

        let rock = graphics::Image::from_path(ctx, "/rock.png", true)?;

        let meshes = vec![
            (None, build_mesh(ctx)?),
            (Some(rock), build_textured_triangle(ctx)?),
        ];

        let rect = graphics::Mesh::from_raw(&ctx.gfx, mb.build());

        let s = MainState {
            image1,
            image2,
            meshes,
            rect,
            rotation: 1.0,
        };

        Ok(s)
    }
}

fn build_mesh(ctx: &mut Context) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();

    mb.line(
        &[
            Vec2::new(200.0, 200.0),
            Vec2::new(400.0, 200.0),
            Vec2::new(400.0, 400.0),
            Vec2::new(200.0, 400.0),
            Vec2::new(200.0, 300.0),
        ],
        4.0,
        Color::new(1.0, 0.0, 0.0, 1.0),
    )?;

    mb.ellipse(
        graphics::DrawMode::fill(),
        Vec2::new(600.0, 200.0),
        50.0,
        120.0,
        1.0,
        Color::new(1.0, 1.0, 0.0, 1.0),
    )?;

    mb.circle(
        graphics::DrawMode::fill(),
        Vec2::new(600.0, 380.0),
        40.0,
        1.0,
        Color::new(1.0, 0.0, 1.0, 1.0),
    )?;

    Ok(graphics::Mesh::from_raw(&ctx.gfx, mb.build()))
}

fn build_textured_triangle(ctx: &mut Context) -> GameResult<graphics::Mesh> {
    let triangle_verts = vec![
        graphics::Vertex {
            position: [100.0, 100.0],
            uv: [1.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        graphics::Vertex {
            position: [0.0, 100.0],
            uv: [0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        graphics::Vertex {
            position: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let triangle_indices = vec![0, 1, 2];

    Ok(graphics::Mesh::from_raw(
        &ctx.gfx,
        graphics::MeshData {
            vertices: &triangle_verts,
            indices: &triangle_indices,
        },
    ))
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while ctx.timer.check_update_time(DESIRED_FPS) {
            self.rotation += 0.01;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            &mut ctx.gfx,
            graphics::CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
        )?;

        // Draw an image.
        let dst = glam::Vec2::new(20.0, 20.0);
        canvas.draw(&self.image1, graphics::DrawParam::new().offset(dst));

        // Draw an image with some options, and different filter modes.
        let dst = glam::Vec2::new(200.0, 100.0);
        let dst2 = glam::Vec2::new(400.0, 400.0);
        let scale = glam::Vec2::new(10.0, 10.0);

        canvas.draw(
            &self.image2,
            graphics::DrawParam::new()
                .offset(dst)
                .rotation(self.rotation)
                .image_scale(true)
                .scale(scale),
        );
        canvas.set_sampler(graphics::Sampler::nearest_clamp());
        canvas.draw(
            &self.image2,
            graphics::DrawParam::new()
                .offset(dst2)
                .rotation(self.rotation)
                .scale(scale)
                .origin(vec2(0.5, 0.5)),
        );
        canvas.set_default_sampler();

        // Draw a filled rectangle mesh.
        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        canvas.draw(
            None,
            graphics::DrawParam::new()
                .dst_rect(rect)
                .color(Color::WHITE),
        );

        // Draw a stroked rectangle mesh.
        canvas.draw_mesh(&self.rect, None, graphics::DrawParam::default());
        canvas.draw(None, graphics::DrawParam::new());

        // Draw some pre-made meshes
        for (image, mesh) in &self.meshes {
            canvas.draw_mesh(mesh, image, graphics::DrawParam::new().image_scale(false));
        }

        // Finished drawing, show it all on the screen!
        canvas.finish();

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

    let cb = ggez::ContextBuilder::new("drawing", "ggez").add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;

    let state = MainState::new(&mut ctx).unwrap();
    event::run(ctx, events_loop, state)
}
