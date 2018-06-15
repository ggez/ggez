//! Keyboard utility functions.

use context::Context;
use event::winit_event::ModifiersState;
use event::KeyCode;
use std::collections::VecDeque;

bitflags! {
    /// Bitflags describing state of keyboard modifiers, such as Control or Shift.
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
    pub fn count(&self) -> usize {
        let mut num_set = 0;
        let mut bits = self.bits();
        loop {
            if num_set >= bits {
                break;
            }
            bits &= bits - 1;
            num_set += 1;
        }
        num_set as usize
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

/// Tracks held down keyboard keys, active keyboard modifiers,
/// and figures out if the system is sending repeat keystrokes.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    active_modifiers: KeyMods,
    pressed_keys: Vec<KeyCode>,
    last_pressed: VecDeque<KeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            active_modifiers: KeyMods::empty(),
            pressed_keys: Vec::with_capacity(16),
            last_pressed: VecDeque::with_capacity(3),
        }
    }

    pub(crate) fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            if !self.pressed_keys.contains(&key) {
                self.pressed_keys.push(key);
            }

            self.last_pressed.push_back(key);
            if self.last_pressed.len() > 2 {
                self.last_pressed.pop_front();
            }
        } else {
            if let Some(i) = self
                .pressed_keys
                .iter()
                .enumerate()
                .find(|(_i, pressed_key)| **pressed_key == key)
                .map(|(i, _)| i)
            {
                self.pressed_keys.swap_remove(i);
            }

            self.last_pressed.clear();
        }
    }

    pub(crate) fn set_modifiers(&mut self, keymods: KeyMods) {
        self.active_modifiers = keymods;
    }

    pub(crate) fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub(crate) fn is_key_repeated(&self) -> bool {
        if let Some(key1) = self.last_pressed.get(0) {
            if let Some(key2) = self.last_pressed.get(1) {
                return key1 == key2;
            }
        }
        false
    }

    pub(crate) fn get_pressed_keys(&self) -> &[KeyCode] {
        &self.pressed_keys
    }

    pub(crate) fn get_active_mods(&self) -> KeyMods {
        self.active_modifiers
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a key is currently pressed down.
pub fn is_key_pressed(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard_context.is_key_pressed(key)
}

/// Checks if the last keystroke sent by the system is repeated,
/// like when a key is held down for a period of time.
pub fn is_key_repeated(ctx: &Context) -> bool {
    ctx.keyboard_context.is_key_repeated()
}

/// Returns a slice with currently held keys.
pub fn get_pressed_keys(ctx: &Context) -> &[KeyCode] {
    ctx.keyboard_context.get_pressed_keys()
}

/// Checks if keyboard modifier (or several) is active.
pub fn is_mod_active(ctx: &Context, keymods: KeyMods) -> bool {
    ctx.keyboard_context.get_active_mods().contains(keymods)
}

/// Returns currently active keyboard modifiers.
pub fn get_active_mods(ctx: &Context) -> KeyMods {
    ctx.keyboard_context.get_active_mods()
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

    #[test]
    fn pressed_keys_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.get_pressed_keys(), &[]);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::A]);
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.get_pressed_keys(), &[]);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::A]);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::A]);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::A, KeyCode::B]);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::A, KeyCode::B]);
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::B]);
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::B]);
        keyboard.set_key(KeyCode::B, false);
        assert_eq!(keyboard.get_pressed_keys(), &[]);
    }

    #[test]
    fn repeated_keys_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), true);
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), true);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.is_key_repeated(), true);
    }
}
