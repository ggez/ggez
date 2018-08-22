//! Gamepad utility functions.
//!
//! This is going to be a bit of a work-in-progress as gamepad input
//! gets fleshed out.  The `gilrs` crate needs help to add better
//! cross-platform support.  Why not give it a hand?

use std::fmt;

use gilrs::{Gamepad, Gilrs};

use context::Context;
use GameResult;

/// A structure that contains gamepad state.
pub struct GamepadContext {
    pub(crate) gilrs: Gilrs,
}

impl fmt::Debug for GamepadContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<GamepadContext: {:p}>", self)
    }
}

impl GamepadContext {
    pub(crate) fn new() -> GameResult<GamepadContext> {
        let gilrs = Gilrs::new()?;
        Ok(GamepadContext { gilrs })
    }
}

/// returns the `Gamepad` associated with an id.
pub fn gamepad(ctx: &Context, id: usize) -> Option<&Gamepad> {
    ctx.gamepad_context.gilrs.get(id)
}

// Properties gamepads might want:
// Number of buttons
// Number of axes
// Name/ID
// Is it connected?  (For consoles?)
// Whether or not they support vibration

/// Lists all gamepads.  With metainfo, maybe?
pub fn list_gamepads() {
    unimplemented!()
}

/// Returns the state of the given axis on a gamepad.
pub fn axis() {
    unimplemented!()
}

/// Returns the state of the given button on a gamepad.
pub fn button_pressed() {
    unimplemented!()
}
