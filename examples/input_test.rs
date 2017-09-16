extern crate ggez;

use ggez::*;
use ggez::event::*;
use ggez::graphics::{DrawMode, Point};
use std::time::Duration;

struct MainState {
    pos_x: f32,
}

impl MainState {
    fn new() -> MainState {
        MainState { pos_x: 100.0 }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::circle(ctx,
                         DrawMode::Fill,
                         Point::new(
                             self.pos_x,
                             380.0,
                         ),
                         100.0,
                         1.0)?;
        graphics::present(ctx);
        Ok(())
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: i32, y: i32) {
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: i32, y: i32) {
        println!("Mouse button released: {:?}, x: {}, y: {}", button, x, y);
    }

    fn mouse_motion_event(&mut self, _state: MouseState, x: i32, y: i32, xrel: i32, yrel: i32) {
        println!("Mouse motion, x: {}, y: {}, relative x: {}, relative y: {}",
                 x,
                 y,
                 xrel,
                 yrel);
    }

    fn mouse_wheel_event(&mut self, x: i32, y: i32) {
        println!("Mousewheel event, x: {}, y: {}", x, y);
    }


    fn key_down_event(&mut self, keycode: Keycode, keymod: Mod, repeat: bool) {
        println!("Key pressed: {:?}, modifier {:?}, repeat: {}",
                 keycode,
                 keymod,
                 repeat);
    }
    fn key_up_event(&mut self, keycode: Keycode, keymod: Mod, repeat: bool) {
        println!("Key released: {:?}, modifier {:?}, repeat: {}",
                 keycode,
                 keymod,
                 repeat);
    }

    fn controller_button_down_event(&mut self, btn: Button, instance_id: i32) {
        println!("Controller button pressed: {:?} Controller_Id: {}",
                 btn,
                 instance_id);
    }

    fn controller_button_up_event(&mut self, btn: Button, instance_id: i32) {
        println!("Controller button released: {:?} Controller_Id: {}",
                 btn,
                 instance_id);
    }

    fn controller_axis_event(&mut self, axis: Axis, value: i16, instance_id: i32) {
        println!("Axis Event: {:?} Value: {} Controller_Id: {}",
                 axis,
                 value,
                 instance_id);
    }


    fn focus_event(&mut self, gained: bool) {
        if gained {
            println!("Focus gained");
        } else {
            println!("Focus lost");
        }
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("event_test", "ggez", c).unwrap();
    let state = &mut MainState::new();
    event::run(ctx, state).unwrap();
}
