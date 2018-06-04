//! Mouse utility functions.

use context::Context;
use graphics;
use graphics::Point2;
use GameResult;
pub use winit::{CursorState, MouseCursor};

/// Stores state information for the mouse,
/// what little of it there is.
#[derive(Clone, Debug)]
pub struct MouseContext {
    last_position: Point2,
    last_delta: Point2,
    cursor_type: MouseCursor,
    cursor_state: CursorState,
}

impl MouseContext {
    pub(crate) fn new() -> Self {
        Self {
            last_position: Point2::origin(),
            last_delta: Point2::origin(),
            cursor_type: MouseCursor::Default,
            cursor_state: CursorState::Normal,
        }
    }

    pub(crate) fn set_last_position(&mut self, p: Point2) {
        self.last_position = p;
    }

    pub(crate) fn set_last_delta(&mut self, p: Point2) {
        self.last_position = p;
    }
}

impl Default for MouseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the current mouse cursor type of the window.
pub fn get_cursor_type(ctx: &Context) -> MouseCursor {
    ctx.mouse_context.cursor_type
}

/// Modifies the mouse cursor type of the window.
pub fn set_cursor_type(ctx: &mut Context, cursor_type: MouseCursor) {
    ctx.mouse_context.cursor_type = cursor_type;
    graphics::get_window(ctx).set_cursor(cursor_type);
}

/// Set whether or not the mouse is grabbed (confined to the window) or hidden (invisible).
pub fn get_cursor_state(ctx: &Context) -> CursorState {
    ctx.mouse_context.cursor_state
}

/// Set whether or not the mouse is grabbed (confined to the window) or hidden (invisible).
pub fn set_cursor_state(ctx: &mut Context, state: CursorState) -> GameResult<()> {
    ctx.mouse_context.cursor_state = state;
    graphics::get_window(ctx).set_cursor_state(state).map_err(|e| e.into())
}

/// Get the current position of the mouse cursor, in pixels.
/// Complement to `set_position()`.
/// Uses strictly window-only coordinates.
pub fn get_position(ctx: &Context) -> Point2 {
    ctx.mouse_context.last_position
}

/// Get the distance the cursor was moved during last frame, in pixels.
pub fn get_delta(ctx: &Context) -> Point2 {
    ctx.mouse_context.last_delta
}

/// Set the current position of the mouse cursor, in pixels.
/// Uses strictly window-only coordinates.
pub fn set_position(ctx: &mut Context, point: Point2) -> GameResult<()> {
    ctx.mouse_context.last_position = point;
    if graphics::get_window(ctx).set_cursor_position(point.x as i32, point.y as i32).is_err() {
        return Err("Couldn't set mouse cursor position!".to_owned().into());
    }
    Ok(())
}
