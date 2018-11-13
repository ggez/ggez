//! The `event` module contains traits and structs to actually run your game mainloop
//! and handle top-level state, as well as handle input events such as keyboard
//! and mouse.
//!
//! If you don't want to do this, you can write your own mainloop and
//! get the necessary event machinery by calling
//! `context.sdl_context.event_pump()` on your `Context`.  You can
//! then call whatever SDL event methods you want on that.  This is
//! not particularly elegant and is not guaranteed to be stable across
//! different versions of ggez (for instance, we may someday get rid of SDL2),
//! but trying to wrap it
//! up more conveniently really ends up with the exact same interface.
//!
//! TODO: UPDATE DOCS!
//!
//! See the `eventloop` example for an implementation.

use gilrs;
use winit::{self, dpi};

/// A mouse button.
pub use winit::MouseButton;

/// An analog axis of some device (controller, joystick...).
pub use gilrs::Axis;
/// A button of some device (controller, joystick...).
pub use gilrs::Button;

/// `winit` events; nested in a module for re-export neatness.
pub mod winit_event {
    pub use super::winit::{
        DeviceEvent, ElementState, Event, KeyboardInput, ModifiersState, MouseScrollDelta,
        TouchPhase, WindowEvent,
    };
}

use self::winit_event::*;
/// `winit` event loop.
pub use winit::EventsLoop;

use crate::context::Context;
use crate::error::GameResult;
pub use crate::input::keyboard::{KeyCode, KeyMods};

/// A trait defining event callbacks; your primary interface with
/// `ggez`'s event loop.  Have a type implement this trait and
/// override at least the update() and draw() methods, then pass it to
/// `event::run()` to run the game's mainloop.
///
/// The default event handlers do nothing, apart from
/// `key_down_event()`, which will by default exit the game if the escape
/// key is pressed.  Just override the methods you want to do things with.
pub trait EventHandler {
    /// Called upon each logic update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, _ctx: &mut Context) -> GameResult;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// `graphics::clear()` and end it with
    /// `graphics::present()` and `timer::yield_now()`
    fn draw(&mut self, _ctx: &mut Context) -> GameResult;

    /// A mouse button was pressed
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    /// A mouse button was released
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    /// The mouse was moved; it provides both absolute x and y coordinates in the window,
    /// and relative x and y coordinates compared to its last position.
    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}

    /// The mousewheel was scrolled, vertically (y, positive away from and negative toward the user)
    /// or horizontally (x, positive to the right and negative to the left).
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}

    /// A keyboard button was pressed.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Escape {
            super::quit(ctx);
        }
    }

    /// A keyboard button was released.
    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymods: KeyMods) {}

    /// A unicode character was received, usually from keyboard input.
    /// This is the intended way of facilitating text input.
    fn text_input_event(&mut self, _ctx: &mut Context, _character: char) {}

    /// A controller button was pressed; id identifies which controller.
    fn controller_button_down_event(&mut self, _ctx: &mut Context, _btn: Button, _id: usize) {}

    /// A controller button was released.
    fn controller_button_up_event(&mut self, _ctx: &mut Context, _btn: Button, _id: usize) {}

    /// A controller axis moved.
    fn controller_axis_event(&mut self, _ctx: &mut Context, _axis: Axis, _value: f32, _id: usize) {}

    /// Called when the window is shown or hidden.
    fn focus_event(&mut self, _ctx: &mut Context, _gained: bool) {}

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit.
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        debug!("quit_event() callback called, quitting...");
        false
    }

    /// Called when the user resizes the window, or when it is resized
    /// via `graphics::set_mode()`.
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
}

/// Runs the game's main loop, calling event callbacks on the given state
/// object as events occur.
///
/// It does not try to do any type of framerate limiting.  See the
/// documentation for the `timer` module for more info.
pub fn run<S>(ctx: &mut Context, events_loop: &mut EventsLoop, state: &mut S) -> GameResult
where
    S: EventHandler,
{
    use crate::input::{keyboard, mouse};

    while ctx.continuing {
        ctx.timer_context.tick();
        events_loop.poll_events(|event| {
            let event = ctx.process_event(&event);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(dpi::LogicalSize { width, height }) => {
                        state.resize_event(ctx, width as f32, height as f32);
                    }
                    WindowEvent::CloseRequested => {
                        if !state.quit_event(ctx) {
                            super::quit(ctx);
                        }
                    }
                    WindowEvent::Focused(gained) => {
                        state.focus_event(ctx, gained);
                    }
                    WindowEvent::ReceivedCharacter(ch) => {
                        state.text_input_event(ctx, ch);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                modifiers,
                                ..
                            },
                        ..
                    } => {
                        let repeat = keyboard::is_key_repeated(ctx);
                        state.key_down_event(ctx, keycode, modifiers.into(), repeat);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(keycode),
                                modifiers,
                                ..
                            },
                        ..
                    } => {
                        state.key_up_event(ctx, keycode, modifiers.into());
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let (x, y) = match delta {
                            MouseScrollDelta::LineDelta(x, y) => (x, y),
                            MouseScrollDelta::PixelDelta(dpi::LogicalPosition { x, y }) => {
                                (x as f32, y as f32)
                            }
                        };
                        state.mouse_wheel_event(ctx, x, y);
                    }
                    WindowEvent::MouseInput {
                        state: element_state,
                        button,
                        ..
                    } => {
                        let position = mouse::position(ctx);
                        match element_state {
                            ElementState::Pressed => {
                                state.mouse_button_down_event(ctx, button, position.x, position.y)
                            }
                            ElementState::Released => {
                                state.mouse_button_up_event(ctx, button, position.x, position.y)
                            }
                        }
                    }
                    WindowEvent::CursorMoved { .. } => {
                        let position = mouse::position(ctx);
                        let delta = mouse::delta(ctx);
                        state.mouse_motion_event(ctx, position.x, position.y, delta.x, delta.y);
                    }
                    x => {
                        trace!("ignoring window event {:?}", x);
                    }
                },
                Event::DeviceEvent { event, .. } => match event {
                    _ => (),
                },
                Event::Awakened => (),
                Event::Suspended(_) => (),
            }
        });
        if ctx.conf.modules.gamepad {
            while let Some(gilrs::Event { id, event, .. }) = ctx.gamepad_context.next_event() {
                match event {
                    gilrs::EventType::ButtonPressed(button, _) => {
                        state.controller_button_down_event(ctx, button, id);
                    }
                    gilrs::EventType::ButtonReleased(button, _) => {
                        state.controller_button_up_event(ctx, button, id);
                    }
                    gilrs::EventType::AxisChanged(axis, value, _) => {
                        state.controller_axis_event(ctx, axis, value, id);
                    }
                    _ => {}
                }
            }
        }
        state.update(ctx)?;
        state.draw(ctx)?;
    }

    Ok(())
}
