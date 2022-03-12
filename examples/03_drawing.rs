//! A collection of semi-random shape and image drawing examples.

use ggez::{
    event,
    graphics::{
        self,
        canvas::{Canvas, CanvasLoadOp},
        draw::DrawParam,
        image::{Image, ScreenImage},
        mesh::{Mesh, MeshBuilder, MeshData, Vertex},
        sampler::Sampler,
        Color, DrawMode,
    },
    Context, GameResult,
};
use glam::*;
use std::{env, path};

struct MainState {
    frame: ScreenImage,
    image1: Image,
    image2: Image,
    meshes: Vec<(Option<Image>, Mesh)>,
    rect: Mesh,
    rotation: f32,
}

impl MainState {
    /// Load images and create meshes.
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let frame = ScreenImage::new(&ctx.gfx, None, 1., 1., 1);

        let image1 = Image::from_path(ctx, "/dragon1.png", true)?;
        let image2 = Image::from_path(ctx, "/shot.png", true)?;

        let mb = &mut MeshBuilder::new();
        mb.rectangle(
            graphics::DrawMode::stroke(1.0),
            graphics::Rect::new(450.0, 450.0, 50.0, 50.0),
            graphics::Color::new(1.0, 0.0, 0.0, 1.0),
        )?;

        let rock = Image::from_path(ctx, "/rock.png", true)?;

        let meshes = vec![
            (None, build_mesh(ctx)?),
            (Some(rock), build_textured_triangle(ctx)?),
        ];

        let rect = Mesh::from_data(&ctx.gfx, mb.build());

        let s = MainState {
            frame,
            image1,
            image2,
            meshes,
            rect,
            rotation: 1.0,
        };

        Ok(s)
    }
}

fn build_mesh(ctx: &mut Context) -> GameResult<Mesh> {
    let mb = &mut MeshBuilder::new();

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
        DrawMode::fill(),
        Vec2::new(600.0, 200.0),
        50.0,
        120.0,
        1.0,
        Color::new(1.0, 1.0, 0.0, 1.0),
    )?;

    mb.circle(
        DrawMode::fill(),
        Vec2::new(600.0, 380.0),
        40.0,
        1.0,
        Color::new(1.0, 0.0, 1.0, 1.0),
    )?;

    Ok(Mesh::from_data(&ctx.gfx, mb.build()))
}

fn build_textured_triangle(ctx: &mut Context) -> GameResult<Mesh> {
    let triangle_verts = vec![
        Vertex {
            position: [100.0, 100.0],
            uv: [1.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.0, 100.0],
            uv: [0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let triangle_indices = vec![0, 1, 2];

    Ok(Mesh::from_data(
        &ctx.gfx,
        MeshData {
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
        let frame = self.frame.image(&ctx.gfx);

        let mut canvas = Canvas::from_image(
            &mut ctx.gfx,
            CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
            &frame,
        );

        // Draw an image.
        let dst = glam::Vec2::new(20.0, 20.0);
        canvas.draw(&self.image1, DrawParam::new().offset(dst));

        // Draw an image with some options, and different filter modes.
        let dst = glam::Vec2::new(200.0, 100.0);
        let dst2 = glam::Vec2::new(400.0, 400.0);
        let scale = glam::Vec2::new(10.0, 10.0);

        canvas.draw(
            &self.image2,
            DrawParam::new()
                .offset(dst)
                .rotation(self.rotation)
                .image_scale(true)
                .scale(scale),
        );
        canvas.set_sampler(Sampler::nearest_clamp());
        canvas.draw(
            &self.image2,
            DrawParam::new()
                .offset(dst2)
                .rotation(self.rotation)
                .scale(scale)
                .origin(vec2(0.5, 0.5)),
        );
        canvas.set_sampler(Sampler::linear_clamp());

        // Create and draw a filled rectangle mesh.
        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        canvas.draw(None, DrawParam::new().dst_rect(rect).color(Color::WHITE));

        // Create and draw a stroked rectangle mesh.
        canvas.draw_mesh(&self.rect, None, DrawParam::default());
        canvas.draw(None, DrawParam::new());

        // Draw some pre-made meshes
        for (image, mesh) in &self.meshes {
            canvas.draw_mesh(mesh, image, DrawParam::new().image_scale(false));
        }

        // Finished drawing, show it all on the screen!
        canvas.finish();
        ctx.gfx.present(&frame)?;

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
