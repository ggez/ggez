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
    /// error originated in `mouse_button_down_event()`
    MouseButtonDownEvent,
    /// error originated in `mouse_button_up_event()`
    MouseButtonUpEvent,
    /// error originated in `mouse_motion_event()`
    MouseMotionEvent,
    /// error originated in `mouse_enter_or_leave()`
    MouseEnterOrLeave,
    /// error originated in `mouse_wheel_event()`
    MouseWheelEvent,
    /// error originated in `key_down_event()`
    KeyDownEvent,
    /// error originated in `key_up_event()`
    KeyUpEvent,
    /// error originated in `text_input_event()`
    TextInputEvent,
    /// error originated in `touch_event()`
    TouchEvent,
    /// error originated in `gamepad_button_down_event()`
    GamepadButtonDownEvent,
    /// error originated in `gamepad_button_up_event()`
    GamepadButtonUpEvent,
    /// error originated in `gamepad_axis_event()`
    GamepadAxisEvent,
    /// error originated in `focus_event()`
    FocusEvent,
    /// error originated in `quit_event()`
    QuitEvent,
    /// error originated in `resize_event()`
    ResizeEvent,
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
    ) -> Result<(), E> {
        Ok(())
    }

    /// A mouse button was released
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), E> {
        Ok(())
    }

    /// The mouse was moved; it provides both absolute x and y coordinates in the window,
    /// and relative x and y coordinates compared to its last position.
    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _x: f32,
        _y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), E> {
        Ok(())
    }

    /// mouse entered or left window area
    fn mouse_enter_or_leave(&mut self, _ctx: &mut Context, _entered: bool) -> Result<(), E> {
        Ok(())
    }

    /// The mousewheel was scrolled, vertically (y, positive away from and negative toward the user)
    /// or horizontally (x, positive to the right and negative to the left).
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) -> Result<(), E> {
        Ok(())
    }

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
    ) -> Result<(), E> {
        if keycode == KeyCode::Escape {
            quit(ctx);
        }
        Ok(())
    }

    /// A keyboard button was released.
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        _keycode: KeyCode,
        _keymods: KeyMods,
    ) -> Result<(), E> {
        Ok(())
    }

    /// A unicode character was received, usually from keyboard input.
    /// This is the intended way of facilitating text input.
    fn text_input_event(&mut self, _ctx: &mut Context, _character: char) -> Result<(), E> {
        Ok(())
    }

    /// An event from a touchscreen has been triggered; it provides the x and y location
    /// inside the window as well as the state of the tap (such as Started, Moved, Ended, etc)
    /// By default, touch events will trigger mouse behavior
    fn touch_event(
        &mut self,
        ctx: &mut Context,
        phase: TouchPhase,
        x: f64,
        y: f64,
    ) -> Result<(), E> {
        crate::input::mouse::handle_move(ctx, x as f32, y as f32);

        match phase {
            TouchPhase::Started => {
                ctx.mouse_context.set_button(MouseButton::Left, true);
                self.mouse_button_down_event(ctx, MouseButton::Left, x as f32, y as f32)?;
            }
            TouchPhase::Moved => {
                let diff = crate::input::mouse::last_delta(ctx);
                self.mouse_motion_event(ctx, x as f32, y as f32, diff.x, diff.y)?;
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                ctx.mouse_context.set_button(MouseButton::Left, false);
                self.mouse_button_up_event(ctx, MouseButton::Left, x as f32, y as f32)?;
            }
        }

        Ok(())
    }

    /// A gamepad button was pressed; `id` identifies which gamepad.
    /// Use [`input::gamepad()`](../input/fn.gamepad.html) to get more info about
    /// the gamepad.
    fn gamepad_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _btn: Button,
        _id: GamepadId,
    ) -> Result<(), E> {
        Ok(())
    }

    /// A gamepad button was released; `id` identifies which gamepad.
    /// Use [`input::gamepad()`](../input/fn.gamepad.html) to get more info about
    /// the gamepad.
    fn gamepad_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _btn: Button,
        _id: GamepadId,
    ) -> Result<(), E> {
        Ok(())
    }

    /// A gamepad axis moved; `id` identifies which gamepad.
    /// Use [`input::gamepad()`](../input/fn.gamepad.html) to get more info about
    /// the gamepad.
    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut Context,
        _axis: Axis,
        _value: f32,
        _id: GamepadId,
    ) -> Result<(), E> {
        Ok(())
    }

    /// Called when the window is shown or hidden.
    fn focus_event(&mut self, _ctx: &mut Context, _gained: bool) -> Result<(), E> {
        Ok(())
    }

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit (the quit event is cancelled).
    fn quit_event(&mut self, _ctx: &mut Context) -> Result<bool, E> {
        debug!("quit_event() callback called, quitting...");
        Ok(false)
    }

    /// Called when the user resizes the window, or when it is resized
    /// via [`graphics::set_mode()`](../graphics/fn.set_mode.html).
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) -> Result<(), E> {
        Ok(())
    }

    /// Something went wrong, causing a `GameError` (or some other kind of error, depending on what you specified).
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
#[allow(clippy::needless_return)] // necessary as the returns used here are actually necessary to break early from the event loop
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
                    let res = state.resize_event(
                        ctx,
                        logical_size.width as f32,
                        logical_size.height as f32,
                    );
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::ResizeEvent) {
                        return;
                    };
                }
                WindowEvent::CloseRequested => {
                    let res = state.quit_event(ctx);
                    if let Ok(false) = state.quit_event(ctx) {
                        quit(ctx);
                    } else if catch_error(ctx, res, state, control_flow, ErrorOrigin::QuitEvent) {
                        return;
                    }
                }
                WindowEvent::Focused(gained) => {
                    let res = state.focus_event(ctx, gained);
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::FocusEvent) {
                        return;
                    };
                }
                WindowEvent::ReceivedCharacter(ch) => {
                    let res = state.text_input_event(ctx, ch);
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::TextInputEvent) {
                        return;
                    };
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
                    let res = state.key_down_event(
                        ctx,
                        keycode,
                        ctx.keyboard_context.active_mods(),
                        repeat,
                    );
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::KeyDownEvent) {
                        return;
                    };
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
                    let res = state.key_up_event(ctx, keycode, ctx.keyboard_context.active_mods());
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::KeyUpEvent) {
                        return;
                    };
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
                    let res = state.mouse_wheel_event(ctx, x, y);
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::MouseWheelEvent) {
                        return;
                    };
                }
                WindowEvent::MouseInput {
                    state: element_state,
                    button,
                    ..
                } => {
                    let position = mouse::position(ctx);
                    match element_state {
                        ElementState::Pressed => {
                            let res =
                                state.mouse_button_down_event(ctx, button, position.x, position.y);
                            if catch_error(
                                ctx,
                                res,
                                state,
                                control_flow,
                                ErrorOrigin::MouseButtonDownEvent,
                            ) {
                                return;
                            };
                        }
                        ElementState::Released => {
                            let res =
                                state.mouse_button_up_event(ctx, button, position.x, position.y);
                            if catch_error(
                                ctx,
                                res,
                                state,
                                control_flow,
                                ErrorOrigin::MouseButtonUpEvent,
                            ) {
                                return;
                            };
                        }
                    }
                }
                WindowEvent::CursorMoved { .. } => {
                    let position = mouse::position(ctx);
                    let delta = mouse::last_delta(ctx);
                    let res =
                        state.mouse_motion_event(ctx, position.x, position.y, delta.x, delta.y);
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::MouseMotionEvent) {
                        return;
                    };
                }
                WindowEvent::Touch(touch) => {
                    let res =
                        state.touch_event(ctx, touch.phase, touch.location.x, touch.location.y);
                    if catch_error(ctx, res, state, control_flow, ErrorOrigin::TouchEvent) {
                        return;
                    };
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
                                let res =
                                    state.gamepad_button_down_event(ctx, button, GamepadId(id));
                                if catch_error(
                                    ctx,
                                    res,
                                    state,
                                    control_flow,
                                    ErrorOrigin::GamepadButtonDownEvent,
                                ) {
                                    return;
                                };
                            }
                            gilrs::EventType::ButtonReleased(button, _) => {
                                let res = state.gamepad_button_up_event(ctx, button, GamepadId(id));
                                if catch_error(
                                    ctx,
                                    res,
                                    state,
                                    control_flow,
                                    ErrorOrigin::GamepadButtonUpEvent,
                                ) {
                                    return;
                                };
                            }
                            gilrs::EventType::AxisChanged(axis, value, _) => {
                                let res = state.gamepad_axis_event(ctx, axis, value, GamepadId(id));
                                if catch_error(
                                    ctx,
                                    res,
                                    state,
                                    control_flow,
                                    ErrorOrigin::GamepadAxisEvent,
                                ) {
                                    return;
                                };
                            }
                            _ => {}
                        }
                    }
                }

                let res = state.update(ctx);
                if catch_error(ctx, res, state, control_flow, ErrorOrigin::Update) {
                    return;
                };

                let res = state.draw(ctx);
                if catch_error(ctx, res, state, control_flow, ErrorOrigin::Draw) {
                    return;
                };

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

fn catch_error<T, E, S: 'static>(
    ctx: &mut Context,
    event_result: Result<T, E>,
    state: &mut S,
    control_flow: &mut ControlFlow,
    origin: ErrorOrigin,
) -> bool
where
    E: std::error::Error,
    S: EventHandler<E>,
{
    if let Err(e) = event_result {
        error!("Error on EventHandler {:?}: {:?}", origin, e);
        eprintln!("Error on EventHandler {:?}: {:?}", origin, e);
        if state.on_error(ctx, origin, e) {
            *control_flow = ControlFlow::Exit;
            return true;
        }
    }
    false
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
                position: physical_position,
                ..
            } => {
                crate::input::mouse::handle_move(
                    ctx,
                    physical_position.x as f32,
                    physical_position.y as f32,
                );
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
