//! This example demonstrates how to roll your own event loop,
//! if for some reason you want to do that instead of using the `EventHandler`
//! trait to do that for you.
//!
//! This is how `ggez::event::run()` works, mostly, (if you want to see which parts were left out
//! of this example, check [event.rs](https://github.com/ggez/ggez/blob/master/src/event.rs),
//! it really is not doing anything magical.  But, if you want a bit more power over
//! the control flow of your game, this is how you get it.
//!
//! It is functionally identical to the `super_simple.rs` example apart from that.

use ggez::event;
use ggez::event::winit_event::{Event, KeyboardInput, WindowEvent};
use ggez::graphics::{self, Color, DrawMode};
use ggez::input::keyboard;
use ggez::GameResult;
use winit::event_loop::ControlFlow;

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("eventloop", "ggez");
    let (mut ctx, events_loop) = cb.build()?;

    let mut position: f32 = 1.0;

    // Handle events. Refer to `winit` docs for more information.
    events_loop.run(move |mut event, _window_target, control_flow| {
        let ctx = &mut ctx;

        if ctx.quit_requested {
            ctx.continuing = false;
        }
        if !ctx.continuing {
            *control_flow = ControlFlow::Exit;
            return;
        }

        *control_flow = ControlFlow::Poll;

        // This tells `ggez` to update it's internal states, should the event require that.
        // These include cursor position, view updating on resize, etc.
        event::process_event(ctx, &mut event);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => ctx.request_quit(),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    if let keyboard::KeyCode::Escape = keycode {
                        ctx.request_quit();
                    }
                }
                // `CloseRequested` and `KeyboardInput` events won't appear here.
                x => println!("Other window event fired: {:?}", x),
            },
            Event::MainEventsCleared => {
                // Tell the timer stuff a frame has happened.
                // Without this the FPS timer functions and such won't work.
                ctx.time.tick();

                // Update
                position += 1.0;

                // Draw
                ctx.gfx.begin_frame().unwrap();

                let mut canvas =
                    graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

                let circle = graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    ggez::glam::Vec2::new(0.0, 0.0),
                    100.0,
                    2.0,
                    Color::WHITE,
                )
                .unwrap();
                canvas.draw(&circle, ggez::glam::Vec2::new(position, 380.0));

                canvas.finish(ctx).unwrap();
                ctx.gfx.end_frame().unwrap();

                // reset the mouse delta for the next frame
                // necessary because it's calculated cumulatively each cycle
                ctx.mouse.reset_delta();

                // Copy the state of the keyboard into the KeyboardContext and
                // the mouse into the MouseContext.
                // Not required for this example but important if you want to
                // use the functions keyboard::is_key_just_pressed/released and
                // mouse::is_button_just_pressed/released.
                ctx.keyboard.save_keyboard_state();
                ctx.mouse.save_mouse_state();

                ggez::timer::yield_now();
            }

            x => println!("Device event fired: {:?}", x),
        }
    });
}
