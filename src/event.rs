//! The `event` module contains traits and structs to actually run your game mainloop
//! and handle top-level state, as well as handle input events such as keyboard
//! and mouse.
//!
//! If you don't want to use `ggez`'s built in event loop, you can
//! write your own mainloop and check for events on your own.  This is
//! not particularly hard, there's nothing special about the
//! `EventHandler` trait.  It just tries to simplify the process a
//! little.  For examples of how to write your own main loop, see the
//! source code for this module, or the [`eventloop`
//! example](https://github.com/ggez/ggez/blob/master/examples/eventloop.rs).

use winit::{self, dpi};

/// A mouse button.
pub use winit::event::MouseButton;

/// An analog axis of some device (gamepad thumbstick, joystick...).
pub use gilrs::Axis;
/// A button of some device (gamepad, joystick...).
pub use gilrs::Button;

/// `winit` events; nested in a module for re-export neatness.
pub mod winit_event {
    pub use super::winit::event::{
        DeviceEvent, ElementState, Event, KeyboardInput, ModifiersState, MouseScrollDelta,
        TouchPhase, WindowEvent,
    };
}
pub use crate::input::gamepad::GamepadId;
pub use crate::input::keyboard::{KeyCode, KeyMods};
use crate::GameError;

use self::winit_event::*;
/// `winit` event loop.
pub use winit::event_loop::{ControlFlow, EventLoop};

use crate::context::Context;

/// Used in [`EventHandler::on_error()`](trait.EventHandler.html#method.on_error)
/// to specify where an error originated
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorOrigin {
    /// error originated in `update()`
    Update,
    /// error originated in `draw()`
    Draw,
}

/// A trait defining event callbacks.  This is your primary interface with
/// `ggez`'s event loop.  Implement this trait for a type and
/// override at least the [`update()`](#tymethod.update) and
/// [`draw()`](#tymethod.draw) methods, then pass it to
/// [`event::run()`](fn.run.html) to run the game's mainloop.
///
/// The default event handlers do nothing, apart from
/// [`key_down_event()`](#method.key_down_event), which will by
/// default exit the game if the escape key is pressed.  Just
/// override the methods you want to use.
///
/// For the error type simply choose the default [`GameError`](../error/enum.GameError.html),
/// or something more generic, if your situation requires it.
pub trait EventHandler<E = GameError>
where
    E: std::error::Error,
{
    /// Called upon each logic update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, _ctx: &mut Context) -> Result<(), E>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// [`graphics::clear()`](../graphics/fn.clear.html) and end it
    /// with [`graphics::present()`](../graphics/fn.present.html) and
    /// maybe [`timer::yield_now()`](../timer/fn.yield_now.html).
    fn draw(&mut self, _ctx: &mut Context) -> Result<(), E>;

    /// A mouse button was pressed
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    /// A mouse button was released
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    /// The mouse was moved; it provides both absolute x and y coordinates in the window,
    /// and relative x and y coordinates compared to its last position.
    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}

    /// mouse entered or left window area
    fn mouse_enter_or_leave(&mut self, _ctx: &mut Context, _entered: bool) {}

    /// The mousewheel was scrolled, vertically (y, positive away from and negative toward the user)
    /// or horizontally (x, positive to the right and negative to the left).
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}

    /// A keyboard button was pressed.
    ///
    /// The default implementation of this will call `ggez::event::quit()`
    /// when the escape key is pressed.  If you override this with
    /// your own event handler you have to re-implment that
    /// functionality yourself.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Escape {
            quit(ctx);
        }
    }

    /// A keyboard button was released.
    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymods: KeyMods) {}

    /// A unicode character was received, usually from keyboard input.
    /// This is the intended way of facilitating text input.
    fn text_input_event(&mut self, _ctx: &mut Context, _character: char) {}

    /// A gamepad button was pressed; `id` identifies which gamepad.
    /// Use [`input::gamepad()`](../input/fn.gamepad.html) to get more info about
    /// the gamepad.
    fn gamepad_button_down_event(&mut self, _ctx: &mut Context, _btn: Button, _id: GamepadId) {}

    /// A gamepad button was released; `id` identifies which gamepad.
    /// Use [`input::gamepad()`](../input/fn.gamepad.html) to get more info about
    /// the gamepad.
    fn gamepad_button_up_event(&mut self, _ctx: &mut Context, _btn: Button, _id: GamepadId) {}

    /// A gamepad axis moved; `id` identifies which gamepad.
    /// Use [`input::gamepad()`](../input/fn.gamepad.html) to get more info about
    /// the gamepad.
    fn gamepad_axis_event(&mut self, _ctx: &mut Context, _axis: Axis, _value: f32, _id: GamepadId) {
    }

    /// Called when the window is shown or hidden.
    fn focus_event(&mut self, _ctx: &mut Context, _gained: bool) {}

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit (the quit event is cancelled).
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        debug!("quit_event() callback called, quitting...");
        false
    }

    /// Called when the user resizes the window, or when it is resized
    /// via [`graphics::set_mode()`](../graphics/fn.set_mode.html).
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}

    /// Something went wrong, causing a `GameError`.
    /// If this returns true, the error was fatal, so the event loop ends, aborting the game.
    fn on_error(&mut self, _ctx: &mut Context, _origin: ErrorOrigin, _e: E) -> bool {
        true
    }
}

/// Terminates the [`ggez::event::run()`](fn.run.html) loop by setting
/// [`Context.continuing`](struct.Context.html#structfield.continuing)
/// to `false`.
pub fn quit(ctx: &mut Context) {
    ctx.continuing = false;
}

/// Runs the game's main loop, calling event callbacks on the given state
/// object as events occur.
///
/// It does not try to do any type of framerate limiting.  See the
/// documentation for the [`timer`](../timer/index.html) module for more info.
pub fn run<S: 'static, E>(mut ctx: Context, event_loop: EventLoop<()>, mut state: S) -> !
where
    S: EventHandler<E>,
    E: std::error::Error,
{
    use crate::input::{keyboard, mouse};

    event_loop.run(move |mut event, _, control_flow| {
        if !ctx.continuing {
            *control_flow = ControlFlow::Exit;
            return;
        }

        *control_flow = ControlFlow::Poll;

        let ctx = &mut ctx;
        let state = &mut state;

        process_event(ctx, &mut event);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    // let actual_size = logical_size;
                    state.resize_event(ctx, logical_size.width as f32, logical_size.height as f32);
                }
                WindowEvent::CloseRequested => {
                    if !state.quit_event(ctx) {
                        quit(ctx);
                    }
                }
                WindowEvent::Focused(gained) => {
                    state.focus_event(ctx, gained);
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    state.text_input_event(ctx, ch);
                }
                WindowEvent::ModifiersChanged(mods) => {
                    ctx.keyboard_context.set_modifiers(KeyMods::from(mods))
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    let repeat = keyboard::is_key_repeated(ctx);
                    state.key_down_event(ctx, keycode, ctx.keyboard_context.active_mods(), repeat);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    state.key_up_event(ctx, keycode, ctx.keyboard_context.active_mods());
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let (x, y) = match delta {
                        MouseScrollDelta::LineDelta(x, y) => (x, y),
                        MouseScrollDelta::PixelDelta(pos) => {
                            let scale_factor = ctx.gfx_context.window.window().scale_factor();
                            let dpi::LogicalPosition { x, y } = pos.to_logical::<f32>(scale_factor);
                            (x, y)
                        }
                    };
                    state.mouse_wheel_event(ctx, x, y);
                }
                WindowEvent::MouseInput {
                    state: element_state,
                    button,
                    ..
                } => {
                    let position = mouse::position(ctx);
                    match element_state {
                        ElementState::Pressed => {
                            state.mouse_button_down_event(ctx, button, position.x, position.y)
                        }
                        ElementState::Released => {
                            state.mouse_button_up_event(ctx, button, position.x, position.y)
                        }
                    }
                }
                WindowEvent::CursorMoved { .. } => {
                    let position = mouse::position(ctx);
                    let delta = mouse::last_delta(ctx);
                    state.mouse_motion_event(ctx, position.x, position.y, delta.x, delta.y);
                }
                _x => {
                    // trace!("ignoring window event {:?}", x);
                }
            },
            Event::DeviceEvent { .. } => (),
            Event::Resumed => (),
            Event::Suspended => (),
            Event::NewEvents(_) => (),
            Event::UserEvent(_) => (),
            Event::MainEventsCleared => {
                // If you are writing your own event loop, make sure
                // you include `timer_context.tick()` and
                // `ctx.process_event()` calls.  These update ggez's
                // internal state however necessary.
                ctx.timer_context.tick();

                // Handle gamepad events if necessary.
                if ctx.conf.modules.gamepad {
                    while let Some(gilrs::Event { id, event, .. }) =
                        ctx.gamepad_context.next_event()
                    {
                        match event {
                            gilrs::EventType::ButtonPressed(button, _) => {
                                state.gamepad_button_down_event(ctx, button, GamepadId(id));
                            }
                            gilrs::EventType::ButtonReleased(button, _) => {
                                state.gamepad_button_up_event(ctx, button, GamepadId(id));
                            }
                            gilrs::EventType::AxisChanged(axis, value, _) => {
                                state.gamepad_axis_event(ctx, axis, value, GamepadId(id));
                            }
                            _ => {}
                        }
                    }
                }

                if let Err(e) = state.update(ctx) {
                    error!("Error on EventHandler::update(): {:?}", e);
                    eprintln!("Error on EventHandler::update(): {:?}", e);
                    if state.on_error(ctx, ErrorOrigin::Update, e) {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                if let Err(e) = state.draw(ctx) {
                    error!("Error on EventHandler::draw(): {:?}", e);
                    eprintln!("Error on EventHandler::draw(): {:?}", e);
                    if state.on_error(ctx, ErrorOrigin::Draw, e) {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                // reset the mouse delta for the next frame
                // necessary because it's calculated cumulatively each cycle
                ctx.mouse_context.reset_delta();
            }
            Event::RedrawRequested(_) => (),
            Event::RedrawEventsCleared => (),
            Event::LoopDestroyed => (),
        }
    })
}

/// Feeds an `Event` into the `Context` so it can update any internal
/// state it needs to, such as detecting window resizes.  If you are
/// rolling your own event loop, you should call this on the events
/// you receive before processing them yourself.
pub fn process_event(ctx: &mut Context, event: &mut winit::event::Event<()>) {
    if let winit_event::Event::WindowEvent { event, .. } = event {
        match event {
            winit_event::WindowEvent::Resized(physical_size) => {
                ctx.gfx_context.window.resize(*physical_size);
                ctx.gfx_context.resize_viewport();
            }
            winit_event::WindowEvent::CursorMoved {
                position: logical_position,
                ..
            } => {
                let current_delta = crate::input::mouse::delta(ctx);
                let current_pos = crate::input::mouse::position(ctx);
                let diff = crate::graphics::Point2::new(
                    logical_position.x as f32 - current_pos.x,
                    logical_position.y as f32 - current_pos.y,
                );
                // Sum up the cumulative mouse change for this frame in `delta`:
                ctx.mouse_context.set_delta(crate::graphics::Point2::new(
                    current_delta.x + diff.x,
                    current_delta.y + diff.y,
                ));
                // `last_delta` is not cumulative.
                // It represents only the change between the last mouse event and the current one.
                ctx.mouse_context.set_last_delta(diff);
                ctx.mouse_context
                    .set_last_position(crate::graphics::Point2::new(
                        logical_position.x as f32,
                        logical_position.y as f32,
                    ));
            }
            winit_event::WindowEvent::MouseInput { button, state, .. } => {
                let pressed = match state {
                    winit_event::ElementState::Pressed => true,
                    winit_event::ElementState::Released => false,
                };
                ctx.mouse_context.set_button(*button, pressed);
            }
            winit_event::WindowEvent::ModifiersChanged(mods) => ctx
                .keyboard_context
                .set_modifiers(crate::input::keyboard::KeyMods::from(*mods)),
            winit_event::WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let pressed = match state {
                    winit_event::ElementState::Pressed => true,
                    winit_event::ElementState::Released => false,
                };
                ctx.keyboard_context.set_key(*keycode, pressed);
            }
            winit_event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                if !ctx.conf.window_mode.resize_on_scale_factor_change {
                    // actively set the new_inner_size to be the desired size
                    // to stop winit from resizing our window
                    **new_inner_size = winit::dpi::PhysicalSize::<u32>::from([
                        ctx.conf.window_mode.width,
                        ctx.conf.window_mode.height,
                    ]);
                }
            }
            _ => (),
        }
    };
}
