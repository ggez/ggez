//! A trait for defining a game state.
//! Implement `load()`, `update()` and `draw()` callbacks on this trait
//! and hand it to a `Game` object to be run.
//! You may also implement the `*_event` traits if you wish to handle
//! those events.
//!
//! The default event handlers do nothing, apart from `key_down_event()`,
//! which *should* by default exit the game if escape is pressed, but which
//! doesn't do such a thing yet...
// 
// TODO: We need an is_done callback or something...

use std::time::Duration;
use sdl2::event::Event;

use {GameResult};
use context::Context;
use conf::Conf;

// I feel like this might be better named a Scene than a State...?
// No, because scene management is more fine-grained and should
// happen at a higher level.
pub trait State {

    // Tricksy trait and lifetime magic!
    // Much thanks to aatch on #rust-beginners for helping make this work.
    fn load(ctx: &mut Context, conf: &Conf) -> GameResult<Self>
        where Self: Sized;
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()>;
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()>;

    // You don't have to override these if you don't want to; the defaults
    // do nothing.
    // It might be nice to be able to have custom event types and a map or
    // such of handlers?  Hmm, maybe later.
    fn mouse_button_down_event(&mut self, _evt: Event) {}

    fn mouse_button_up_event(&mut self, _evt: Event) {}

    fn mouse_motion_event(&mut self, _evt: Event) {}

    fn mouse_wheel_event(&mut self, _evt: Event) {}

    // TODO: These event types need to be better,
    // but I'm not sure how to do it yet.
    // They should be SdlEvent::KeyDow or something similar,
    // but those are enum fields, not actual types.
    fn key_down_event(&mut self, _evt: Event) {
        //done = true,
    }

    fn key_up_event(&mut self, _evt: Event) {}

    fn focus(&mut self, _gained: bool) {}

    fn quit(&mut self) {
        println!("Quitting game");
    }
}
