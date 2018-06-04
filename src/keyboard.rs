//! Keyboard utility functions.

use context::Context;
use error::GameResult;
use event::Keycode;
use graphics;

/// Tracks last key pressed, to distinguish if the system
/// is sending repeat events when a key is held down.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    last_pressed: Option<Keycode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            last_pressed: None,
        }
    }

    pub(crate) fn set_last_pressed(&mut self, key: Keycode) {
        self.last_pressed = key;
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the last key held down.
pub fn get_last_held(ctx: &Context) -> Option<Keycode> {
    ctx.keyboard_context.last_pressed
}

/// Checks if the system is sending repeat events of the keystroke,
/// like when a key is held down.
pub fn is_repeated(ctx: &Context, key: Keycode) -> bool {
    let key = Some(key);
    let result = ctx.keyboard_context.last_pressed == key;
    ctx.keyboard_context.last_pressed = key;
    result
}
