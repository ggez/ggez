//! How to use coroutines to update game state.

use std::cell::Cell;
use std::rc::Rc;

use ggez;
use ggez::event::{self, MouseButton};
use ggez::graphics::{self, Color, Text};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use ggez::task;

use rand::Rng;

struct MainState {
    pos_x: f32,
    pos_y: f32,
    // Updated by a main-thread coroutine, so we can use Rc instead of Arc.
    color: Rc<Cell<Color>>,
    // Prevents overlapping change coroutines from being started.
    changing: Rc<Cell<bool>>,
    text: Text
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let s = MainState {
            pos_x: 0.0,
            pos_y: 0.0,
            color: Rc::new(Cell::new(graphics::WHITE)),
            changing: Rc::new(Cell::new(false)),
            text: Text::new("Click the mouse button to start a coroutine which randomly changes the circle's color. How exciting!")
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (na::Point2::new(self.pos_x, self.pos_y), self.color.get()))?;

        graphics::draw(ctx, &self.text, (na::Point2::new(10.0, 10.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }

    // Make the circle follow the cursor.
    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.pos_x = x;
        self.pos_y = y;
    }

    // When the user clicks a mouse button, kick off a color-change coroutine.
    fn mouse_button_down_event(&mut self, ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        // This condition is used to ensure that an overlapping coroutine isn't running.
        if let false = self.changing.replace(true) {
            start_color_change_coroutine(ctx, self);
        }
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new()?;
    event::run(ctx, event_loop, state)
}

fn start_color_change_coroutine(ctx: &mut Context, state: &MainState) {
    let mut main_handle = ctx.main_handle();
    let circle_color = state.color.clone();
    let changing = state.changing.clone();
    task::spawn_on_main(ctx, async move {
        for i in 1..=20 {
            task::sleep_updates(&mut main_handle, i * 2).await;
            let mut rng = rand::thread_rng();
            let next_color = Color::from_rgba(
                rng.gen(),
                rng.gen(),
                rng.gen(),
                255
            );
            circle_color.replace(next_color);
        }
        // Allow a new coroutine to start.
        changing.replace(false);
    });
}
