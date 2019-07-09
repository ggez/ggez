//! A collection of semi-random shape and image drawing examples.

use cgmath;

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::graphics::{Color, DrawMode, DrawParam};
use ggez::nalgebra::Point2;
use ggez::timer;
use ggez::{Context, GameResult};
use std::env;
use std::path;

struct MainState {
    image1: graphics::Image,
    image2_linear: graphics::Image,
    image2_nearest: graphics::Image,
    meshes: Vec<graphics::Mesh>,
    rotation: f32,
}

impl MainState {
    /// Load images and create meshes.
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image1 = graphics::Image::new(ctx, "/dragon1.png")?;
        let image2_linear = graphics::Image::new(ctx, "/shot.png")?;
        let mut image2_nearest = graphics::Image::new(ctx, "/shot.png")?;
        image2_nearest.set_filter(graphics::FilterMode::Nearest);

        let meshes = vec![build_mesh(ctx)?, build_textured_triangle(ctx)?];
        let s = MainState {
            image1,
            image2_linear,
            image2_nearest,
            meshes,
            rotation: 1.0,
        };

        Ok(s)
    }
}

fn build_mesh(ctx: &mut Context) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();

    mb.line(
        &[
            Point2::new(200.0, 200.0),
            Point2::new(400.0, 200.0),
            Point2::new(400.0, 400.0),
            Point2::new(200.0, 400.0),
            Point2::new(200.0, 300.0),
        ],
        4.0,
        Color::new(1.0, 0.0, 0.0, 1.0),
    )?;

    mb.ellipse(
        DrawMode::fill(),
        Point2::new(600.0, 200.0),
        50.0,
        120.0,
        1.0,
        Color::new(1.0, 1.0, 0.0, 1.0),
    );

    mb.circle(
        DrawMode::fill(),
        Point2::new(600.0, 380.0),
        40.0,
        1.0,
        Color::new(1.0, 0.0, 1.0, 1.0),
    );

    mb.build(ctx)
}

fn build_textured_triangle(ctx: &mut Context) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();
    let triangle_verts = vec![
        graphics::Vertex {
            pos: [100.0, 100.0],
            uv: [1.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        graphics::Vertex {
            pos: [0.0, 100.0],
            uv: [0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        graphics::Vertex {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let triangle_indices = vec![0, 1, 2];

    let i = graphics::Image::new(ctx, "/rock.png")?;
    mb.raw(&triangle_verts, &triangle_indices, Some(i));
    mb.build(ctx)
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.rotation += 0.01;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // Draw an image.
        let dst = cgmath::Point2::new(20.0, 20.0);
        graphics::draw(ctx, &self.image1, (dst,))?;

        // Draw an image with some options, and different filter modes.
        let dst = cgmath::Point2::new(200.0, 100.0);
        let dst2 = cgmath::Point2::new(400.0, 400.0);
        let scale = cgmath::Vector2::new(10.0, 10.0);

        graphics::draw(
            ctx,
            &self.image2_linear,
            graphics::DrawParam::new()
                .dest(dst)
                .rotation(self.rotation)
                .scale(scale),
        )?;
        graphics::draw(
            ctx,
            &self.image2_nearest,
            graphics::DrawParam::new()
                .dest(dst2)
                .rotation(self.rotation)
                .offset(Point2::new(0.5, 0.5))
                .scale(scale),
        )?;

        // Create and draw a filled rectangle mesh.
        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        let r1 =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::WHITE)?;
        graphics::draw(ctx, &r1, DrawParam::default())?;

        // Create and draw a stroked rectangle mesh.
        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        let r2 = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(1.0),
            rect,
            graphics::Color::new(1.0, 0.0, 0.0, 1.0),
        )?;
        graphics::draw(ctx, &r2, DrawParam::default())?;

        // Draw some pre-made meshes
        for m in &self.meshes {
            graphics::draw(ctx, m, DrawParam::new())?;
        }

        // Finished drawing, show it all on the screen!
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

    let cb = ggez::ContextBuilder::new("drawing", "ggez").add_resource_path(resource_dir);

    let (ctx, events_loop) = &mut cb.build()?;

    println!("{}", graphics::renderer_info(ctx)?);
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, events_loop, state)
}
