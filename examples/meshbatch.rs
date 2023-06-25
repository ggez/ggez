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

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut rng = Rand32::new(12345);
        let mesh = graphics::Mesh::from_data(
            ctx,
            graphics::MeshBuilder::new()
                .circle(
                    graphics::DrawMode::stroke(4.0),
                    Vec2::new(0.0, 0.0),
                    8.0,
                    1.0,
                    (0, 0, 255).into(),
                )?
                .line(
                    &[Vec2::new(0.0, 0.0), Vec2::new(8.0, 0.0)],
                    2.0,
                    (255, 255, 0).into(),
                )?
                .build(),
        );

        // Generate enough instances to fill the entire screen
        let size = ctx.gfx.drawable_size();
        let items_x = (size.0 / 16.0) as usize;
        let items_y = (size.1 / 16.0) as usize;
        let mut mesh_batch = graphics::InstanceArray::new(ctx, None);
        mesh_batch.resize(ctx, items_x * items_y);

        mesh_batch.set((0..items_x).flat_map(|x| {
            (0..items_y).map(move |y| {
                let x = x as f32;
                let y = y as f32;

                DrawParam::new()
                    .dest(Vec2::new(x * 16.0, y * 16.0))
                    .rotation(rng.rand_float() * TWO_PI)
            })
        }));

        let s = MainState { mesh_batch, mesh };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    #[allow(clippy::needless_range_loop)]
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.time.ticks() % 100 == 0 {
            println!("Delta frame time: {:?} ", ctx.time.delta());
            println!("Average FPS: {}", ctx.time.fps());
        }

        // Update first 50 instances in the mesh batch
        let delta_time = ctx.time.delta().as_secs_f32() * 1000.0;
        let instances = self.mesh_batch.instances();

        let mut updated_params = Vec::new();
        for i in 0..50 {
            let mut p = instances[i as usize];
            if let graphics::Transform::Values {
                ref mut rotation, ..
            } = p.transform
            {
                if (i % 2) == 0 {
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
        for i in 0..50 {
            // TODO: this is pretty inefficient and also a bit ridiculous
            //       a way to update parts of an InstanceArray would be good to have
            self.mesh_batch.update(i, updated_params[i as usize]);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw the batch
        canvas.draw_instanced_mesh(
            self.mesh.clone(),
            &self.mesh_batch,
            DrawParam::new().dest(glam::Vec2::new(5.0, 8.0)),
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

    let cb = ggez::ContextBuilder::new("meshbatch", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
