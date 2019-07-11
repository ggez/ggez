//! This example demonstrates how to roll your own event loop,
//! if for some reason you want to do that instead of using the `EventHandler`
//! trait to do that for you.
//!
//! This is exactly how `ggez::event::run()` works, it really is not
//! doing anything magical.  But, if you want a bit more power over
//! the control flow of your game, this is how you get it.
//!
//! It is functionally identical to the `super_simple.rs` example apart from that.

use cgmath;
use ggez;

use ggez::event;
use ggez::event::winit_event::{Event, KeyboardInput, WindowEvent};
use ggez::graphics::{self, DrawMode};
use ggez::GameResult;

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("eventloop", "ggez");
    let (ctx, events_loop) = &mut cb.build()?;

    let mut position: f32 = 1.0;

    // This is also used in the loop inside `::run()` - it can be flipped off with `event::quit()`
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
                    WindowEvent::CloseRequested => event::quit(ctx),
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        event::KeyCode::Escape => event::quit(ctx),

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
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            cgmath::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(ctx, &circle, (cgmath::Point2::new(position, 380.0),))?;
        graphics::present(ctx)?;
        ggez::timer::yield_now();
    }
    Ok(())
}
