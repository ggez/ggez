//! Example that just prints out all the input events.

use ggez;

use anyhow::Result;
use ggez::event::{self, Axis, Button, GamepadId, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, DrawMode};
use ggez::input;
use ggez::Context;

struct MainState {
    pos_x: f32,
    pos_y: f32,
    mouse_down: bool,
}

impl MainState {
    fn new() -> MainState {
        MainState {
            pos_x: 100.0,
            pos_y: 100.0,
            mouse_down: false,
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {
        if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
            println!("The A key is pressed");
            if input::keyboard::is_mod_active(ctx, input::keyboard::KeyMods::SHIFT) {
                println!("The shift key is held too.");
            }
            println!(
                "Full list of pressed keys: {:?}",
                input::keyboard::pressed_keys(ctx)
            );
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<()> {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            graphics::Rect {
                x: self.pos_x,
                y: self.pos_y,
                w: 400.0,
                h: 300.0,
            },
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &rectangle, (ggez::nalgebra::Point2::new(0.0, 0.0),))?;
        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_down = true;
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_down = false;
        println!("Mouse button released: {:?}, x: {}, y: {}", button, x, y);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, xrel: f32, yrel: f32) {
        if self.mouse_down {
            self.pos_x = x;
            self.pos_y = y;
        }
        println!(
            "Mouse motion, x: {}, y: {}, relative x: {}, relative y: {}",
            x, y, xrel, yrel
        );
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        println!("Mousewheel event, x: {}, y: {}", x, y);
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool,
    ) {
        println!(
            "Key pressed: {:?}, modifier {:?}, repeat: {}",
            keycode, keymod, repeat
        );
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, keymod: KeyMods) {
        println!("Key released: {:?}, modifier {:?}", keycode, keymod);
    }

    fn text_input_event(&mut self, _ctx: &mut Context, ch: char) {
        println!("Text input: {}", ch);
    }

    fn gamepad_button_down_event(&mut self, _ctx: &mut Context, btn: Button, id: GamepadId) {
        println!("Gamepad button pressed: {:?} Gamepad_Id: {:?}", btn, id);
    }

    fn gamepad_button_up_event(&mut self, _ctx: &mut Context, btn: Button, id: GamepadId) {
        println!("Gamepad button released: {:?} Gamepad_Id: {:?}", btn, id);
    }

    fn gamepad_axis_event(&mut self, _ctx: &mut Context, axis: Axis, value: f32, id: GamepadId) {
        println!(
            "Axis Event: {:?} Value: {} Gamepad_Id: {:?}",
            axis, value, id
        );
    }

    fn focus_event(&mut self, _ctx: &mut Context, gained: bool) {
        if gained {
            println!("Focus gained");
        } else {
            println!("Focus lost");
        }
    }
}

pub fn main() -> Result<()> {
    let cb = ggez::ContextBuilder::new("input_test", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new();
    event::run(ctx, event_loop, state)
}
