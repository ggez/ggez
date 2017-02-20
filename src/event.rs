//! The `event` module contains traits and structs to actually run your game mainloop
//! and handle top-level state, as well as handle input events such as keyboard
//! and mouse.

pub use sdl2::keyboard::Keycode;
pub use sdl2::keyboard::Mod;
pub use sdl2::mouse::MouseButton;
pub use sdl2::mouse::MouseState;

pub use sdl2::controller::Button;
pub use sdl2::controller::Axis;

use sdl2::event::Event::*;
use sdl2::event;
use sdl2::mouse;
use sdl2::keyboard;


use context::Context;
use GameResult;
use timer;

use std::time::Duration;




/// A trait defining event callbacks.  Have a type implement this trait
/// and override at least the update() and draw() methods, then pass it
/// to `event::run()` to run the game's mainloop.
///
/// The default event handlers do nothing, apart from `key_down_event()`,
/// which *should* by default exit the game if escape is pressed.
pub trait EventHandler {
    /// Called upon each physics update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// `graphics::clear()` and end it with
    /// `graphics::present()` and `timer::sleep_until_next_frame()`
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()>;

    // You don't have to override these if you don't want to; the defaults
    // do nothing.
    // It might be nice to be able to have custom event types and a map or
    // such of handlers?  Hmm, maybe later.
    fn mouse_button_down_event(&mut self, _button: mouse::MouseButton, _x: i32, _y: i32) {}

    fn mouse_button_up_event(&mut self, _button: mouse::MouseButton, _x: i32, _y: i32) {}

    fn mouse_motion_event(&mut self,
                          _state: mouse::MouseState,
                          _x: i32,
                          _y: i32,
                          _xrel: i32,
                          _yrel: i32) {
    }

    fn mouse_wheel_event(&mut self, _x: i32, _y: i32) {}

    fn key_down_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {}

    fn key_up_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {}

    fn controller_button_down_event(&mut self, _btn: Button) {}
    fn controller_button_up_event(&mut self, _btn: Button) {}
    fn controller_axis_event(&mut self, _axis: Axis, _value: i16) {}

    fn focus_event(&mut self, _gained: bool) {}

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit.
    fn quit_event(&mut self) -> bool {
        println!("Quitting game");
        false
    }
}

/// Runs the game's main loop, calling event callbacks on the given state
/// object as events occur.
///
/// It does not try to do any type of rate limiting, apart from calling
/// `timer::sleep(0)` at the end of each frame.
pub fn run<S>(ctx: &mut Context, state: &mut S) -> GameResult<()>
    where S: EventHandler
{
    {
        let mut event_pump = ctx.sdl_context.event_pump()?;

        let mut continuing = true;
        while continuing {
            ctx.timer_context.tick();

            for event in event_pump.poll_iter() {
                match event {
                    Quit { .. } => {
                        continuing = state.quit_event();
                        // println!("Quit event: {:?}", t);
                    }
                    // TODO: We need a good way to have
                    // a default like this, while still allowing
                    // it to be overridden.
                    // Bah, just put it in the GameState trait
                    // as the default function.
                    // But it doesn't have access to the context
                    // to call quit!  Bah.
                    KeyDown { keycode, keymod, repeat, .. } => {
                        if let Some(key) = keycode {
                            if key == keyboard::Keycode::Escape {
                                ctx.quit()?;
                            } else {
                                state.key_down_event(key, keymod, repeat)
                            }
                        }
                    }
                    KeyUp { keycode, keymod, repeat, .. } => {
                        if let Some(key) = keycode {
                            state.key_up_event(key, keymod, repeat)
                        }
                    }
                    MouseButtonDown { mouse_btn, x, y, .. } => {
                        state.mouse_button_down_event(mouse_btn, x, y)
                    }
                    MouseButtonUp { mouse_btn, x, y, .. } => {
                        state.mouse_button_up_event(mouse_btn, x, y)
                    }
                    MouseMotion { mousestate, x, y, xrel, yrel, .. } => {
                        state.mouse_motion_event(mousestate, x, y, xrel, yrel)
                    }
                    MouseWheel { x, y, .. } => state.mouse_wheel_event(x, y),
                    ControllerButtonDown { button, .. } => {
                        state.controller_button_down_event(button)
                    }
                    ControllerButtonUp { button, .. } => state.controller_button_up_event(button),
                    ControllerAxisMotion { axis, value, .. } => {
                        state.controller_axis_event(axis, value)
                    }
                    Window { win_event: event::WindowEvent::FocusGained, .. } => {
                        state.focus_event(true)
                    }
                    Window { win_event: event::WindowEvent::FocusLost, .. } => {
                        state.focus_event(false)
                    }
                    _ => {}
                }
            }

            let dt = timer::get_delta(ctx);
            state.update(ctx, dt)?;
            state.draw(ctx)?;
        }
    }

    Ok(())
}
