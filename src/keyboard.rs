//! Keyboard utility functions.

use context::Context;
use event::KeyCode;

/// Tracks last key pressed, to distinguish if the system
/// is sending repeat events when a key is held down.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    last_pressed: Option<KeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            last_pressed: None,
        }
    }

    pub(crate) fn set_last_pressed(&mut self, key: Option<KeyCode>) {
        self.last_pressed = key;
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the last key held down.
pub fn get_last_held(ctx: &Context) -> Option<KeyCode> {
    ctx.keyboard_context.last_pressed
}

/// Checks if the system is sending repeat events of the keystroke,
/// like when a key is held down.
pub fn is_repeated(ctx: &mut Context, key: KeyCode) -> bool {
    let key = Some(key);
    let result = ctx.keyboard_context.last_pressed == key;
    ctx.keyboard_context.last_pressed = key;
    result
}
