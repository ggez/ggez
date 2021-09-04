//! This example demonstrates how to roll your own event loop,
//! if for some reason you want to do that instead of using the `EventHandler`
//! trait to do that for you.
//!
//! This is exactly how `ggez::event::run()` works, it really is not
//! doing anything magical.  But, if you want a bit more power over
//! the control flow of your game, this is how you get it.
//!
//! It is functionally identical to the `super_simple.rs` example apart from that.

use ggez::event;
use ggez::event::winit_event::{Event, KeyboardInput, WindowEvent};
use ggez::graphics::{self, Color, DrawMode};
use ggez::GameResult;
use winit::event_loop::ControlFlow;

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("eventloop", "ggez");
    let (mut ctx, events_loop) = cb.build()?;

    let mut position: f32 = 1.0;

    // Handle events. Refer to `winit` docs for more information.
    events_loop.run(move |mut event, _window_target, control_flow| {
        if !ctx.continuing {
            *control_flow = ControlFlow::Exit;
            return;
        }

        *control_flow = ControlFlow::Poll;

        let ctx = &mut ctx;

        // This tells `ggez` to update it's internal states, should the event require that.
        // These include cursor position, view updating on resize, etc.
        event::process_event(ctx, &mut event);
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
                } => {
                    if let event::KeyCode::Escape = keycode {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                }
                // `CloseRequested` and `KeyboardInput` events won't appear here.
                x => println!("Other window event fired: {:?}", x),
            },
            Event::MainEventsCleared => {
                // Tell the timer stuff a frame has happened.
                // Without this the FPS timer functions and such won't work.
                ctx.timer_context.tick();

                // Update
                position += 1.0;

                // Draw
                graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    glam::Vec2::new(0.0, 0.0),
                    100.0,
                    2.0,
                    Color::WHITE,
                )
                .unwrap();
                graphics::draw(ctx, &circle, (glam::Vec2::new(position, 380.0),)).unwrap();
                graphics::present(ctx).unwrap();

                // reset the mouse delta for the next frame
                // necessary because it's calculated cumulatively each cycle
                ctx.mouse_context.reset_delta();

                ggez::timer::yield_now();
            }

            x => println!("Device event fired: {:?}", x),
        }
    });
}
