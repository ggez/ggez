//! The Game struct starts up the game and runs the mainloop and such.

use context::Context;
use GameError;
use GameResult;
use conf;
use filesystem as fs;

use std::cmp;
use std::sync::atomic;
use std::path::Path;
use std::time::Duration;
use std::thread::sleep;

use sdl2;
use sdl2::event::Event::*;
use sdl2::event::*;
use sdl2::keyboard::Keycode::*;


/// A trait for defining a game state.
/// Implement `load()`, `update()` and `draw()` callbacks on this trait
/// and hand it to a `Game` object to be run.
/// You may also implement the `*_event` traits if you wish to handle
/// those events.
///
/// The default event handlers do nothing, apart from `key_down_event()`,
/// which *should* by default exit the game if escape is pressed, but which
/// doesn't do such a thing yet...
pub trait GameState {

    // Tricksy trait and lifetime magic!
    // Much thanks to aatch on #rust-beginners for helping make this work.
    fn load(ctx: &mut Context, conf: &conf::Conf) -> GameResult<Self>
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
    fn key_down_event(&mut self, _evt: Event) {}

    fn key_up_event(&mut self, _evt: Event) {}

    fn focus(&mut self, _gained: bool) {}

    fn quit(&mut self) {
        println!("Quitting game");
    }
}


#[derive(Debug)]
pub struct Game<'a, S: GameState> {
    conf: conf::Conf,
    state: S,
    context: Context<'a>,
}

/// Looks for a file named "conf.toml" in the resources directory
/// loads it if it finds it.
/// If it can't read it for some reason, returns None
fn get_default_config(fs: &mut fs::Filesystem) -> GameResult<conf::Conf> {
    let conf_path = Path::new("conf.toml");
    if fs.is_file(conf_path) {
        let mut file = try!(fs.open(conf_path));
        let c = try!(conf::Conf::from_toml_file(&mut file));
        Ok(c)

    } else {
        Err(GameError::ConfigError(String::from("Config file not found")))
    }
}

impl<'a, S: GameState + 'static> Game<'a, S> {
    /// Creates a new `Game` with the given initial gamestate and
    /// default config (which will be used if there is no config file)
    pub fn new(default_config: conf::Conf) -> GameResult<Game<'a, S>>
        //where T: Fn(&Context, &conf::Conf) -> S
    {
        let sdl_context = try!(sdl2::init());
        let mut fs = try!(fs::Filesystem::new());

        // TODO: Verify config version == this version
        let config = get_default_config(&mut fs)
            .unwrap_or(default_config);

        let mut context = try!(Context::from_conf(&config, fs, sdl_context));

        let init_state = try!(S::load(&mut context, &config));

        Ok(Game {
            conf: config,
            state: init_state,
            context: context,
        })
    }

    /// Re-initializes the game state using the type's `::load()` method.
    pub fn reload_state(&mut self) -> GameResult<()> {
        let newstate = try!(S::load(&mut self.context, &self.conf));
        self.state = newstate;
        Ok(())
    }

    /// Calls the given function to create a new gamestate, and replaces
    /// the current one with it.
    pub fn replace_state_with<F>(&mut self, f: &F) -> GameResult<()>
        where F: Fn(&mut Context, &conf::Conf) -> GameResult<S> {
        let newstate = try!(f(&mut self.context, &self.conf));
        self.state = newstate;
        Ok(())
    }

    /// Replaces the gamestate with the given one without
    /// having to re-initialize everything in the Context.
    pub fn replace_state(&mut self, state: S){
        self.state = state;
    }

    /// Runs the game's mainloop.
    pub fn run(&mut self) -> GameResult<()> {
        // TODO: Window icon
        let ref mut ctx = self.context;
        let mut timer = try!(ctx.sdl_context.timer());
        let mut event_pump = try!(ctx.sdl_context.event_pump());

        let mut delta = 0u64;
        let mut done = false;
        while !done {
            let start_time = timer.ticks() as u64;

            for event in event_pump.poll_iter() {
                //println!("Got event {:?}", event);
                match event {
                    Quit { timestamp: t } => {
                        //println!("Quit event: {:?}", t);
                        done = true
                    }
                    // TODO: We need a good way to have
                    // a default like this, while still allowing
                    // it to be overridden.
                    // But the GameState can't access the Game,
                    // so we can't modify the Game's done property...
                    // Hmmmm.
                    KeyDown { keycode, .. } => {
                        match keycode {
                            Some(Escape) => {
                                ctx.quit();
                        },
                            _ => self.state.key_down_event(event),
                        }
                    }
                    KeyUp { .. } => self.state.key_up_event(event),
                    MouseButtonDown { .. } => self.state.mouse_button_down_event(event),
                    MouseButtonUp { .. } => self.state.mouse_button_up_event(event),
                    MouseMotion { .. } => self.state.mouse_motion_event(event),
                    MouseWheel { .. } => self.state.mouse_wheel_event(event),
                    Window { win_event_id: WindowEventId::FocusGained, .. } => {
                        self.state.focus(true)
                    }
                    Window { win_event_id: WindowEventId::FocusLost, .. } => {
                        self.state.focus(false)
                    }
                    _ => {}
                }
            }
            try!(self.state.update(ctx, Duration::from_millis(delta)));
            try!(self.state.draw(ctx));

            // TODO: For now this is locked at 60 FPS, should fix that.
            // Better FPS stats would also be nice.
            let end_time = timer.ticks() as u64;
            delta = end_time - start_time;
            let desired_frame_time = cmp::max(1000 / 60, delta);
            let sleep_time = Duration::from_millis(desired_frame_time - delta);
            sleep(sleep_time);
        }

        self.state.quit();
        Ok(())
    }
}

