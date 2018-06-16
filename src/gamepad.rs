// TODO: Documentation
//! Gamepad utility functions.

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
pub fn get_gamepad(ctx: &Context, id: usize) -> Option<&Gamepad> {
    ctx.gamepad_context.gilrs.get(id)
}
