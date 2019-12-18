//! Gamepad utility functions.
//!
//! This is going to be a bit of a work-in-progress as gamepad input
//! gets fleshed out.  The `gilrs` crate needs help to add better
//! cross-platform support.  Why not give it a hand?
use gilrs::ConnectedGamepadsIterator;
use std::fmt;

pub use gilrs::{self, Event, Gamepad, Gilrs};

/// A unique identifier for a particular GamePad
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GamepadId(pub(crate) gilrs::GamepadId);

use crate::context::Context;
use crate::error::GameResult;

/// Trait object defining a gamepad/joystick context.
pub trait GamepadContext {
    /// Returns a gamepad event.
    fn next_event(&mut self) -> Option<Event>;

    /// returns the `Gamepad` associated with an id.
    fn gamepad(&self, id: GamepadId) -> Gamepad;

    /// returns an iterator over the connected `Gamepad`s.
    fn gamepads(&self) -> GamepadsIterator;
}

/// A structure that contains gamepad state using `gilrs`.
pub struct GilrsGamepadContext {
    pub(crate) gilrs: Gilrs,
}

impl fmt::Debug for GilrsGamepadContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<GilrsGamepadContext: {:p}>", self)
    }
}

impl GilrsGamepadContext {
    pub(crate) fn new() -> GameResult<Self> {
        let gilrs = Gilrs::new()?;
        Ok(GilrsGamepadContext { gilrs })
    }
}

impl GamepadContext for GilrsGamepadContext {
    fn next_event(&mut self) -> Option<Event> {
        self.gilrs.next_event()
    }

    fn gamepad(&self, id: GamepadId) -> Gamepad {
        self.gilrs.gamepad(id.0)
    }

    fn gamepads(&self) -> GamepadsIterator {
        GamepadsIterator {
            wrapped: self.gilrs.gamepads(),
        }
    }
}

/// An iterator of the connected gamepads
pub struct GamepadsIterator<'a> {
    wrapped: ConnectedGamepadsIterator<'a>,
}

impl<'a> fmt::Debug for GamepadsIterator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<GamepadsIterator: {:p}>", self)
    }
}

impl<'a> Iterator for GamepadsIterator<'a> {
    type Item = (GamepadId, Gamepad<'a>);

    fn next(&mut self) -> Option<(GamepadId, Gamepad<'a>)> {
        match self.wrapped.next() {
            Some((id, gp)) => Some((GamepadId(id), gp)),
            None => None,
        }
    }
}

/// A structure that implements [`GamepadContext`](trait.GamepadContext.html)
/// but does nothing; a stub for when you don't need it or are
/// on a platform that `gilrs` doesn't support.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NullGamepadContext {}

impl GamepadContext for NullGamepadContext {
    fn next_event(&mut self) -> Option<Event> {
        panic!("Gamepad module disabled")
    }

    fn gamepad(&self, _id: GamepadId) -> Gamepad {
        panic!("Gamepad module disabled")
    }

    fn gamepads(&self) -> GamepadsIterator {
        panic!("Gamepad module disabled")
    }
}

/// Returns the `Gamepad` associated with an `id`.
pub fn gamepad(ctx: &Context, id: GamepadId) -> Gamepad {
    ctx.gamepad_context.gamepad(id)
}

/// Return an iterator of all the `Gamepads` that are connected.
pub fn gamepads(ctx: &Context) -> GamepadsIterator {
    ctx.gamepad_context.gamepads()
}

// Properties gamepads might want:
// Number of buttons
// Number of axes
// Name/ID
// Is it connected?  (For consoles?)
// Whether or not they support vibration

/*
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
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gilrs_init() {
        assert!(GilrsGamepadContext::new().is_ok());
    }
}
