//! The Game struct starts up the game and runs the mainloop and such.

use state::State;
use context::Context;
use GameError;
use GameResult;
use conf;
use filesystem as fs;

use std::path::Path;
use std::time::Duration;
use std::thread::sleep;

use sdl2;
use sdl2::event::Event::*;
use sdl2::event::*;
use sdl2::keyboard::Keycode::*;




#[derive(Debug)]
pub struct Game<'a, S: State> {
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

impl<'a, S: State + 'static> Game<'a, S> {
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
                match event {
                    Quit { .. } => done = true,
                    // TODO: We need a good way to have
                    // a default like this, while still allowing
                    // it to be overridden.
                    // But the State can't access the Game,
                    // so we can't modify the Game's done property...
                    // Hmmmm.
                    KeyDown { keycode, .. } => {
                        match keycode {
                            Some(Escape) => done = true,
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
            let desired_frame_time = 1000 / 60;
            let sleep_time = Duration::from_millis(desired_frame_time - delta);
            sleep(sleep_time);
        }

        self.state.quit();
        Ok(())
    }
}

