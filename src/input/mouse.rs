//! Mouse utility functions.

use crate::{error::GameError, error::GameResult, graphics::Point2};
use std::collections::HashMap;
use winit::dpi;

use crate::graphics::context::GraphicsContext;
pub use winit::event::MouseButton;
pub use winit::window::CursorIcon;

/// Stores state information for the mouse.
#[derive(Clone, Debug)]
pub struct MouseContext {
    last_position: Point2,
    last_delta: Point2,
    delta: Point2,
    buttons_pressed: HashMap<MouseButton, bool>,
    cursor_type: CursorIcon,
    cursor_grabbed: bool,
    cursor_hidden: bool,
}

impl MouseContext {
    pub(crate) fn new() -> Self {
        Self {
            last_position: Point2::ZERO,
            last_delta: Point2::ZERO,
            delta: Point2::ZERO,
            cursor_type: CursorIcon::Default,
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
        let _ = self.buttons_pressed.insert(button, pressed);
    }

    /// Returns the current mouse cursor type of the window.
    pub fn cursor_type(&self) -> CursorIcon {
        self.cursor_type
    }

    /// Modifies the mouse cursor type of the window.
    pub fn set_cursor_type(&mut self, gfx: &mut GraphicsContext, cursor_type: CursorIcon) {
        self.cursor_type = cursor_type;
        gfx.window.set_cursor_icon(cursor_type);
    }

    /// Get whether or not the mouse is grabbed (confined to the window).
    pub fn cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    /// Set whether or not the mouse is grabbed (confined to the window).
    pub fn set_cursor_grabbed(
        &mut self,
        gfx: &mut GraphicsContext,
        grabbed: bool,
    ) -> GameResult<()> {
        self.cursor_grabbed = grabbed;
        gfx.window
            .set_cursor_grab(grabbed)
            .map_err(|e| GameError::WindowError(e.to_string()))
    }

    /// Return whether or not the mouse is hidden (invisible).
    pub fn cursor_hidden(&self) -> bool {
        self.cursor_hidden
    }

    /// Set whether or not the mouse is hidden (invisible).
    pub fn set_cursor_hidden(&mut self, gfx: &mut GraphicsContext, hidden: bool) {
        self.cursor_hidden = hidden;
        gfx.window.set_cursor_visible(!hidden);
    }

    /// Get the current position of the mouse cursor, in pixels.
    /// Complement to [`set_position()`](fn.set_position.html).
    /// Uses strictly window-only coordinates.
    pub fn position(&self) -> mint::Point2<f32> {
        self.last_position.into()
    }

    /// Set the current position of the mouse cursor, in pixels.
    /// Uses strictly window-only coordinates.
    pub fn set_position(
        &mut self,
        gfx: &mut GraphicsContext,
        point: impl Into<mint::Point2<f32>>,
    ) -> GameResult<()> {
        let point = point.into();
        self.last_position = point.into();
        gfx.window
            .set_cursor_position(dpi::LogicalPosition {
                x: f64::from(point.x),
                y: f64::from(point.y),
            })
            .map_err(|_| GameError::WindowError("Couldn't set mouse cursor position!".to_owned()))
    }

    /// Get the distance the cursor was moved during the current frame, in pixels.
    pub fn delta(&self) -> mint::Point2<f32> {
        self.delta.into()
    }

    /// Get the distance the cursor was moved between the latest two mouse_motion_events.
    pub fn last_delta(&self) -> mint::Point2<f32> {
        self.last_delta.into()
    }

    /// Returns whether or not the given mouse button is pressed.
    pub fn button_pressed(&self, button: MouseButton) -> bool {
        *(self.buttons_pressed.get(&button).unwrap_or(&false))
    }
}

impl Default for MouseContext {
    fn default() -> Self {
        Self::new()
    }
}
