/// Based on the bunnymark example from [`tetra`](https://crates.io/crates/tetra)
/// which is based on <https://github.com/openfl/openfl-samples/tree/master/demos/BunnyMark>
/// Original BunnyMark (and sprite) by Iain Lobb
use std::env;
use std::path;

use ggez::input::keyboard;
use oorandom::Rand32;

use ggez::graphics::{Color, Image, InstanceArray};
use ggez::Context;
use ggez::*;

use ggez::glam::*;
use ggez::input::keyboard::KeyInput;

// NOTE: Using a high number here yields worse performance than adding more bunnies over
// time - I think this is due to all of the RNG being run on the same tick...
const INITIAL_BUNNIES: usize = 1000;
const WIDTH: u16 = 800;
const HEIGHT: u16 = 600;
const GRAVITY: f32 = 0.5;

struct Bunny {
    position: Vec2,
    velocity: Vec2,
}

impl Bunny {
    fn new(rng: &mut Rand32) -> Bunny {
        let x_vel = rng.rand_float() * 5.0;
        let y_vel = (rng.rand_float() * 5.0) - 2.5;

        Bunny {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(x_vel, y_vel),
        }
    }
}

struct GameState {
    rng: Rand32,
    texture: Image,
    bunnies: Vec<Bunny>,
    max_x: f32,
    max_y: f32,

    click_timer: i32,
    bunnybatch: InstanceArray,
    batched_drawing: bool,
}

impl GameState {
    fn new(ctx: &mut Context) -> ggez::GameResult<GameState> {
        // We just use the same RNG seed every time.
        let mut rng = Rand32::new(12345);
        let texture = Image::from_path(ctx, "/wabbit_alpha.png")?;
        let mut bunnies = Vec::with_capacity(INITIAL_BUNNIES);
        let max_x = (WIDTH - texture.width() as u16) as f32;
        let max_y = (HEIGHT - texture.height() as u16) as f32;

        for _ in 0..INITIAL_BUNNIES {
            bunnies.push(Bunny::new(&mut rng));
        }

        let mut bunnybatch = InstanceArray::new(ctx, texture.clone());
        bunnybatch.resize(ctx, INITIAL_BUNNIES);

        Ok(GameState {
            rng,
            texture,
            bunnies,
            max_x,
            max_y,

            click_timer: 0,
            bunnybatch,
            batched_drawing: true,
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.click_timer > 0 {
            self.click_timer -= 1;
        }

        for bunny in &mut self.bunnies {
            bunny.position += bunny.velocity;
            bunny.velocity += Vec2::new(0.0, GRAVITY);

            if bunny.position.x > self.max_x {
                bunny.velocity *= Vec2::new(-1.0, 0.);
                bunny.position.x = self.max_x;
            } else if bunny.position.x < 0.0 {
                bunny.velocity *= Vec2::new(-1.0, 0.0);
                bunny.position.x = 0.0;
            }

            if bunny.position.y > self.max_y {
                bunny.velocity.y *= -0.8;
                bunny.position.y = self.max_y;

                // Flip a coin
                if self.rng.rand_i32() > 0 {
                    bunny.velocity -= Vec2::new(0.0, 3.0 + (self.rng.rand_float() * 4.0));
                }
            } else if bunny.position.y < 0.0 {
                bunny.velocity.y = 0.0;
                bunny.position.y = 0.0;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from((0.392, 0.584, 0.929)));

        if self.batched_drawing {
            self.bunnybatch.set(
                self.bunnies
                    .iter()
                    .map(|bunny| graphics::DrawParam::new().dest(bunny.position)),
            );

            canvas.draw(&self.bunnybatch, graphics::DrawParam::default());
        } else {
            for bunny in &self.bunnies {
                canvas.draw(
                    &self.texture,
                    graphics::DrawParam::new().dest(bunny.position),
                );
            }
        }

        ctx.gfx.set_window_title(&format!(
            "BunnyMark - {} bunnies - {:.0} FPS - batched drawing: {}",
            self.bunnies.len(),
            ctx.time.fps(),
            self.batched_drawing
        ));

        canvas.finish(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if input.keycode == Some(keyboard::KeyCode::Space) {
            self.batched_drawing = !self.batched_drawing;
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: input::mouse::MouseButton,
        _x: f32,
        _y: f32,
    ) -> GameResult {
        if button == input::mouse::MouseButton::Left && self.click_timer == 0 {
            for _ in 0..INITIAL_BUNNIES {
                self.bunnies.push(Bunny::new(&mut self.rng));
            }
            self.click_timer = 10;
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("bunnymark", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
