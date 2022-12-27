//! Example that just prints out all the input events.

use ggez::conf;
use ggez::event::{self, Axis, Button, GamepadId, MouseButton};
use ggez::glam::*;
use ggez::graphics::{self, Color, DrawMode};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{Context, GameResult};

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

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.keyboard.is_key_pressed(KeyCode::A) {
            println!("The A key is pressed");
            if ctx
                .keyboard
                .is_mod_active(ggez::input::keyboard::KeyMods::SHIFT)
            {
                println!("The shift key is held too.");
            }
            println!(
                "Full list of pressed keys: {:?}",
                ctx.keyboard.pressed_keys()
            );
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.1, 0.2, 0.3, 1.0]));
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            graphics::Rect {
                x: self.pos_x,
                y: self.pos_y,
                w: 400.0,
                h: 300.0,
            },
            Color::WHITE,
        )?;
        canvas.draw(&rectangle, Vec2::new(0.0, 0.0));
        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse_down = true;
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse_down = false;
        println!("Mouse button released: {:?}, x: {}, y: {}", button, x, y);
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    ) -> GameResult {
        if self.mouse_down {
            // Mouse coordinates are PHYSICAL coordinates, but here we want logical coordinates.

            // If you simply use the initial coordinate system, then physical and logical
            // coordinates are identical.
            self.pos_x = x;
            self.pos_y = y;

            // If you change your screen coordinate system you need to calculate the
            // logical coordinates like this:
            /*
            let screen_rect = graphics::screen_coordinates(_ctx);
            let size = graphics::window(_ctx).inner_size();
            self.pos_x = (x / (size.width  as f32)) * screen_rect.w + screen_rect.x;
            self.pos_y = (y / (size.height as f32)) * screen_rect.h + screen_rect.y;
            */
        }
        println!(
            "Mouse motion, x: {}, y: {}, relative x: {}, relative y: {}",
            x, y, xrel, yrel
        );
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) -> GameResult {
        println!("Mousewheel event, x: {}, y: {}", x, y);
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, repeat: bool) -> GameResult {
        println!(
            "Key pressed: scancode {}, keycode {:?}, modifier {:?}, repeat: {}",
            input.scancode, input.keycode, input.mods, repeat
        );
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        println!(
            "Key released: scancode {}, keycode {:?}, modifier {:?}",
            input.scancode, input.keycode, input.mods
        );
        Ok(())
    }

    fn text_input_event(&mut self, _ctx: &mut Context, ch: char) -> GameResult {
        println!("Text input: {}", ch);
        Ok(())
    }

    fn gamepad_button_down_event(
        &mut self,
        _ctx: &mut Context,
        btn: Button,
        id: GamepadId,
    ) -> GameResult {
        println!("Gamepad button pressed: {:?} Gamepad_Id: {:?}", btn, id);
        Ok(())
    }

    fn gamepad_button_up_event(
        &mut self,
        _ctx: &mut Context,
        btn: Button,
        id: GamepadId,
    ) -> GameResult {
        println!("Gamepad button released: {:?} Gamepad_Id: {:?}", btn, id);
        Ok(())
    }

    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut Context,
        axis: Axis,
        value: f32,
        id: GamepadId,
    ) -> GameResult {
        println!(
            "Axis Event: {:?} Value: {} Gamepad_Id: {:?}",
            axis, value, id
        );
        Ok(())
    }

    fn focus_event(&mut self, _ctx: &mut Context, gained: bool) -> GameResult {
        if gained {
            println!("Focus gained");
        } else {
            println!("Focus lost");
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("input_test", "ggez").window_mode(
        conf::WindowMode::default()
            .fullscreen_type(conf::FullscreenType::Windowed)
            .resizable(true),
    );
    let (ctx, event_loop) = cb.build()?;

    // remove the comment to see how physical mouse coordinates can differ
    // from logical game coordinates when the screen coordinate system changes
    // canvas.set_screen_coordinates(Rect::new(20.0, 50.0, 2000.0, 1000.0));

    // alternatively, resizing the window also leads to screen coordinates
    // and physical window size being out of sync

    let state = MainState::new();
    event::run(ctx, event_loop, state)
}
