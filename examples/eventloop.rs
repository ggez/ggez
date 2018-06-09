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
use ggez::event::winit_event::{Event, KeyboardInput, WindowEvent};
use ggez::graphics::{self, DrawMode, Point2};
use ggez::{conf, Context, GameResult};

pub fn main() -> GameResult {
    let c = conf::Conf::new();
    let (ctx, events_loop) = &mut Context::load_from_conf("eventloop", "ggez", c)?;

    let mut position: f32 = 1.0;

    // This is also used in the loop inside `::run()` - it can be flipped off with `ctx.quit()`
    while ctx.continuing {
        // Tell the timer stuff a frame has happened.
        // Without this the FPS timer functions and such won't work.
        ctx.timer_context.tick();
        // Handle events. Refer to `winit` docs for more information.
        events_loop.poll_events(|event| {
            // This tells `ggez` to update it's internal states, should the event require that.
            // These include cursor position, view updating on resize, etc.
            ctx.process_event(&event);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => ctx.quit(),
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        event::KeyCode::Escape => ctx.quit(),

                        _ => (),
                    },
                    // `CloseRequested` and `KeyboardInput` events won't appear here.
                    x => println!("Other window event fired: {:?}", x),
                },

                x => println!("Device event fired: {:?}", x),
            }
        });

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
        graphics::present(ctx)?;
        ggez::timer::yield_now();
    }
    Ok(())
}
