//! Keyboard utility functions; allow querying state of keyboard keys and modifiers.
//!
//! Example:
//!
//! ```rust, compile
//! use ggez::event::{self, EventHandler, KeyCode, KeyMods};
//! use ggez::{graphics, timer};
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
//!         // Create a circle at `position_x` and draw
//!         let circle = graphics::Mesh::new_circle(
//!             ctx,
//!             graphics::DrawMode::fill(),
//!             glam::vec2(self.position_x, 380.0),
//!             100.0,
//!             2.0,
//!             graphics::Color::WHITE,
//!         )?;
//!         graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
//!         graphics::present(ctx)?;
//!         timer::yield_now();
//!         Ok(())
//!     }
//!
//!     fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
//!         match key {
//!             // Quit if Shift+Ctrl+Q is pressed.
//!             KeyCode::Q => {
//!                 if mods.contains(KeyMods::SHIFT) && mods.contains(KeyMods::CTRL) {
//!                     println!("Terminating!");
//!                     event::quit(ctx);
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

use crate::context::Context;

use std::collections::HashSet;
use winit::event::ModifiersState;
pub use winit::event::ScanCode;
/// A key code.
pub use winit::event::VirtualKeyCode as KeyCode;

bitflags! {
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
    /// The scancode.
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

    // These two are necessary for tracking key-repeat.
    last_pressed: Option<KeyCode>,
    current_pressed: Option<KeyCode>,

    // Represents the state of pressed_keys_set last frame.
    previously_pressed_set: HashSet<KeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            active_modifiers: KeyMods::empty(),
            // We just use 256 as a number Big Enough For Keyboard Keys to try to avoid resizing.
            pressed_keys_set: HashSet::with_capacity(256),
            last_pressed: None,
            current_pressed: None,
            previously_pressed_set: HashSet::with_capacity(256),
        }
    }

    /// Checks if a key is currently pressed down.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys_set.contains(&key)
    }

    /// Checks if a key has been pressed down this frame.
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys_set.contains(&key) && !self.previously_pressed_set.contains(&key)
    }

    /// Checks if a key has been released this frame.
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        !self.pressed_keys_set.contains(&key) && self.previously_pressed_set.contains(&key)
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
        self.previously_pressed_set = self.pressed_keys_set.clone();
    }

    pub(crate) fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            let _ = self.pressed_keys_set.insert(key);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(key);
        } else {
            let _ = self.pressed_keys_set.remove(&key);
            self.current_pressed = None;
        }

        self.set_key_modifier(key, pressed);
    }

    pub(crate) fn set_modifiers(&mut self, keymods: KeyMods) {
        self.active_modifiers = keymods;
    }

    /// Take a modifier key code and alter our state.
    ///
    /// Double check that this edge handling is necessary;
    /// winit sounds like it should do this for us,
    /// see https://docs.rs/winit/0.18.0/winit/struct.KeyboardInput.html#structfield.modifiers
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
// TODO: Add deprecation version
#[deprecated(note = "Use `ctx.keyboard.is_key_pressed` instead")]
pub fn is_key_pressed(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard.is_key_pressed(key)
}

/// Checks if a key has been pressed down this frame.
// TODO: Add deprecation version
#[deprecated(note = "Use `ctx.keyboard.is_key_just_pressed` instead")]
pub fn is_key_just_pressed(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard.is_key_just_pressed(key)
}

/// Checks if a key has been released this frame.
// TODO: Add deprecation version
#[deprecated(note = "Use `ctx.keyboard.is_key_just_released` instead")]
pub fn is_key_just_released(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard.is_key_just_released(key)
}

/// Checks if the last keystroke sent by the system is repeated,
/// like when a key is held down for a period of time.
// TODO: Add deprecation version
#[deprecated(note = "Use `ctx.keyboard.is_key_repeated` instead")]
pub fn is_key_repeated(ctx: &Context) -> bool {
    ctx.keyboard.is_key_repeated()
}

/// Returns a reference to the set of currently pressed keys.
// TODO: Add deprecation version
#[deprecated(note = "Use `ctx.keyboard.pressed_keys` instead")]
pub fn pressed_keys(ctx: &Context) -> &HashSet<KeyCode> {
    ctx.keyboard.pressed_keys()
}

/// Checks if keyboard modifier (or several) is active.
// TODO: Add deprecation version
#[deprecated(note = "Use `ctx.keyboard.is_mod_active` instead")]
pub fn is_mod_active(ctx: &Context, keymods: KeyMods) -> bool {
    ctx.keyboard.is_mod_active(keymods)
}

/// Returns currently active keyboard modifiers.
// TODO: Add deprecation version
#[deprecated(note = "Use `ctx.keyboard.active_mods` instead")]
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
        assert_eq!(keyboard.pressed_keys(), &[].iter().cloned().collect());
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().cloned().collect()
        );
        assert!(keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.pressed_keys(), &[].iter().cloned().collect());
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().cloned().collect()
        );
        assert!(keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A, KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A, KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::B, false);
        assert_eq!(keyboard.pressed_keys(), &[].iter().cloned().collect());
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
        keyboard.set_key(KeyCode::A, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::A, false);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::A, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::A, true);
        assert!(keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::A, false);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::A, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::B, true);
        assert!(!keyboard.is_key_repeated(),);
        keyboard.set_key(KeyCode::A, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::A, true);
        assert!(keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::B, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_key(KeyCode::B, true);
        assert!(keyboard.is_key_repeated());
    }
}
