//! Gamepad utility functions.
//!
//! This is going to be a bit of a work-in-progress as gamepad input
//! gets fleshed out.  The `gilrs` crate needs help to add better
//! cross-platform support.  Why not give it a hand?
//!
//! TODO: All of this.

use std::fmt;

#[cfg(feature = "gilrs")]
use gilrs::{Event, Gamepad, Gilrs};

use crate::context::Context;
use crate::error::GameResult;

/// A type that contains a reference to the underlying implementation's
/// gamepad interface.
#[derive(Clone, Debug)]
pub enum ConcreteGamepad<'a> {
    #[cfg(feature = "gilrs")]
    /// A gamepad interface from the `gilrs` crate
    GilrsGamepad(&'a Gamepad),
    /// A dummy gamepad interface
    NullGamepad(&'a ())
}

/// Trait object defining a gamepad/joystick context.
pub trait GamepadContext {
    /// Returns a gamepad event.
    fn next_event(&mut self) -> Option<Event>;

    /// returns the `Gamepad` associated with an id.
    fn gamepad(&self, id: usize) -> Option<ConcreteGamepad>;
}

#[cfg(feature = "gilrs")]
/// A structure that contains gamepad state using `gilrs`.
pub struct GilrsGamepadContext {
    pub(crate) gilrs: Gilrs,
}

#[cfg(feature = "gilrs")]
impl fmt::Debug for GilrsGamepadContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<GilrsGamepadContext: {:p}>", self)
    }
}
#[cfg(feature = "gilrs")]
impl GilrsGamepadContext {
    pub(crate) fn new() -> GameResult<Self> {
        let gilrs = Gilrs::new()?;
        Ok(GilrsGamepadContext { gilrs })
    }
}

#[cfg(feature = "gilrs")]
impl GamepadContext for GilrsGamepadContext {
    fn next_event(&mut self) -> Option<Event> {
        self.gilrs.next_event()
    }

    fn gamepad(&self, id: usize) -> Option<ConcreteGamepad> {
        self.gilrs.get(id)
            .map(|gp| ConcreteGamepad::GilrsGamepad(gp))
    }
}

/// A structure that implements [`GamepadContext`](trait.GamepadContext.html)
/// but does nothing; a stub for when you don't need it or are
/// on a platform that `gilrs` doesn't support.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullGamepadContext {}

impl GamepadContext for NullGamepadContext {
    fn next_event(&mut self) -> Option<Event> {
        panic!("Gamepad module disabled")
    }

    fn gamepad(&self, _id: usize) -> Option<ConcreteGamepad> {
        None
    }
}

/// Returns the `Gamepad` associated with an `id`.
pub fn gamepad(ctx: &Context, id: usize) -> Option<ConcreteGamepad> {
    ctx.gamepad_context.gamepad(id)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gilrs_init() {
        assert!(GilrsGamepadContext::new().is_ok());
    }
}
