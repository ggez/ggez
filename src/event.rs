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
//! See the `eventloop` example for an implementation.

use winit;

/// A key code.
pub use winit::VirtualKeyCode as Keycode;

/// A struct that holds the state of keyboard modifier buttons such as ctrl or shift.
pub use winit::ModifiersState as KeyboardModifiers;
/// A mouse button.
pub use winit::MouseButton;

/// An analog axis of some device (controller, joystick...).
// TODO: verify.
pub use winit::AxisId as Axis;
/// A button of some device (controller, joystick...).
pub use winit::ButtonId as Button;

/// `winit` events; nested in a module for re-export neatness.
pub mod winit_event {
    pub use super::winit::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState};
}
use self::winit_event::*;
/// `winit` event loop.
pub use winit::EventsLoop;

use GameResult;
use context::Context;

/// A trait defining event callbacks; your primary interface with
/// `ggez`'s event loop.  Have a type implement this trait and
/// override at least the update() and draw() methods, then pass it to
/// `event::run()` to run the game's mainloop.
///
/// The default event handlers do nothing, apart from
/// `key_down_event()`, which will by default exit the game if escape
/// is pressed.  Just override the methods you want to do things with.
pub trait EventHandler {
    /// Called upon each physics update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// `graphics::clear()` and end it with
    /// `graphics::present()` and `timer::yield_now()`
    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()>;

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
    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _xrel: f32, _yrel: f32) {
    }

    /// The mousewheel was scrolled, vertically (y, positive away from and negative toward the user)
    /// or horizontally (x, positive to the right and negative to the left).
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}

    /// A keyboard button was pressed.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: Keycode,
        _keymods: KeyboardModifiers,
        _repeat: bool,
    ) {
        if keycode == Keycode::Escape {
            ctx.quit();
        }
    }

    /// A keyboard button was released.
    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: Keycode, _keymods: KeyboardModifiers) {
    }

    /// Candidate text is passed by the OS (via Input Method Editor).
    /// Refer to:
    /// <https://wiki.libsdl.org/SDL_TextEditingEvent>
    /// <https://wiki.libsdl.org/SDL_TextInputEvent>
    /// <https://wiki.libsdl.org/Tutorials/TextInput>
    fn text_editing_event(&mut self, _ctx: &mut Context, _text: String, _start: i32, _length: i32) {
    }

    /// Resulting text (usually a unicode character) is passed by the OS (via Input Method Editor).
    /// Refer to:
    /// <https://wiki.libsdl.org/SDL_TextEditingEvent>
    /// <https://wiki.libsdl.org/SDL_TextInputEvent>
    /// <https://wiki.libsdl.org/Tutorials/TextInput>
    fn text_input_event(&mut self, _ctx: &mut Context, _text: String) {}

    /// A controller button was pressed; instance_id identifies which controller.
    fn controller_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _btn: Button,
        _instance_id: i32,
    ) {
    }
    /// A controller button was released.
    fn controller_button_up_event(&mut self, _ctx: &mut Context, _btn: Button, _instance_id: i32) {}
    /// A controller axis moved.
    fn controller_axis_event(
        &mut self,
        _ctx: &mut Context,
        _axis: Axis,
        _value: i16,
        _instance_id: i32,
    ) {
    }

    /// Called when the window is shown or hidden.
    fn focus_event(&mut self, _ctx: &mut Context, _gained: bool) {}

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit.
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        debug!("quit_event() callback called, quitting...");
        false
    }

    /// Called when the user resizes the window.
    /// Is not called when you resize it yourself with
    /// `graphics::set_mode()` though.
    fn resize_event(&mut self, _ctx: &mut Context, _width: u32, _height: u32) {}
}

/// Runs the game's main loop, calling event callbacks on the given state
/// object as events occur.
///
/// It does not try to do any type of framerate limiting.  See the
/// documentation for the `timer` module for more info.
pub fn run<S>(ctx: &mut Context, state: &mut S) -> GameResult<()>
where
    S: EventHandler,
{
    use keyboard;
    use mouse;

    let mut events_loop = &ctx.events_loop;

    while ctx.continuing {
        ctx.timer_context.tick();

        events_loop.poll_events(|event| {
            ctx.process_event(&event);
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            if state.quit_event(ctx) {
                                ctx.quit();
                            }
                        }
                        WindowEvent::Focused(gained) => {
                            state.focus_event(ctx, gained);
                        }
                        WindowEvent::KeyboardInput {
                            input: winit::KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                modifiers,
                                ..
                            },
                            ..
                        } => {
                            match state {
                                ElementState::Pressed => {
                                    let repeat = keyboard::is_repeated(ctx, keycode);
                                    state.key_down_event(ctx, keycode, modifiers, repeat);
                                }
                                ElementState::Released => {
                                    state.key_up_event(ctx, keycode, modifiers);
                                }
                            }
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            let position = mouse::get_position(ctx);
                            match state {
                                ElementState::Pressed => {
                                    state.mouse_button_down_event(
                                        ctx,
                                        button,
                                        position.x,
                                        position.y,
                                    )
                                }
                                ElementState::Released => {
                                    state.mouse_button_up_event(
                                        ctx,
                                        button,
                                        position.x,
                                        position.y,
                                    )
                                }
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let position = mouse::get_position(ctx);
                            let delta = mouse::get_delta(ctx);
                            state.mouse_motion_event(
                                ctx,
                                position.x,
                                position.y,
                                delta.x,
                                delta.y,
                            );
                        }
                        _ => (),
                    }
                }
                Event::DeviceEvent { event, .. } => {
                    match event {
                        _ => (),
                    }
                }
                Event::Awakened => unimplemented!(),
                Event::Suspended(_) => unimplemented!(),
            }
        });
        /*{
                TextEditing {
                    text,
                    start,
                    length,
                    ..
                } => state.text_editing_event(ctx, text, start, length),
                TextInput { text, .. } => state.text_input_event(ctx, text),
                MouseWheel { x, y, .. } => state.mouse_wheel_event(ctx, x, y),
                ControllerButtonDown { button, which, .. } => {
                    state.controller_button_down_event(ctx, button, which)
                }
                ControllerButtonUp { button, which, .. } => {
                    state.controller_button_up_event(ctx, button, which)
                }
                ControllerAxisMotion {
                    axis, value, which, ..
                } => state.controller_axis_event(ctx, axis, value, which),
                Window {
                    win_event: event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    state.resize_event(ctx, w as u32, h as u32);
                }
                _ => {}
            }*/
        state.update(ctx)?;
        state.draw(ctx)?;
    }

    Ok(())
}
