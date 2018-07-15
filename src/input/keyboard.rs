//! Keyboard utility functions; allow querying state of keyboard keys and modifiers.
//!
//! Example:
//!
//! ```rust, no-run
//! use ggez::event::{EventHandler, KeyCode, KeyMods};
//! use ggez::{graphics, nalgebra as na, timer};
//! use ggez::input::keyboard;
//! use ggez::{Context, GameResult};
//!
//! struct MainState {
//!     position_x: f32,
//! }
//!
//! impl EventHandler for MainState {
//!     fn update(&mut self, ctx: &mut Context) -> GameResult {
//!         // Increase or decrease `position_x` by 0.5, or by 5.0 if Shift is held.
//!         if keyboard::is_key_pressed(ctx, KeyCode::Right) {
//!             if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
//!                 self.position_x += 4.5;
//!             }
//!             self.position_x += 0.5;
//!         } else if keyboard::is_key_pressed(ctx, KeyCode::Left) {
//!             if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
//!                 self.position_x -= 4.5;
//!             }
//!             self.position_x -= 0.5;
//!         }
//!         Ok(())
//!     }
//!
//!     fn draw(&mut self, ctx: &mut Context) -> GameResult {
//!         graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
//!         // Draw a circle at `position_x`.
//!         graphics::circle(
//!             ctx,
//!             graphics::WHITE,
//!             graphics::DrawMode::Fill,
//!             na::Point2::new(self.position_x, 380.0),
//!             100.0,
//!             2.0,
//!         )?;
//!         graphics::present(ctx)?;
//!         timer::yield_now();
//!         Ok(())
//!     }
//!
//!     fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
//!         match key {
//!             // Quit if Shift+Ctrl+Q is pressed.
//!             KeyCode::Q => {
//!                 if mods.contains(KeyMods::SHIFT | KeyMods::CTRL) {
//!                     println!("Terminating!");
//!                     ctx.quit();
//!                 } else if mods.contains(KeyMods::SHIFT) || mods.contains(KeyMods::CTRL) {
//!                     println!("You need to hold both Shift and Control to quit.");
//!                 } else {
//!                     println!("Now you're not even trying!");
//!                 }
//!             }
//!             _ => (),
//!         }
//!     }
//! }
//! ```

use context::Context;

use winit::ModifiersState;
/// A key code.
pub use winit::VirtualKeyCode as KeyCode;

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
    /// A simple mapping of which key code has been pressed.
    /// KeyCode's are a c-like enum, and so can be converted to/from
    /// simple integers.
    /// As of winit 0.16 this is Big Enough For Anyone; assertions
    /// will check if that assumption gets violated.
    // Maybe we can just use a HashSet instead?  Eh.
    pressed_keys: Vec<bool>,

    // These two are necessary for tracking key-repeat.
    last_pressed: Option<KeyCode>,
    current_pressed: Option<KeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        // We reserve a fixed-size vec because we never want to have
        // to bother resizing it, but it is not easy to ask
        // Rust what an enum's max member is and a Sufficiently Big
        // fixed-size array `[bool; MAX_KEY_IDX]` doesn't implement
        // nice things like Debug.  :|
        // We have an assert everywhere pressed_keys is accessed so
        // we know if this assumption is broken.
        const MAX_KEY_IDX: usize = 256;
        let mut key_vec = Vec::with_capacity(MAX_KEY_IDX);
        key_vec.resize(MAX_KEY_IDX, false);
        Self {
            active_modifiers: KeyMods::empty(),
            pressed_keys: key_vec,
            last_pressed: None,
            current_pressed: None,
        }
    }

    // TODO: Set modifiers correctly
    // and in general cmake sure this is hooked up correctly
    // from Context::process_event().
    // Looks like it is, but, not 100% sure.
    pub(crate) fn set_key(&mut self, key: KeyCode, pressed: bool) {
        let key_idx = key as usize;
        assert!(
            key_idx < self.pressed_keys.len(),
            "Impossible KeyCode detected!"
        );
        self.pressed_keys[key_idx] = pressed;
        if pressed {
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(key);
        } else {
            self.current_pressed = None;
            // Double check that this edge handling is necessary;
            // winit sounds like it should do this for us,
            // see https://docs.rs/winit/0.16.1/winit/struct.KeyboardInput.html#structfield.modifiers
            match key {
                KeyCode::LShift | KeyCode::RShift => self.active_modifiers -= KeyMods::SHIFT,
                KeyCode::LControl | KeyCode::RControl => self.active_modifiers -= KeyMods::CTRL,
                KeyCode::LAlt | KeyCode::RAlt => self.active_modifiers -= KeyMods::ALT,
                KeyCode::LWin | KeyCode::RWin => self.active_modifiers -= KeyMods::LOGO,
                _ => (),
            }
        }
    }

    // TODO: Merge into set_key?
    pub(crate) fn set_modifiers(&mut self, keymods: KeyMods) {
        self.active_modifiers = keymods;
    }

    pub(crate) fn is_key_pressed(&self, key: KeyCode) -> bool {
        let key_idx = key as usize;
        assert!(
            key_idx < self.pressed_keys.len(),
            "Impossible KeyCode detected!"
        );
        self.pressed_keys[key_idx]
    }

    pub(crate) fn is_key_repeated(&self) -> bool {
        if let Some(_) = self.last_pressed {
            self.last_pressed == self.current_pressed
        } else {
            false
        }
    }

    pub(crate) fn get_pressed_keys(&self) -> Vec<KeyCode> {
        self.pressed_keys
            .iter()
            .enumerate()
            .filter_map(|(key_idx, b)| {
                if *b {
                    // Sigh
                    // Horrible unsafe pointer cast to turn a number
                    // into the matching KeyCode, because Rust's support
                    // for C-like numeric enums is UTTER GARBAGE.
                    // Can we protect this with an assertion somehow?
                    // I don't even see a way to get the max element of an
                    // enum.
                    let keycode: &KeyCode =
                        unsafe { &*(&key_idx as *const usize as *const KeyCode) };
                    Some(*keycode)
                } else {
                    None
                }
            })
            .collect()
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

/// Returns a Vec with currently pressed keys.
pub fn get_pressed_keys(ctx: &Context) -> Vec<KeyCode> {
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
    fn pressed_keys_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.get_pressed_keys(), &[]);
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::A]);
        assert!(keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.get_pressed_keys(), &[]);
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.get_pressed_keys(), &[KeyCode::A]);
        assert!(keyboard.is_key_pressed(KeyCode::A));
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
