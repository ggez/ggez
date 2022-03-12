//! Input handling modules for keyboard, mouse and gamepad.
pub mod gamepad;
pub mod keyboard;
pub mod mouse;
use self::{gamepad::GamepadContext, keyboard::KeyboardContext, mouse::MouseContext};
use crate::GameResult;

/// Contains contexts related to user input, i.e. keyboard, mouse and gamepads connected.
#[derive(Debug)]
pub struct InputContext {
    /// The mouse input context.
    pub mouse: MouseContext,
    /// The keyboard input context.
    pub keyboard: KeyboardContext,
    #[cfg(feature = "gamepad")]
    /// The gamepad input context.
    pub gamepad: GamepadContext,
}

impl InputContext {
    pub(crate) fn new() -> GameResult<Self> {
        Ok(Self {
            mouse: MouseContext::new(),
            keyboard: KeyboardContext::new(),
            gamepad: GamepadContext::new()?,
        })
    }
}
