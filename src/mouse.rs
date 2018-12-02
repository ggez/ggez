//! Mouse utility functions.

use context::Context;
use error::GameResult;
use graphics;
use graphics::Point2;

/// Stores state information for the mouse.
// ...what little of it there is.
#[derive(Clone, Debug)]
pub struct MouseContext {
    last_position: Point2,
}

impl MouseContext {
    /// Creates a new `MouseContext`.
    pub fn new() -> Self {
        Self {
            last_position: Point2::origin(),
        }
    }

    pub(crate) fn set_last_position(&mut self, p: Point2) {
        self.last_position = p;
    }
}

impl Default for MouseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Get whether or not the mouse is "grabbed", i.e., confined to the window.
pub fn get_grabbed(ctx: &Context) -> bool {
    graphics::get_window(ctx).grab()
}

/// Set whether or not the mouse is "grabbed", i.e., confined to the window.
pub fn set_grabbed(ctx: &mut Context, grabbed: bool) {
    graphics::get_window_mut(ctx).set_grab(grabbed)
}

/// Get whether or not the mouse is in relative mode.
///
/// In relative mode, the cursor is hidden and doesn't move when the mouse
/// does, but relative motion events are still generated.  This is useful
/// for things such as implementing mouselook in an FPS.
pub fn get_relative_mode(ctx: &Context) -> bool {
    ctx.sdl_context.mouse().relative_mouse_mode()
}

/// Set whether or not the mouse is in relative mode.
///
/// In relative mode, the cursor is hidden and doesn't move when the mouse
/// does, but relative motion events are still generated.  This is useful
/// for things such as implementing mouselook in an FPS.
pub fn set_relative_mode(ctx: &Context, mode: bool) {
    ctx.sdl_context.mouse().set_relative_mouse_mode(mode)
}

/// Get the current position of the mouse cursor, in pixels.
/// Complement to [`set_position()`](fn.set_position.html).
/// Uses strictly window-only coordinates.
pub fn get_position(ctx: &Context) -> GameResult<Point2> {
    // TODO: Next time we can break the API, remove the GameResult here.
    Ok(ctx.mouse_context.last_position)
}

/// Set the current position of the mouse cursor, in pixels.
/// Uses strictly window-only coordinates.
pub fn set_position(ctx: &Context, point: Point2) {
    let window = graphics::get_window(ctx);
    ctx.sdl_context
        .mouse()
        .warp_mouse_in_window(window, point.x as i32, point.y as i32)
}

/// Set whether or not the cursor is visible (not hidden) on the screen
pub fn set_cursor_visible(ctx: &Context, visible: bool) {
    ctx.sdl_context.mouse().show_cursor(visible)
}

/// Get whether or not the cursor is visible (not hidden) on the screen
pub fn get_cursor_visible(ctx: &Context) -> bool {
    ctx.sdl_context.mouse().is_cursor_showing()
}
