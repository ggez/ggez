//! Mouse utility functions.

use crate::context::Context;
use crate::error::GameError;
use crate::error::GameResult;
use crate::graphics;
use crate::graphics::Point2;
use std::collections::HashSet;
use winit::dpi;
pub use winit::event::MouseButton;
pub use winit::window::CursorIcon;

/// Stores state information for the mouse.
#[derive(Clone, Debug)]
pub struct MouseContext {
    last_position: Point2,
    last_delta: Point2,
    delta: Point2,
    buttons_pressed: HashSet<MouseButton>,
    cursor_type: CursorIcon,
    cursor_grabbed: bool,
    cursor_hidden: bool,
    previous_buttons_pressed: HashSet<MouseButton>,
}

impl MouseContext {
    pub(crate) fn new() -> Self {
        Self {
            last_position: Point2::ZERO,
            last_delta: Point2::ZERO,
            delta: Point2::ZERO,
            cursor_type: CursorIcon::Default,
            buttons_pressed: HashSet::new(),
            cursor_grabbed: false,
            cursor_hidden: false,
            previous_buttons_pressed: HashSet::new(),
        }
    }

    pub(crate) fn set_last_position(&mut self, p: Point2) {
        self.last_position = p;
    }

    pub(crate) fn set_last_delta(&mut self, p: Point2) {
        self.last_delta = p;
    }

    /// Resets the value returned by [`mouse::delta`](fn.delta.html) to zero.
    /// You shouldn't need to call this, except when you're running your own event loop.
    /// In this case call it right at the end, after `draw` and `update` have finished.
    pub fn reset_delta(&mut self) {
        self.delta = Point2::ZERO;
    }

    pub(crate) fn set_delta(&mut self, p: Point2) {
        self.delta = p;
    }

    pub(crate) fn set_button(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            let _ = self.buttons_pressed.insert(button);
        } else {
            let _ = self.buttons_pressed.remove(&button);
        }
    }

    fn button_pressed(&self, button: MouseButton) -> bool {
        self.buttons_pressed.contains(&button)
    }

    pub(crate) fn button_just_pressed(&self, button: MouseButton) -> bool {
        self.buttons_pressed.contains(&button) && !self.previous_buttons_pressed.contains(&button)
    }

    pub(crate) fn button_just_released(&self, button: MouseButton) -> bool {
        !self.buttons_pressed.contains(&button) && self.previous_buttons_pressed.contains(&button)
    }

    /// Copies the current state of the mouse buttons into the context. If you are writing your own event loop
    /// you need to call this at the end of every update in order to use the functions `is_button_just_pressed`
    /// and `is_button_just_released`. Otherwise this is handled for you.
    pub fn save_mouse_state(&mut self) {
        self.previous_buttons_pressed = self.buttons_pressed.clone();
    }
}

impl Default for MouseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the current mouse cursor type of the window.
pub fn cursor_type(ctx: &Context) -> CursorIcon {
    ctx.mouse_context.cursor_type
}

/// Modifies the mouse cursor type of the window.
pub fn set_cursor_type(ctx: &mut Context, cursor_type: CursorIcon) {
    ctx.mouse_context.cursor_type = cursor_type;
    graphics::window(ctx).set_cursor_icon(cursor_type);
}

/// Get whether or not the mouse is grabbed (confined to the window)
pub fn cursor_grabbed(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_grabbed
}

/// Set whether or not the mouse is grabbed (confined to the window)
pub fn set_cursor_grabbed(ctx: &mut Context, grabbed: bool) -> GameResult<()> {
    ctx.mouse_context.cursor_grabbed = grabbed;
    graphics::window(ctx)
        .set_cursor_grab(grabbed)
        .map_err(|e| GameError::WindowError(e.to_string()))
}

/// Set whether or not the mouse is hidden (invisible)
pub fn cursor_hidden(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_hidden
}

/// Set whether or not the mouse is hidden (invisible).
pub fn set_cursor_hidden(ctx: &mut Context, hidden: bool) {
    ctx.mouse_context.cursor_hidden = hidden;
    graphics::window(ctx).set_cursor_visible(!hidden)
}

/// Get the current position of the mouse cursor, in pixels.
/// Complement to [`set_position()`](fn.set_position.html).
/// Uses strictly window-only coordinates.
pub fn position(ctx: &Context) -> mint::Point2<f32> {
    ctx.mouse_context.last_position.into()
}

/// Set the current position of the mouse cursor, in pixels.
/// Uses strictly window-only coordinates.
pub fn set_position<P>(ctx: &mut Context, point: P) -> GameResult<()>
where
    P: Into<mint::Point2<f32>>,
{
    let mintpoint = point.into();
    ctx.mouse_context.last_position = Point2::from(mintpoint);
    graphics::window(ctx)
        .set_cursor_position(dpi::LogicalPosition {
            x: f64::from(mintpoint.x),
            y: f64::from(mintpoint.y),
        })
        .map_err(|_| GameError::WindowError("Couldn't set mouse cursor position!".to_owned()))
}

/// Get the distance the cursor was moved during the current frame, in pixels.
pub fn delta(ctx: &Context) -> mint::Point2<f32> {
    ctx.mouse_context.delta.into()
}

/// Get the distance the cursor was moved between the latest two mouse_motion_events.
pub(crate) fn last_delta(ctx: &Context) -> mint::Point2<f32> {
    ctx.mouse_context.last_delta.into()
}

/// Returns whether or not the given mouse button is pressed.
pub fn button_pressed(ctx: &Context, button: MouseButton) -> bool {
    ctx.mouse_context.button_pressed(button)
}

/// Returns whether or not the given mouse button has been pressed this frame.
pub fn button_just_pressed(ctx: &Context, button: MouseButton) -> bool {
    ctx.mouse_context.button_just_pressed(button)
}

/// Returns whether or not the given mouse button has been released this frame.
pub fn button_just_released(ctx: &Context, button: MouseButton) -> bool {
    ctx.mouse_context.button_just_released(button)
}

/// Updates delta and position values.
/// The inputs are interpreted as pixel coordinates inside the window.
///
/// This function is called internally whenever the mouse moves to a new location.
/// It can also be used to simulate mouse input.
/// (It gets called inside the default implementation of the
/// [`touch_event`](../../event/trait.EventHandler.html#method.touch_event), for example.)
/// Calling this function alone won't trigger a
/// [`mouse_motion_event`](../../event/trait.EventHandler.html#method.mouse_motion_event) though.
/// (Note that the default implementation of
/// [`touch_event`](../../event/trait.EventHandler.html#method.touch_event) DOES trigger one, but
/// it does so by invoking it on the `EventHandler` manually.)
pub fn handle_move(ctx: &mut Context, new_x: f32, new_y: f32) {
    let current_delta = crate::input::mouse::delta(ctx);
    let current_pos = crate::input::mouse::position(ctx);
    let diff = crate::graphics::Point2::new(new_x - current_pos.x, new_y - current_pos.y);
    // Sum up the cumulative mouse change for this frame in `delta`:
    ctx.mouse_context.set_delta(crate::graphics::Point2::new(
        current_delta.x + diff.x,
        current_delta.y + diff.y,
    ));
    // `last_delta` is not cumulative.
    // It represents only the change between the last mouse event and the current one.
    ctx.mouse_context.set_last_delta(diff);
    ctx.mouse_context
        .set_last_position(crate::graphics::Point2::new(new_x as f32, new_y as f32));
}
