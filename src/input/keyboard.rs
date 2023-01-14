//! Keyboard utility functions; allow querying state of keyboard keys and modifiers.
//!
//! # On Keycodes and Scancodes
//!
//! You can see functions for keys and functions for scancodes listed here.
//!
//! Keycodes are the "meaning" of a key once keyboard layout translation
//! has been applied. For example, when the user presses "Q" in their
//! layout, the enum value for Q is provided to this function.
//!
//! Scancodes are hardware dependent names for keys that refer to the key's
//! location rather than the character it prints when pressed. They are not
//! necessarily cross platform (e.g. between Windows and Linux).
//!
//! For example, on a US QWERTY keyboard layout, the WASD keys are located
//! in an inverted T shape on the left of the keyboard. This is not the
//! case for AZERTY keyboards, which have those keys in a different
//! location. Using scan codes over key codes in this case would map those
//! characters to their physical location on the keyboard.
//!
//! In general, keycodes should be used when the meaning of the typed
//! character is important (e.g. "I" to open the inventory), and scancodes
//! for when the location is important (e.g. the WASD key block). The
//! `text_input_event` handler should be used to collect raw text.
//!
//! The keycode is optional because not all inputs can be matched to a
//! specific key code. This will happen on non-English keyboards, for
//! example.
//!
//! -----
//!
//! Example:
//!
//! ```rust, compile
//! use ggez::event::{self, EventHandler};
//! use ggez::input::keyboard::{KeyCode, KeyMods, KeyInput};
//! use ggez::{graphics::{self, Color}, timer};
//! use ggez::{Context, GameResult};
//!
//! struct MainState {
//!     position_x: f32,
//! }
//!
//! impl EventHandler for MainState {
//!     fn update(&mut self, ctx: &mut Context) -> GameResult {
//!         let k_ctx = &ctx.keyboard;
//!         // Increase or decrease `position_x` by 0.5, or by 5.0 if Shift is held.
//!         if k_ctx.is_key_pressed(KeyCode::Right) {
//!             if k_ctx.is_mod_active(KeyMods::SHIFT) {
//!                 self.position_x += 4.5;
//!             }
//!             self.position_x += 0.5;
//!         } else if k_ctx.is_key_pressed(KeyCode::Left) {
//!             if k_ctx.is_mod_active(KeyMods::SHIFT) {
//!                 self.position_x -= 4.5;
//!             }
//!             self.position_x -= 0.5;
//!         }
//!         Ok(())
//!     }
//!
//!     fn draw(&mut self, ctx: &mut Context) -> GameResult {
//!         let mut canvas = graphics::Canvas::from_frame(
//!             ctx,
//!             Color::from([0.1, 0.2, 0.3, 1.0]),
//!         );
//!         // Create a circle at `position_x` and draw
//!         let circle = graphics::Mesh::new_circle(
//!             ctx,
//!             graphics::DrawMode::fill(),
//!             glam::vec2(self.position_x, 380.0),
//!             100.0,
//!             2.0,
//!             graphics::Color::WHITE,
//!         )?;
//!         canvas.draw(&circle, graphics::DrawParam::default());
//!         canvas.finish(ctx)?;
//!         timer::yield_now();
//!         Ok(())
//!     }
//!
//!     fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
//!         match input.keycode {
//!             // Quit if Shift+Ctrl+Q is pressed.
//!             Some(KeyCode::Q) => {
//!                 if input.mods.contains(KeyMods::SHIFT) && input.mods.contains(KeyMods::CTRL) {
//!                     println!("Terminating!");
//!                     ctx.request_quit();
//!                 } else if input.mods.contains(KeyMods::SHIFT) || input.mods.contains(KeyMods::CTRL) {
//!                     println!("You need to hold both Shift and Control to quit.");
//!                 } else {
//!                     println!("Now you're not even trying!");
//!                 }
//!             }
//!             _ => (),
//!         }
//!         Ok(())
//!     }
//! }
//!
//! pub fn main() -> GameResult {
//!     let cb = ggez::ContextBuilder::new("keyboard", "ggez");
//!     let (mut ctx, event_loop) = cb.build()?;
//!
//!     let state = MainState { position_x: 0.0 };
//!     event::run(ctx, event_loop, state)
//! }
//! ```

use crate::context::Context;

use std::collections::HashSet;
use winit::event::ModifiersState;
pub use winit::event::ScanCode;
/// A key code.
pub use winit::event::VirtualKeyCode as KeyCode;

bitflags::bitflags! {
    /// Bitflags describing the state of keyboard modifiers, such as `Control` or `Shift`.
    #[derive(Default)]
    pub struct KeyMods: u8 {
        /// No modifiers; equivalent to `KeyMods::default()` and
        /// [`KeyMods::empty()`](struct.KeyMods.html#method.empty).
        const NONE  = 0b0000_0000;
        /// Left or right Shift key.
        const SHIFT = 0b0000_0001;
        /// Left or right Control key.
        const CTRL  = 0b0000_0010;
        /// Left or right Alt key.
        const ALT   = 0b0000_0100;
        /// Left or right Win/Cmd/equivalent key.
        const LOGO  = 0b0000_1000;
    }
}

impl From<ModifiersState> for KeyMods {
    fn from(state: ModifiersState) -> Self {
        let mut keymod = KeyMods::empty();
        if state.shift() {
            keymod |= Self::SHIFT;
        }
        if state.ctrl() {
            keymod |= Self::CTRL;
        }
        if state.alt() {
            keymod |= Self::ALT;
        }
        if state.logo() {
            keymod |= Self::LOGO;
        }
        keymod
    }
}

/// A simple wrapper bundling the four properties of a keyboard stroke.
#[derive(Copy, Clone, Debug)]
pub struct KeyInput {
    /// The scancode. For more info on what they are and when to use them refer to the
    /// [`keyboard`](crate::input::keyboard) module.
    pub scancode: ScanCode,
    /// The keycode corresponding to the scancode, if there is one.
    pub keycode: Option<KeyCode>,
    /// The keyboard modifiers active at the moment of input.
    pub mods: KeyMods,
}

/// Tracks held down keyboard keys, active keyboard modifiers,
/// and figures out if the system is sending repeat keystrokes.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    active_modifiers: KeyMods,
    /// A simple mapping of which key code has been pressed.
    /// We COULD use a `Vec<bool>` but turning Rust enums to and from
    /// integers is unsafe and a set really is what we want anyway.
    pressed_keys_set: HashSet<KeyCode>,
    pressed_scancodes_set: HashSet<ScanCode>,

    // These two are necessary for tracking key-repeat.
    last_pressed: Option<ScanCode>,
    current_pressed: Option<ScanCode>,

    // Represents the state of pressed_keys_set last frame.
    previously_pressed_keys_set: HashSet<KeyCode>,
    previously_pressed_scancodes_set: HashSet<ScanCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            active_modifiers: KeyMods::empty(),
            // We just use 256 as a number Big Enough For Keyboard Keys to try to avoid resizing.
            pressed_keys_set: HashSet::with_capacity(256),
            pressed_scancodes_set: HashSet::with_capacity(256),
            last_pressed: None,
            current_pressed: None,
            previously_pressed_keys_set: HashSet::with_capacity(256),
            previously_pressed_scancodes_set: HashSet::with_capacity(256),
        }
    }

    /// Checks if a key is currently pressed down.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys_set.contains(&key)
    }

    /// Checks if a key has been pressed down this frame.
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys_set.contains(&key) && !self.previously_pressed_keys_set.contains(&key)
    }

    /// Checks if a key has been released this frame.
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        !self.pressed_keys_set.contains(&key) && self.previously_pressed_keys_set.contains(&key)
    }

    /// Checks if a key with the corresponding scan code is currently pressed down.
    pub fn is_scancode_pressed(&self, code: ScanCode) -> bool {
        self.pressed_scancodes_set.contains(&code)
    }

    /// Checks if a key with the corresponding scan code has been pressed down this frame.
    pub fn is_scancode_just_pressed(&self, code: ScanCode) -> bool {
        self.pressed_scancodes_set.contains(&code)
            && !self.previously_pressed_scancodes_set.contains(&code)
    }

    /// Checks if a key with the corresponding scan code has been released this frame.
    pub fn is_scancode_just_released(&self, code: ScanCode) -> bool {
        !self.pressed_scancodes_set.contains(&code)
            && self.previously_pressed_scancodes_set.contains(&code)
    }

    /// Checks if the last keystroke sent by the system is repeated,
    /// like when a key is held down for a period of time.
    pub fn is_key_repeated(&self) -> bool {
        if self.last_pressed.is_some() {
            self.last_pressed == self.current_pressed
        } else {
            false
        }
    }

    /// Returns a reference to the set of currently pressed keys.
    pub fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys_set
    }

    /// Returns a reference to the set of currently pressed scancodes.
    pub fn pressed_scancodes(&self) -> &HashSet<ScanCode> {
        &self.pressed_scancodes_set
    }

    /// Checks if keyboard modifier (or several) is active.
    pub fn is_mod_active(&self, keymods: KeyMods) -> bool {
        self.active_mods().contains(keymods)
    }

    /// Returns currently active keyboard modifiers.
    pub fn active_mods(&self) -> KeyMods {
        self.active_modifiers
    }

    /// Copies the current state of the keyboard into the context. If you are writing your own event loop
    /// you need to call this at the end of every update in order to use the functions `is_key_just_pressed`
    /// and `is_key_just_released`. Otherwise this is handled for you.
    pub fn save_keyboard_state(&mut self) {
        self.previously_pressed_keys_set = self.pressed_keys_set.clone();
        self.previously_pressed_scancodes_set = self.pressed_scancodes_set.clone();
    }

    pub(crate) fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            let _ = self.pressed_keys_set.insert(key);
        } else {
            let _ = self.pressed_keys_set.remove(&key);
        }

        self.set_key_modifier(key, pressed);
    }

    pub(crate) fn set_scancode(&mut self, code: ScanCode, pressed: bool) {
        if pressed {
            let _ = self.pressed_scancodes_set.insert(code);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(code);
        } else {
            let _ = self.pressed_scancodes_set.remove(&code);
            self.current_pressed = None;
        }
    }

    /// Set the keyboard active modifiers
    /// Really useful only if you are writing your own event loop
    pub fn set_modifiers(&mut self, keymods: KeyMods) {
        self.active_modifiers = keymods;
    }

    /// Take a modifier key code and alter our state.
    ///
    /// Double check that this edge handling is necessary;
    /// winit sounds like it should do this for us,
    /// see <https://docs.rs/winit/0.18.0/winit/struct.KeyboardInput.html#structfield.modifiers>
    ///
    /// ...more specifically, we should refactor all this to consistant-ify events a bit and
    /// make winit do more of the work.
    /// But to quote Scott Pilgrim, "This is... this is... Booooooring."
    fn set_key_modifier(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            match key {
                KeyCode::LShift | KeyCode::RShift => self.active_modifiers |= KeyMods::SHIFT,
                KeyCode::LControl | KeyCode::RControl => self.active_modifiers |= KeyMods::CTRL,
                KeyCode::LAlt | KeyCode::RAlt => self.active_modifiers |= KeyMods::ALT,
                KeyCode::LWin | KeyCode::RWin => self.active_modifiers |= KeyMods::LOGO,
                _ => (),
            }
        } else {
            match key {
                KeyCode::LShift | KeyCode::RShift => self.active_modifiers -= KeyMods::SHIFT,
                KeyCode::LControl | KeyCode::RControl => self.active_modifiers -= KeyMods::CTRL,
                KeyCode::LAlt | KeyCode::RAlt => self.active_modifiers -= KeyMods::ALT,
                KeyCode::LWin | KeyCode::RWin => self.active_modifiers -= KeyMods::LOGO,
                _ => (),
            }
        }
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a key is currently pressed down.
#[deprecated(since = "0.8.0", note = "Use `ctx.keyboard.is_key_pressed` instead")]
pub fn is_key_pressed(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard.is_key_pressed(key)
}

/// Checks if a key has been pressed down this frame.
#[deprecated(
    since = "0.8.0",
    note = "Use `ctx.keyboard.is_key_just_pressed` instead"
)]
pub fn is_key_just_pressed(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard.is_key_just_pressed(key)
}

/// Checks if a key has been released this frame.
#[deprecated(
    since = "0.8.0",
    note = "Use `ctx.keyboard.is_key_just_released` instead"
)]
pub fn is_key_just_released(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard.is_key_just_released(key)
}

/// Checks if the last keystroke sent by the system is repeated,
/// like when a key is held down for a period of time.
#[deprecated(since = "0.8.0", note = "Use `ctx.keyboard.is_key_repeated` instead")]
pub fn is_key_repeated(ctx: &Context) -> bool {
    ctx.keyboard.is_key_repeated()
}

/// Returns a reference to the set of currently pressed keys.
#[deprecated(since = "0.8.0", note = "Use `ctx.keyboard.pressed_keys` instead")]
pub fn pressed_keys(ctx: &Context) -> &HashSet<KeyCode> {
    ctx.keyboard.pressed_keys()
}

/// Checks if keyboard modifier (or several) is active.
#[deprecated(since = "0.8.0", note = "Use `ctx.keyboard.is_mod_active` instead")]
pub fn is_mod_active(ctx: &Context, keymods: KeyMods) -> bool {
    ctx.keyboard.is_mod_active(keymods)
}

/// Returns currently active keyboard modifiers.
#[deprecated(since = "0.8.0", note = "Use `ctx.keyboard.active_mods` instead")]
pub fn active_mods(ctx: &Context) -> KeyMods {
    ctx.keyboard.active_mods()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mod_conversions() {
        let shift = winit::event::ModifiersState::SHIFT;
        let alt = winit::event::ModifiersState::ALT;
        let ctrl = winit::event::ModifiersState::CTRL;

        assert_eq!(KeyMods::empty(), KeyMods::from(ModifiersState::empty()));
        assert_eq!(KeyMods::SHIFT, KeyMods::from(shift));
        assert_eq!(KeyMods::SHIFT | KeyMods::ALT, KeyMods::from(shift | alt));
        assert_eq!(
            KeyMods::SHIFT | KeyMods::ALT | KeyMods::CTRL,
            KeyMods::from(shift | alt | ctrl)
        );
        assert_eq!(KeyMods::SHIFT - KeyMods::ALT, KeyMods::from(shift));
        assert_eq!(
            (KeyMods::SHIFT | KeyMods::ALT) - KeyMods::ALT,
            KeyMods::from(shift)
        );
        assert_eq!(
            KeyMods::SHIFT - (KeyMods::ALT | KeyMods::SHIFT),
            KeyMods::from(ModifiersState::empty())
        );
    }

    #[test]
    fn pressed_keys_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.pressed_keys(), &[].iter().copied().collect());
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().copied().collect()
        );
        assert!(keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.pressed_keys(), &[].iter().copied().collect());
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().copied().collect()
        );
        assert!(keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().copied().collect()
        );
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A, KeyCode::B].iter().copied().collect()
        );
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A, KeyCode::B].iter().copied().collect()
        );
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::B].iter().copied().collect()
        );
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::B].iter().copied().collect()
        );
        keyboard.set_key(KeyCode::B, false);
        assert_eq!(keyboard.pressed_keys(), &[].iter().copied().collect());
    }

    #[test]
    fn pressed_scancodes_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.pressed_scancodes(), &[].iter().copied().collect());
        assert!(!keyboard.is_scancode_pressed(3));
        keyboard.set_scancode(3, true);
        assert_eq!(keyboard.pressed_scancodes(), &[3].iter().copied().collect());
        assert!(keyboard.is_scancode_pressed(3));
        keyboard.set_scancode(3, false);
        assert_eq!(keyboard.pressed_scancodes(), &[].iter().copied().collect());
        assert!(!keyboard.is_scancode_pressed(3));
        keyboard.set_scancode(3, true);
        assert_eq!(keyboard.pressed_scancodes(), &[3].iter().copied().collect());
        assert!(keyboard.is_scancode_pressed(3));
        keyboard.set_scancode(3, true);
        assert_eq!(keyboard.pressed_scancodes(), &[3].iter().copied().collect());
        keyboard.set_scancode(4, true);
        assert_eq!(
            keyboard.pressed_scancodes(),
            &[3, 4].iter().copied().collect()
        );
        keyboard.set_scancode(4, true);
        assert_eq!(
            keyboard.pressed_scancodes(),
            &[3, 4].iter().copied().collect()
        );
        keyboard.set_scancode(3, false);
        assert_eq!(keyboard.pressed_scancodes(), &[4].iter().copied().collect());
        keyboard.set_scancode(3, false);
        assert_eq!(keyboard.pressed_scancodes(), &[4].iter().copied().collect());
        keyboard.set_scancode(4, false);
        assert_eq!(keyboard.pressed_scancodes(), &[].iter().copied().collect());
    }

    #[test]
    fn keyboard_modifiers() {
        let mut keyboard = KeyboardContext::new();

        // this test is mostly useless and is primarily for code coverage
        assert_eq!(keyboard.active_mods(), KeyMods::default());
        keyboard.set_modifiers(KeyMods::from(ModifiersState::all()));

        // these test the workaround for https://github.com/tomaka/winit/issues/600
        assert_eq!(
            keyboard.active_mods(),
            KeyMods::SHIFT | KeyMods::CTRL | KeyMods::ALT | KeyMods::LOGO
        );
        keyboard.set_key(KeyCode::LControl, false);
        assert_eq!(
            keyboard.active_mods(),
            KeyMods::SHIFT | KeyMods::ALT | KeyMods::LOGO
        );
        keyboard.set_key(KeyCode::RAlt, false);
        assert_eq!(keyboard.active_mods(), KeyMods::SHIFT | KeyMods::LOGO);
        keyboard.set_key(KeyCode::LWin, false);
        assert_eq!(keyboard.active_mods(), KeyMods::SHIFT);
    }

    #[test]
    fn repeated_keys_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(1, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(1, false);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(1, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(1, true);
        assert!(keyboard.is_key_repeated());
        keyboard.set_scancode(1, false);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(1, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(2, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(1, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(1, true);
        assert!(keyboard.is_key_repeated());
        keyboard.set_scancode(2, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_scancode(2, true);
        assert!(keyboard.is_key_repeated());
    }
}
