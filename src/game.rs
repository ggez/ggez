//! The Game struct starts up the game and runs the mainloop and such.

use state::State;
use context::Context;
use GameError;
use GameResult;
use warn;
use conf;
use filesystem as fs;

use std::path::Path;
use std::thread;
use std::time::Duration;

use sdl2;
use sdl2::pixels::Color;
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

impl<'a, S: State> Game<'a, S> {
    pub fn new(initial_state: S, default_config: conf::Conf) -> GameResult<Game<'a, S>> {
        let sdl_context = try!(sdl2::init());
        let mut fs = try!(fs::Filesystem::new());

        // TODO: Verify config version == this version
        let config = get_default_config(&mut fs)
            .unwrap_or(default_config);

        let context = try!(Context::from_conf(&config, fs, sdl_context));

        Ok(Game {
            conf: config,
            state: initial_state,
            context: context,
        })
    }

    /// Replaces the gamestate with the given one without
    /// having to re-initialize everything in the Context.
    pub fn replace_state(&mut self, state: S) -> () {
        self.state = state;
    }


    pub fn run(&mut self) -> GameResult<()> {
        // TODO: Window icon
        let ref mut ctx = self.context;
        let mut timer = try!(ctx.sdl_context.timer());
        let mut event_pump = try!(ctx.sdl_context.event_pump());

        self.state.load(ctx);

        let mut delta = Duration::new(0, 0);
        let mut done = false;
        while !done {
            let start_time = timer.ticks();

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
            self.state.update(ctx, delta);
            self.state.draw(ctx);

            let end_time = timer.ticks();
            delta = Duration::from_millis((end_time - start_time) as u64);
            thread::sleep_ms(1000 / 60);
        }

        self.state.quit();
        Ok(())
    }
}

