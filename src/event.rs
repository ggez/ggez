//! The `event` module contains traits and structs to actually run your game mainloop
//! and handle top-level state, as well as handle input events such as keyboard
//! and mouse.
//!
//! If you don't want to do this, you can write your own mainloop and
//! get the necessary event machinery by calling
//! [`Context::sdl_context.event_pump()`](../struct.Context.html#structfield.sdl_context).
//! You can then call whatever SDL event methods you want on that.  This is
//! not particularly elegant and is not guaranteed to be stable across
//! different versions of `ggez` (for instance, we may someday get rid of SDL2),
//! but trying to wrap it up more conveniently really ends up with the exact same
//! interface.
//!
//! See the `eventloop` example for an implementation.
//!
//! Note: Most of these types are re-exported from
//! [`sdl2::{mouse, keyboard, controller, event}`](https://docs.rs/sdl2/).
// This note is here because the sdl2 crate has next to no documentation on the specific
// enums, constants, etc., and since it's `pub use`'d, there's no link back to sdl2.

use sdl2;

/// A key code.
pub use sdl2::keyboard::Keycode;

/// A struct that holds the state of modifier buttons such as ctrl or shift.
pub use sdl2::keyboard::Mod;
/// A mouse button press.
pub use sdl2::mouse::MouseButton;
/// A struct containing the mouse state at a given instant.
pub use sdl2::mouse::MouseState;

/// A controller axis.
pub use sdl2::controller::Axis;
/// A controller button.
pub use sdl2::controller::Button;

pub use sdl2::event::Event;
/// The event iterator
pub use sdl2::event::EventPollIterator;

use sdl2::event;
use sdl2::event::Event::*;
use sdl2::keyboard;
use sdl2::mouse;

use context::Context;
use GameResult;

pub use sdl2::keyboard::{CAPSMOD, LALTMOD, LCTRLMOD, LGUIMOD, LSHIFTMOD, MODEMOD, NOMOD, NUMMOD,
                         RALTMOD, RCTRLMOD, RESERVEDMOD, RGUIMOD, RSHIFTMOD};

/// A trait defining event callbacks; your primary interface with
/// `ggez`'s event loop.  Have a type implement this trait and
/// override at least the [`update()`](#tymethod.update) and [`draw()`](#tymethod.draw)
/// methods, then pass it to [`event::run()`](fn.run.html) to run the game's mainloop.
///
/// The default event handlers do nothing, apart from
/// [`key_down_event()`](#tymethod.key_down_event), which will by default exit the game
/// if escape is pressed.  Just override the methods you want to do things with.
pub trait EventHandler {
    /// Called upon each physics update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// [`graphics::clear()`](../graphics/fn.clear.html) and end it with
    /// [`graphics::present()`](../graphics/fn.present.html) and 
    /// [`timer::yield_now()`](../timer/fn.yield_now.html).
    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()>;

    /// A mouse button was pressed
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: mouse::MouseButton,
        _x: i32,
        _y: i32,
    ) {
    }

    /// A mouse button was released
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: mouse::MouseButton,
        _x: i32,
        _y: i32,
    ) {
    }

    /// The mouse was moved; it provides both absolute x and y coordinates in the window,
    /// and relative x and y coordinates compared to its last position.
    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _state: mouse::MouseState,
        _x: i32,
        _y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
    }

    /// The mousewheel was scrolled, vertically (y, positive away from and negative toward the user)
    /// or horizontally (x, positive to the right and negative to the left).
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: i32, _y: i32) {}

    /// A keyboard button was pressed.
    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        if keycode == keyboard::Keycode::Escape {
            ctx.quit().expect("Should never fail");
        }
    }

    /// A keyboard button was released.
    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
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

    /// A controller button was pressed; `instance_id` identifies which controller.
    /// This can be used with [`get_gamepad()`](../input/fn.get_gamepad.html) to get more
    /// information about the controller.
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
    /// the game does not exit (the quit event is cancelled).
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        debug!("quit_event() callback called, quitting...");
        false
    }

    /// Called when the user resizes the window.
    /// Is not called when you resize it yourself with
    /// [`graphics::set_mode()`](../graphics/fn.set_mode.html), though.
    fn resize_event(&mut self, _ctx: &mut Context, _width: u32, _height: u32) {}
}

/// A handle to access the OS's event pump.
pub struct Events(sdl2::EventPump);

use std::fmt;
impl fmt::Debug for Events {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Events: {:p}>", self)
    }
}

impl Events {
    /// Create a new `Events` object.
    pub fn new(ctx: &Context) -> GameResult<Events> {
        let e = ctx.sdl_context.event_pump()?;
        Ok(Events(e))
    }

    /// Get an iterator for all events.
    pub fn poll(&mut self) -> EventPollIterator {
        self.0.poll_iter()
    }
}

/// Runs the game's main loop, calling event callbacks on the given state
/// object as events occur.
///
/// It does not try to do any type of framerate limiting.  See the
/// documentation for the [`timer`](../timer/index.html) module for more info.
pub fn run<S>(ctx: &mut Context, state: &mut S) -> GameResult<()>
where
    S: EventHandler,
{
    let mut event_pump = ctx.sdl_context.event_pump()?;

    let mut continuing = true;
    while continuing {
        ctx.timer_context.tick();

        for event in event_pump.poll_iter() {
            ctx.process_event(&event);
            match event {
                Quit { .. } => {
                    continuing = state.quit_event(ctx);
                    // println!("Quit event: {:?}", t);
                }
                KeyDown {
                    keycode,
                    keymod,
                    repeat,
                    ..
                } => {
                    if let Some(key) = keycode {
                        state.key_down_event(ctx, key, keymod, repeat)
                    }
                }
                KeyUp {
                    keycode,
                    keymod,
                    repeat,
                    ..
                } => {
                    if let Some(key) = keycode {
                        state.key_up_event(ctx, key, keymod, repeat)
                    }
                }
                TextEditing {
                    text,
                    start,
                    length,
                    ..
                } => state.text_editing_event(ctx, text, start, length),
                TextInput { text, .. } => state.text_input_event(ctx, text),
                MouseButtonDown {
                    mouse_btn, x, y, ..
                } => state.mouse_button_down_event(ctx, mouse_btn, x, y),
                MouseButtonUp {
                    mouse_btn, x, y, ..
                } => state.mouse_button_up_event(ctx, mouse_btn, x, y),
                MouseMotion {
                    mousestate,
                    x,
                    y,
                    xrel,
                    yrel,
                    ..
                } => {
                    state.mouse_motion_event(ctx, mousestate, x, y, xrel, yrel);
                }
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
                    win_event: event::WindowEvent::FocusGained,
                    ..
                } => state.focus_event(ctx, true),
                Window {
                    win_event: event::WindowEvent::FocusLost,
                    ..
                } => state.focus_event(ctx, false),
                Window {
                    win_event: event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    state.resize_event(ctx, w as u32, h as u32);
                }
                _ => {}
            }
        }
        state.update(ctx)?;
        state.draw(ctx)?;
    }

    Ok(())
}
