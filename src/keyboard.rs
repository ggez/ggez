//! Keyboard utility functions.

use context::Context;
use event::KeyCode;
use event::winit_event::ModifiersState;

bitflags! {
    /// Bitflags describing state of keyboard modifiers such as ctrl or shift.
    #[derive(Default)]
    pub struct KeyMods: u8 {
        /// No modifiers; equivalent to `KeyMods::default()` and `KeyMods::empty()`.
        const NONE  = 0b00000000;
        /// Left or right Shift key.
        const SHIFT = 0b00000001;
        /// Left or right Control key.
        const CTRL  = 0b00000010;
        /// Left or right Alt key.
        const ALT   = 0b00000100;
        /// Left or right Win/Cmd/equivalent key.
        const LOGO  = 0b00001000;
    }
}

impl KeyMods {
    /// Amount of flags set (Kernighan/Wegner/Lehmer method).
    pub fn count(&self) -> u8 {
        let mut num_set = 0;
        let mut bits = self.bits();
        loop {
            if num_set >= bits {
                break;
            }
            bits &= bits - 1;
            num_set += 1;
        }
        num_set
    }
}

impl From<ModifiersState> for KeyMods {
    fn from(state: ModifiersState) -> Self {
        let mut keymod = KeyMods::empty();
        if state.shift {
            keymod |= Self::SHIFT;
        }
        if state.ctrl {
            keymod |= Self::CTRL;
        }
        if state.alt {
            keymod |= Self::ALT;
        }
        if state.logo {
            keymod |= Self::LOGO;
        }
        keymod
    }
}

/// Tracks last key pressed, to distinguish if the system
/// is sending repeat events when a key is held down.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    last_pressed: Option<KeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self { last_pressed: None }
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
/// Also sneakily updates with the queried key.
pub fn is_repeated(ctx: &mut Context, key: KeyCode) -> bool {
    let key = Some(key);
    let result = ctx.keyboard_context.last_pressed == key;
    ctx.keyboard_context.last_pressed = key;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mod_conversions() {
        assert_eq!(
            KeyMods::empty(),
            KeyMods::from(ModifiersState {
                shift: false,
                ctrl: false,
                alt: false,
                logo: false,
            })
        );
        assert_eq!(
            KeyMods::SHIFT,
            KeyMods::from(ModifiersState {
                shift: true,
                ctrl: false,
                alt: false,
                logo: false,
            })
        );
        assert_eq!(
            KeyMods::SHIFT | KeyMods::ALT,
            KeyMods::from(ModifiersState {
                shift: true,
                ctrl: false,
                alt: true,
                logo: false,
            })
        );
        assert_eq!(
            KeyMods::SHIFT | KeyMods::ALT | KeyMods::CTRL,
            KeyMods::from(ModifiersState {
                shift: true,
                ctrl: true,
                alt: true,
                logo: false,
            })
        );
        assert_eq!(
            KeyMods::SHIFT - KeyMods::ALT,
            KeyMods::from(ModifiersState {
                shift: true,
                ctrl: false,
                alt: false,
                logo: false,
            })
        );
        assert_eq!(
            (KeyMods::SHIFT | KeyMods::ALT) - KeyMods::ALT,
            KeyMods::from(ModifiersState {
                shift: true,
                ctrl: false,
                alt: false,
                logo: false,
            })
        );
        assert_eq!(
            KeyMods::SHIFT - (KeyMods::ALT | KeyMods::SHIFT),
            KeyMods::from(ModifiersState {
                shift: false,
                ctrl: false,
                alt: false,
                logo: false,
            })
        );
    }

    #[test]
    fn key_mod_set_bit_count() {
        assert_eq!(KeyMods::empty().count(), 0);
        assert_eq!(KeyMods::SHIFT.count(), 1);
        assert_eq!(KeyMods::CTRL.count(), 1);
        assert_eq!(KeyMods::ALT.count(), 1);
        assert_eq!(KeyMods::LOGO.count(), 1);
        assert_eq!((KeyMods::SHIFT | KeyMods::CTRL).count(), 2);
        assert_eq!((KeyMods::SHIFT | KeyMods::LOGO).count(), 2);
        assert_eq!((KeyMods::LOGO | KeyMods::SHIFT).count(), 2);
        assert_eq!((KeyMods::LOGO | KeyMods::SHIFT | KeyMods::ALT).count(), 3);
        assert_eq!((!KeyMods::ALT).count(), 3);
    }
}
