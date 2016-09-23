use std::time::Duration;
use sdl2::event::Event;

use GameError;
use context::Context;

// I feel like this might be better named a Scene than a State...?
// No, because scene management is more fine-grained and should
// happen at a higher level.
pub trait State {
    fn load(&mut self, ctx: &mut Context) -> Result<(), GameError>;
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> Result<(), GameError>;
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError>;

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
    fn key_down_event(&mut self, _evt: Event) {}

    fn key_up_event(&mut self, _evt: Event) {}

    fn focus(&mut self, _gained: bool) {}

    fn quit(&mut self) {
        println!("Quitting game");
    }
}
