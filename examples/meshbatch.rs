//! An example of how to use an `InstanceArray` to draw custom `Mesh`es with instanced draws.

use ggez::event;
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
use glam::*;
use oorandom::Rand32;
use std::env;
use std::f32::consts::PI;
use std::path;

const TWO_PI: f32 = 2.0 * PI;

struct MainState {
    mesh_batch: graphics::InstanceArray,
    mesh: graphics::Mesh,
}

impl ggez::Game for MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mesh = graphics::Mesh::from_data(
            ctx,
            graphics::MeshBuilder::new()
                .circle(
                    graphics::DrawMode::stroke(4.0),
                    Vec2::new(0.0, 0.0),
                    16.0,
                    1.0,
                    (0, 0, 255).into(),
                )?
                .line(
                    &[Vec2::new(0.0, 0.0), Vec2::new(16.0, 0.0)],
                    2.0,
                    (255, 255, 0).into(),
                )?
                .build(),
        );

        let mesh_batch = graphics::InstanceArray::new(ctx, None);
        let mut s = MainState { mesh_batch, mesh };
        let (w, h) = ctx.gfx.drawable_size();
        s.populate(ctx, w, h);
        Ok(s)
    }
}

impl MainState {
    fn populate(&mut self, ctx: &mut Context, width: f32, height: f32) {
        // On web `drawable_size` can report (0, 0) until the canvas `ResizeObserver` fires.
        // Clamp to at least 1×1 so `resize` doesn't trip capacity assert.
        let items_x = ((width / 64.0) as usize).max(1);
        let items_y = ((height / 64.0) as usize).max(1);
        self.mesh_batch.resize(ctx, items_x * items_y);

        let mut rng = Rand32::new(12345);
        self.mesh_batch.set((0..items_x).flat_map(|x| {
            (0..items_y).map(move |y| {
                let x = x as f32;
                let y = y as f32;

                DrawParam::new()
                    .dest(Vec2::new(x * 64.0, y * 64.0))
                    .rotation(rng.rand_float() * TWO_PI)
            })
        }));
    }
}

impl event::EventHandler for MainState {
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.populate(ctx, width, height);
        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.time.ticks().is_multiple_of(100) {
            println!("Delta frame time: {:?} ", ctx.time.delta());
            println!("Average FPS: {}", ctx.time.fps());
        }

        // Update up to the first 50 instances in the mesh batch.
        let delta_time = ctx.time.delta().as_secs_f32() * 1000.0;
        let instances = self.mesh_batch.instances();
        let count = instances.len().min(50);

        let mut updated_params = Vec::new();
        for i in 0..count {
            let mut p = instances[i];
            if let graphics::Transform::Values {
                ref mut rotation, ..
            } = p.transform
            {
                if i.is_multiple_of(2) {
                    *rotation += 0.001 * TWO_PI * delta_time;
                    if *rotation > TWO_PI {
                        *rotation -= TWO_PI;
                    }
                } else {
                    *rotation -= 0.001 * TWO_PI * delta_time;
                    if *rotation < 0.0 {
                        *rotation += TWO_PI;
                    }
                }
            }
            updated_params.push(p);
        }
        for i in 0..count {
            // TODO: this is pretty inefficient and also a bit ridiculous
            //       a way to update parts of an InstanceArray would be good to have
            self.mesh_batch.update(i as u32, updated_params[i]);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw the batch
        canvas.draw_instanced_mesh(
            self.mesh.clone(),
            &self.mesh_batch,
            DrawParam::new().dest(glam::Vec2::new(5.0, 16.0)),
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

    ggez::ContextBuilder::new("meshbatch", "ggez")
        .add_resource_path(resource_dir)
        .run::<MainState>()
}
