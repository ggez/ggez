//! The `game` module contains traits and structs to actually run your game mainloop
//! and handle top-level state.

use context::Context;
use GameResult;
use conf;
use timer;

use std::time::Duration;

use super::event as gevent;

use sdl2::event::Event::*;
use sdl2::event;
use sdl2::mouse;
use sdl2::keyboard;

/// A trait for defining a game state.
/// Implement `load()`, `update()` and `draw()` callbacks on this trait
/// and create a `Game` object using your gamestate type.
/// You may also implement the `*_event` callbacks if you wish to handle
/// those events.
///
/// The default event handlers do nothing, apart from `key_down_event()`,
/// which *should* by default exit the game if escape is pressed.
/// (Once we work around some event bugs in rust-sdl2.)
pub trait GameState {
    // Tricksy trait and lifetime magic happens in load()'s
    // signature.
    // It doesn't look complicated but is easy to get wrong.
    // Much thanks to aatch on #rust-beginners for helping make this work.

    /// Called to initially create your `GameState` object
    /// after all hardware initialization has been done.
    /// It is handed a `Context` to load resources from,
    /// and the `Conf` object that has either been loaded
    /// from your `resources/conf.toml` file or the default
    /// that has been provided to `Game::new()` if no conf
    /// file exists.
    fn load(ctx: &mut Context) -> GameResult<Self> where Self: Sized;

    /// Called upon each physics update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// `graphics::clear()` and end it with
    /// `graphics::present()` and `timer::sleep_until_next_frame()`
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()>;

    // You don't have to override these if you don't want to; the defaults
    // do nothing.
    // It might be nice to be able to have custom event types and a map or
    // such of handlers?  Hmm, maybe later.
    fn mouse_button_down_event(&mut self, _button: mouse::MouseButton, _x: i32, _y: i32) {}

    fn mouse_button_up_event(&mut self, _button: mouse::MouseButton, _x: i32, _y: i32) {}

    fn mouse_motion_event(&mut self,
                          _state: mouse::MouseState,
                          _x: i32,
                          _y: i32,
                          _xrel: i32,
                          _yrel: i32) {
    }

    fn mouse_wheel_event(&mut self, _x: i32, _y: i32) {}

    fn key_down_event(&mut self, _keycode: gevent::Keycode, _keymod: gevent::Mod, _repeat: bool) {}

    fn key_up_event(&mut self, _keycode: gevent::Keycode, _keymod: gevent::Mod, _repeat: bool) {}

    fn focus_event(&mut self, _gained: bool) {}

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit.
    fn quit_event(&mut self) -> bool {
        println!("Quitting game");
        false
    }
}



/// The `Game` struct takes an object you define that
/// implements the `GameState` trait
/// and does the actual work of running a gameloop,
/// passing events to your handlers, and all that stuff.
#[derive(Debug)]
pub struct Game<'a, S> {
    state: S,
    context: Context<'a>,
}



impl<'a, S> Game<'a, S> {
    pub fn from_state(ctx: Context<'a>, state: S) -> Game<S> {
        Game {
            state: state,
            context: ctx,
        }
    }

    /// Replaces the gamestate with the given one without
    /// having to re-initialize the hardware context.
    pub fn replace_state(&mut self, state: S) {
        self.state = state;
    }

    /// Calls the given function to create a new gamestate, and replaces
    /// the current one with it.
    pub fn replace_state_with<F>(&mut self, f: &F) -> GameResult<()>
        where F: Fn(&mut Context) -> GameResult<S>
    {
        let newstate = f(&mut self.context)?;
        self.state = newstate;
        Ok(())
    }
}

impl<'a, S: GameState + 'static> Game<'a, S> {
    /// Creates a new `Game` with the given  default config
    /// (which will be used if there is no config file).
    /// It will initialize a hardware context and call the `load()` method of
    /// the given `GameState` type to create a new `GameState`.
    ///
    /// The `id` field is a unique identifier for your game that will
    /// be used to create a save directory to write files to.
    pub fn new(id: &str, default_config: conf::Conf) -> GameResult<Game<'a, S>> {
        let mut context = Context::load_from_conf(id, default_config)?;

        let init_state = S::load(&mut context)?;

        Ok(Game {
            state: init_state,
            context: context,
        })
    }


    /// Re-creates a fresh `GameState` using the existing one's `::load()` method.
    pub fn reload_state(&mut self) -> GameResult<()> {
        let newstate = S::load(&mut self.context)?;
        self.state = newstate;
        Ok(())
    }

    /// Runs the game's mainloop.
    /// Continues until a `Quit` event is created, for instance
    /// via `Context::quit()`
    pub fn run(&mut self) -> GameResult<()> {
        let ctx = &mut self.context;
        let mut event_pump = ctx.sdl_context.event_pump()?;

        let mut continuing = true;
        while continuing {
            ctx.timer_context.tick();

            for event in event_pump.poll_iter() {
                match event {
                    Quit { .. } => {
                        continuing = self.state.quit_event();
                        // println!("Quit event: {:?}", t);
                    }
                    // TODO: We need a good way to have
                    // a default like this, while still allowing
                    // it to be overridden.
                    // Bah, just put it in the GameState trait
                    // as the default function.
                    // But it doesn't have access to the context
                    // to call quit!  Bah.
                    KeyDown { keycode, keymod, repeat, .. } => {
                        if let Some(key) = keycode {
                            if key == keyboard::Keycode::Escape {
                                ctx.quit()?;
                            } else {
                                self.state.key_down_event(key, keymod, repeat)
                            }
                        }
                    }
                    KeyUp { keycode, keymod, repeat, .. } => {
                        if let Some(key) = keycode {
                            self.state.key_up_event(key, keymod, repeat)
                        }
                    }
                    MouseButtonDown { mouse_btn, x, y, .. } => {
                        self.state.mouse_button_down_event(mouse_btn, x, y)
                    }
                    MouseButtonUp { mouse_btn, x, y, .. } => {
                        self.state.mouse_button_up_event(mouse_btn, x, y)
                    }
                    MouseMotion { mousestate, x, y, xrel, yrel, .. } => {
                        self.state.mouse_motion_event(mousestate, x, y, xrel, yrel)
                    }
                    MouseWheel { x, y, .. } => self.state.mouse_wheel_event(x, y),
                    Window { win_event: event::WindowEvent::FocusGained, .. } => {
                        self.state.focus_event(true)
                    }
                    Window { win_event: event::WindowEvent::FocusLost, .. } => {
                        self.state.focus_event(false)
                    }
                    _ => {}
                }
            }


            // TODO: Currently, logic and display are locked
            // together to the same framerate; we should probably
            // change that.
            // How does Love2D do it though?
            // Love2D does it the simple and dumb way and just does
            // sleep(0.001) after each draw; see
            // http://www.love2d.org/wiki/love.run
            let dt = timer::get_delta(ctx);
            self.state.update(ctx, dt)?;
            self.state.draw(ctx)?;
        }

        Ok(())
    }
}
