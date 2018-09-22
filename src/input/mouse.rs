//! Mouse utility functions.

use context::Context;
use graphics;
use graphics::Point2;
use std::collections::HashMap;
use winit::dpi;
pub use winit::{MouseButton, MouseCursor};
use GameError;
use GameResult;

/// Stores state information for the mouse,
/// what little of it there is.
#[derive(Clone, Debug)]
pub struct MouseContext {
    last_position: Point2,
    last_delta: Point2,
    buttons_pressed: HashMap<MouseButton, bool>,
    cursor_type: MouseCursor,
    cursor_grabbed: bool,
    cursor_hidden: bool,
}

impl MouseContext {
    pub(crate) fn new() -> Self {
        Self {
            last_position: Point2::origin(),
            last_delta: Point2::origin(),
            cursor_type: MouseCursor::Default,
            buttons_pressed: HashMap::new(),
            cursor_grabbed: false,
            cursor_hidden: false,
        }
    }

    pub(crate) fn set_last_position(&mut self, p: Point2) {
        self.last_position = p;
    }

    pub(crate) fn set_last_delta(&mut self, p: Point2) {
        self.last_delta = p;
    }

    pub(crate) fn set_button(&mut self, button: MouseButton, pressed: bool) {
        let _ = self.buttons_pressed.insert(button, pressed);
    }

    fn button_pressed(&self, button: MouseButton) -> bool {
        *(self.buttons_pressed.get(&button).unwrap_or(&false))
    }
}

impl Default for MouseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the current mouse cursor type of the window.
pub fn cursor_type(ctx: &Context) -> MouseCursor {
    ctx.mouse_context.cursor_type
}

/// Modifies the mouse cursor type of the window.
pub fn set_cursor_type(ctx: &mut Context, cursor_type: MouseCursor) {
    ctx.mouse_context.cursor_type = cursor_type;
    graphics::window(ctx).set_cursor(cursor_type);
}

/// Get whether or not the mouse is grabbed (confined to the window)
pub fn cursor_grabbed(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_grabbed
}

/// Set whether or not the mouse is grabbed (confined to the window)
pub fn set_cursor_grabbed(ctx: &mut Context, grabbed: bool) -> GameResult<()> {
    ctx.mouse_context.cursor_grabbed = grabbed;
    graphics::window(ctx)
        .grab_cursor(grabbed)
        .map_err(|e| GameError::WindowError(e.to_string()))
}

/// Set whether or not the mouse is hidden (invisible)
pub fn cursor_hidden(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_hidden
}

/// Set whether or not the mouse is hidden (invisible).
pub fn set_cursor_hidden(ctx: &mut Context, hidden: bool) {
    ctx.mouse_context.cursor_hidden = hidden;
    graphics::window(ctx).hide_cursor(hidden)
}

/// Get the current position of the mouse cursor, in pixels.
/// Complement to `set_position()`.
/// Uses strictly window-only coordinates.
pub fn position(ctx: &Context) -> Point2 {
    ctx.mouse_context.last_position
}

/// Set the current position of the mouse cursor, in pixels.
/// Uses strictly window-only coordinates.
pub fn set_position(ctx: &mut Context, point: Point2) -> GameResult<()> {
    ctx.mouse_context.last_position = point;
    graphics::window(ctx)
        .set_cursor_position(dpi::LogicalPosition {
            x: f64::from(point.x),
            y: f64::from(point.y),
        }).map_err(|_| GameError::WindowError("Couldn't set mouse cursor position!".to_owned()))
}

/// Get the distance the cursor was moved during last frame, in pixels.
pub fn delta(ctx: &Context) -> Point2 {
    ctx.mouse_context.last_delta
}

/// Returns whether or not the given mouse button is pressed.
pub fn button_pressed(ctx: &Context, button: MouseButton) -> bool {
    ctx.mouse_context.button_pressed(button)
}

/// TODO: Can we implement this?  Check with Winit peoples.
/// Winit doesn't implement it itself, we can do it by locking
/// the cursor to the window and resetting it to center each frame?
pub fn relative_mode() -> bool {
    unimplemented!()
}

/// TODO: Can we impement this?  Check with Winit peoples.
pub fn set_relative_mode() {
    unimplemented!()
}
