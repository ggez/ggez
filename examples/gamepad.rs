extern crate ggez;

use ggez::*;
use ggez::event::*;
use ggez::graphics::{Color, DrawMode, Point};
use std::time::Duration;

struct MainState {
    pos_x: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> MainState {
        MainState { pos_x: 100.0 }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::circle(ctx, DrawMode::Fill, Point { x: self.pos_x, y: 380.0 }, 100.0, 32)?;
        graphics::present(ctx);
        Ok(())
    }

    fn controller_button_down_event(&mut self, btn: Button, instance_id: i32) {
        println!("Button pressed: {:?} Controller_Id: {}", btn, instance_id);
    }


    fn controller_axis_event(&mut self, axis: Axis, value: i16, instance_id: i32) {
        println!("Axis Event: {:?} Value: {} Controller_Id: {}", axis, value, instance_id);
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("does_not_exist.toml", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx);
    event::run(ctx, state).unwrap();
}
