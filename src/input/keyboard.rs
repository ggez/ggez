//! Keyboard utility functions; allow querying state of keyboard keys and modifiers.
//!
//! # On logical keys and physical keys
//!
//! Logical keys are the "meaning" of a key once keyboard layout translation
//! has been applied. For example, when the user presses "Q" in their
//! layout, the enum value for Q is provided to this function.
//!
//! Physical keys are hardware dependent names for keys that refer to the key's
//! location rather than the character it prints when pressed. They are not
//! necessarily cross platform (e.g. between Windows and Linux).
//!
//! For example, on a US QWERTY keyboard layout, the WASD keys are located
//! in an inverted T shape on the left of the keyboard. This is not the
//! case for AZERTY keyboards, which have those keys in a different
//! location. Using physical keys in this case would map those
//! characters to their physical location on the keyboard.
//!
//! In general, logical keys should be used when the meaning of the typed
//! character is important (e.g. "I" to open the inventory), and physical keys
//! for when the location is important (e.g. the WASD key block).
//!
//! -----
//!
//! Example:
//!
//! ```rust, compile
//! use winit::keyboard::{Key, ModifiersState, NamedKey};
//! use ggez::event::{self, EventHandler};
//! use ggez::input::keyboard::KeyInput;
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
//!         if k_ctx.is_logical_key_pressed(&Key::Named(NamedKey::ArrowRight)) {
//!             if k_ctx.active_modifiers.contains(ModifiersState::SHIFT) {
//!                 self.position_x += 4.5;
//!             }
//!             self.position_x += 0.5;
//!         } else if k_ctx.is_logical_key_pressed(&Key::Named(NamedKey::ArrowLeft)) {
//!             if k_ctx.active_modifiers.contains(ModifiersState::SHIFT) {
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
//!         match input.event.logical_key {
//!             // Quit if Shift+Ctrl+Q is pressed.
//!             Key::Character(c) => {
//!                 if c == "q" {
//!                     if input.mods.contains(ModifiersState::SHIFT) && input.mods.contains(ModifiersState::CONTROL) {
//!                         println!("Terminating!");
//!                         ctx.request_quit();
//!                     } else if input.mods.contains(ModifiersState::SHIFT) || input.mods.contains(ModifiersState::CONTROL) {
//!                         println!("You need to hold both Shift and Control to quit.");
//!                     } else {
//!                         println!("Now you're not even trying!");
//!                     }
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

use std::collections::HashSet;
pub use winit::keyboard::{Key, KeyCode};
use winit::{
    event::KeyEvent,
    keyboard::{ModifiersState, PhysicalKey},
};

/// A simple wrapper bundling the properties of a keyboard stroke.
/// See [`winit`](winit::event::KeyEvent) for more information.
#[derive(Clone, Debug)]
pub struct KeyInput {
    /// The key that was pressed.
    pub event: KeyEvent,
    /// The keyboard modifiers active at the moment of input.
    pub mods: ModifiersState,
}

/// Tracks held down keyboard keys, active keyboard modifiers,
/// and figures out if the system is sending repeat keystrokes.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    /// The currently active modifiers.
    pub active_modifiers: ModifiersState,
    /// The currently pressed physical keys.
    pub pressed_physical_keys: HashSet<PhysicalKey>,
    /// The currently pressed logical keys.
    pub pressed_logical_keys: HashSet<Key>,

    // These two are necessary for tracking key-repeat.
    last_pressed: Option<PhysicalKey>,
    current_pressed: Option<PhysicalKey>,

    // Represents the state of pressed_logical_keys last frame.
    previously_pressed_physical_keys: HashSet<PhysicalKey>,
    previously_pressed_logical_keys: HashSet<Key>,
}

impl KeyboardContext {
    /// Create a new KeyboardContext
    pub fn new() -> Self {
        Self {
            active_modifiers: ModifiersState::default(),
            // We just use 256 as a number Big Enough For Keyboard Keys to try to avoid resizing.
            pressed_physical_keys: HashSet::with_capacity(256),
            pressed_logical_keys: HashSet::with_capacity(256),
            last_pressed: None,
            current_pressed: None,
            previously_pressed_physical_keys: HashSet::with_capacity(256),
            previously_pressed_logical_keys: HashSet::with_capacity(256),
        }
    }

    /// Checks if a key is currently pressed down.
    pub fn is_logical_key_pressed(&self, key: &Key) -> bool {
        self.pressed_logical_keys.contains(key)
    }

    /// Checks if a key has been pressed down this frame.
    pub fn is_logical_key_just_pressed(&self, key: &Key) -> bool {
        self.pressed_logical_keys.contains(key)
            && !self.previously_pressed_logical_keys.contains(key)
    }

    /// Checks if a key has been released this frame.
    pub fn is_logical_key_just_released(&self, key: &Key) -> bool {
        !self.pressed_logical_keys.contains(key)
            && self.previously_pressed_logical_keys.contains(key)
    }

    /// Checks if a key with the corresponding scan code is currently pressed down.
    pub fn is_physical_key_pressed(&self, physical_key: &PhysicalKey) -> bool {
        self.pressed_physical_keys.contains(physical_key)
    }

    /// Checks if a key with the corresponding scan code has been pressed down this frame.
    pub fn is_physical_key_just_pressed(&self, physical_key: &PhysicalKey) -> bool {
        self.pressed_physical_keys.contains(physical_key)
            && !self.previously_pressed_physical_keys.contains(physical_key)
    }

    /// Checks if a key with the corresponding scan code has been released this frame.
    pub fn is_physical_key_just_released(&self, physical_key: &PhysicalKey) -> bool {
        !self.pressed_physical_keys.contains(physical_key)
            && self.previously_pressed_physical_keys.contains(physical_key)
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

    /// Copies the current state of the keyboard into the context. If you are writing your own event loop
    /// you need to call this at the end of every update in order to use the functions `is_key_just_pressed`
    /// and `is_key_just_released`. Otherwise this is handled for you.
    pub fn save_keyboard_state(&mut self) {
        self.previously_pressed_logical_keys
            .clone_from(&self.pressed_logical_keys);
        self.previously_pressed_physical_keys
            .clone_from(&self.pressed_physical_keys);
    }

    pub(crate) fn set_logical_key(&mut self, key: &Key, pressed: bool) {
        if pressed {
            let _ = self.pressed_logical_keys.insert(key.clone());
        } else {
            let _ = self.pressed_logical_keys.remove(key);
        }
    }

    pub(crate) fn set_physical_key(&mut self, physical_key: &PhysicalKey, pressed: bool) {
        if pressed {
            let _ = self.pressed_physical_keys.insert(*physical_key);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(*physical_key);
        } else {
            let _ = self.pressed_physical_keys.remove(physical_key);
            self.current_pressed = None;
        }
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_logical_keys_tracking() {
        let a = &Key::Character("a".into());
        let b = &Key::Character("b".into());
        let empty = HashSet::new();
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.pressed_logical_keys, empty);
        assert!(!keyboard.is_logical_key_pressed(a));
        keyboard.set_logical_key(a, true);
        assert_eq!(
            keyboard.pressed_logical_keys,
            [a.clone()].iter().cloned().collect()
        );
        assert!(keyboard.is_logical_key_pressed(a));
        keyboard.set_logical_key(a, false);
        assert_eq!(keyboard.pressed_logical_keys, empty);
        assert!(!keyboard.is_logical_key_pressed(a));
        keyboard.set_logical_key(a, true);
        assert_eq!(
            keyboard.pressed_logical_keys,
            [a.clone()].iter().cloned().collect()
        );
        assert!(keyboard.is_logical_key_pressed(a));
        keyboard.set_logical_key(a, true);
        assert_eq!(
            keyboard.pressed_logical_keys,
            [a.clone()].iter().cloned().collect()
        );
        keyboard.set_logical_key(b, true);
        assert_eq!(
            keyboard.pressed_logical_keys,
            [a.clone(), b.clone()].iter().cloned().collect()
        );
        keyboard.set_logical_key(b, true);
        assert_eq!(
            keyboard.pressed_logical_keys,
            [a.clone(), b.clone()].iter().cloned().collect()
        );
        keyboard.set_logical_key(a, false);
        assert_eq!(
            keyboard.pressed_logical_keys,
            [b.clone()].iter().cloned().collect()
        );
        keyboard.set_logical_key(a, false);
        assert_eq!(
            keyboard.pressed_logical_keys,
            [b.clone()].iter().cloned().collect()
        );
        keyboard.set_logical_key(b, false);
        assert_eq!(keyboard.pressed_logical_keys, empty);
    }

    #[test]
    fn pressed_scancodes_tracking() {
        let mut keyboard = KeyboardContext::new();
        let a = &PhysicalKey::Code(KeyCode::KeyA);
        let b = &PhysicalKey::Code(KeyCode::KeyB);
        assert_eq!(keyboard.pressed_physical_keys, [].iter().copied().collect());
        assert!(!keyboard.is_physical_key_pressed(a));
        keyboard.set_physical_key(a, true);
        assert_eq!(
            keyboard.pressed_physical_keys,
            [*a].iter().copied().collect()
        );
        assert!(keyboard.is_physical_key_pressed(a));
        keyboard.set_physical_key(a, false);
        assert_eq!(keyboard.pressed_physical_keys, [].iter().copied().collect());
        assert!(!keyboard.is_physical_key_pressed(a));
        keyboard.set_physical_key(a, true);
        assert_eq!(
            keyboard.pressed_physical_keys,
            [*a].iter().copied().collect()
        );
        assert!(keyboard.is_physical_key_pressed(a));
        keyboard.set_physical_key(a, true);
        assert_eq!(
            keyboard.pressed_physical_keys,
            [*a].iter().copied().collect()
        );
        keyboard.set_physical_key(b, true);
        assert_eq!(
            keyboard.pressed_physical_keys,
            [*a, *b].iter().copied().collect()
        );
        keyboard.set_physical_key(b, true);
        assert_eq!(
            keyboard.pressed_physical_keys,
            [*a, *b].iter().copied().collect()
        );
        keyboard.set_physical_key(a, false);
        assert_eq!(
            keyboard.pressed_physical_keys,
            [*b].iter().copied().collect()
        );
        keyboard.set_physical_key(a, false);
        assert_eq!(
            keyboard.pressed_physical_keys,
            [*b].iter().copied().collect()
        );
        keyboard.set_physical_key(b, false);
        assert_eq!(keyboard.pressed_physical_keys, [].iter().copied().collect());
    }

    #[test]
    fn repeated_keys_tracking() {
        let a = &PhysicalKey::Code(KeyCode::KeyA);
        let b = &PhysicalKey::Code(KeyCode::KeyB);
        let mut keyboard = KeyboardContext::new();
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(a, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(a, false);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(a, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(a, true);
        assert!(keyboard.is_key_repeated());
        keyboard.set_physical_key(a, false);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(a, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(b, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(a, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(a, true);
        assert!(keyboard.is_key_repeated());
        keyboard.set_physical_key(b, true);
        assert!(!keyboard.is_key_repeated());
        keyboard.set_physical_key(b, true);
        assert!(keyboard.is_key_repeated());
    }
}
