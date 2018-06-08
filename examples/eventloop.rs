//! This example demonstrates how to roll your own event loop,
//! if for some reason you want to do that instead of using the `EventHandler`
//! trait to do that for you.
//!
//! This is exactly how `ggez::event::run()` works, it really is not
//! doing anything magical.  But, if you want a bit more power over
//! the control flow of your game, this is how you get it.
//!
//! It is functionally identical to the `super_simple.rs` example apart from that.

extern crate ggez;

use ggez::event;
use ggez::graphics::{self, DrawMode, Point2};
use ggez::{conf, Context, GameResult};

pub fn main() -> GameResult<()> {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("eventloop", "ggez", c)?;
    let mut events = event::Events::new(ctx)?;
    let mut continuing = true;

    let mut position: f32 = 1.0;

    while continuing {
        // Tell the timer stuff a frame has happened.
        // Without this the FPS timer functions and such won't work.
        ctx.timer_context.tick();
        // Handle events
        for event in events.poll() {
            ctx.process_event(&event);
            match event {
                event::Event::Quit { .. }
                | event::Event::KeyDown {
                    keycode: Some(event::Keycode::Escape),
                    ..
                } => {
                    println!("Quitting");
                    continuing = false
                }
                x => println!("Event fired: {:?}", x),
            }
        }

        // Update
        position += 1.0;

        // Draw
        graphics::clear(ctx);
        graphics::circle(
            ctx,
            DrawMode::Fill,
            Point2::new(position, 380.0),
            100.0,
            2.0,
        )?;
        graphics::present(ctx);
        ggez::timer::yield_now();
    }
    Ok(())
}
