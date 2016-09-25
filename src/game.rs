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

use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::event::*;
use sdl2::keyboard::Keycode::*;




#[derive(Debug)]
pub struct Game<'a, S: State> {
    conf: conf::Conf,
    state: S,
    context: Option<Context<'a>>,
}

impl<'a, S: State> Game<'a, S> {
    pub fn new(config: conf::Conf, initial_state: S) -> Game<'a, S> {
        // TODO: Verify config version == this version
        Game {
            conf: config,
            state: initial_state,
            context: None,
        }
    }

    /// Looks for a file named "conf.toml" in the resources directory
    /// loads it if it finds it.
    /// If it can't read it for some reason, returns an error.
    /// (Probably best used with `.or(some_default)`)
    pub fn from_config_file(initial_state: S) -> GameResult<Game<'a, S>> {
        let mut fs = try!(fs::Filesystem::new());
        let conf_path = Path::new("conf.toml");
        if fs.is_file(conf_path) {
            let mut file = try!(fs.open(conf_path));
            let c = try!(conf::Conf::from_toml_file(&mut file));
            Ok(Game::new(c, initial_state))

        } else {
            let msg = String::from("Config file 'conf.toml' not found");
            let err = GameError::ResourceNotFound(msg);
            Err(err)
        }
    }

    pub fn run(&mut self) -> GameResult<()> {
        // TODO: Window icon
        // TODO: Module init should all happen in the Context
        let mut ctx = try!(Context::new(&self.conf.window_title,
                                        self.conf.window_width,
                                        self.conf.window_height));

        self.context = Some(ctx);
        // This unwrap should never fail, but having to take() the
        // context out of self is a little wonky.
        let mut ctx = self.context.take().unwrap();
        let mut timer = try!(ctx.sdl_context.timer());
        let mut event_pump = try!(ctx.sdl_context.event_pump());

        // Initialize State handlers
        self.state.load(&mut ctx);

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
            self.state.update(&mut ctx, delta);
            self.state.draw(&mut ctx);

            let end_time = timer.ticks();
            delta = Duration::from_millis((end_time - start_time) as u64);
            thread::sleep_ms(1000 / 60);
        }

        self.state.quit();
        Ok(())
    }
}

