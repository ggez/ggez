use state::State;
use context::Context;
use resources::{ResourceManager, TextureManager};
use GameError;
use warn;
use conf;
use filesystem as fs;

use std::path::Path;
use std::thread;
use std::option;
use std::io::Read;
use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::event::*;
use sdl2::keyboard::Keycode::*;
use sdl2::surface::Surface;

use rand::{self, Rand};



#[derive(Debug)]
pub struct Game<'a, S: State> {
    conf: conf::Conf,
    states: Vec<S>,
    context: Option<Context<'a>>,
}

impl<'a, S: State> Game<'a, S> {
    pub fn new(config: conf::Conf, initial_state: S) -> Game<'a, S> {
        // TODO: Verify config version == this version
        Game {
            conf: config,
            states: vec![initial_state],
            context: None,
        }
    }

    /// Looks for a file named "conf.toml" in the resources directory
    /// loads it if it finds it.
    /// If it can't read it for some reason, returns an error.
    /// (Probably best used with `.or(some_default)`)
    pub fn from_config_file(initial_state: S) -> Result<Game<'a, S>, GameError> {
        let fs = fs::Filesystem::new();
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

    // TODO: If we're going to have a real stack of states here,
    // (I really prefer the name scenes though,)
    // we should give them enter() and leave() events as well
    // as load()
    pub fn push_state(&mut self, state: S) {
        self.states.push(state);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    fn get_active_state(&mut self) -> Option<&mut S> {
        self.states.last_mut()
    }


    pub fn run(&mut self) -> Result<(), GameError> {
        // TODO: Window icon
        // TODO: Module init should all happen in the Context
        let mut ctx = try!(Context::new(&self.conf.window_title,
                                        self.conf.window_width,
                                        self.conf.window_height));

        self.context = Some(ctx);
        // self.init_sound_system().or_else(warn);
        let mut ctx = self.context.take().unwrap();
        let mut timer = try!(ctx.sdl_context.timer());
        let mut event_pump = try!(ctx.sdl_context.event_pump());

        // If the example text is too big for the screen, downscale it (and center irregardless)
        let padding = 64;

        // Initialize State handlers
        for s in &mut self.states {
            s.load(&mut ctx);
        }

        let mut delta = Duration::new(0, 0);
        let mut done = false;
        while !done {
            let start_time = timer.ticks();

            if let Some(active_state) = self.get_active_state() {
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
                                _ => active_state.key_down_event(event),
                            }
                        }
                        KeyUp { .. } => active_state.key_up_event(event),
                        MouseButtonDown { .. } => active_state.mouse_button_down_event(event),
                        MouseButtonUp { .. } => active_state.mouse_button_up_event(event),
                        MouseMotion { .. } => active_state.mouse_motion_event(event),
                        MouseWheel { .. } => active_state.mouse_wheel_event(event),
                        Window { win_event_id: WindowEventId::FocusGained, .. } => {
                            active_state.focus(true)
                        }
                        Window { win_event_id: WindowEventId::FocusLost, .. } => {
                            active_state.focus(false)
                        }
                        _ => {}
                    }
                }

                active_state.update(&mut ctx, delta);

                // ctx.renderer.set_draw_color(Color::rand(&mut rng));
                // ctx.renderer.clear();
                active_state.draw(&mut ctx);
                // ctx.renderer.present();
            } else {
                done = true;
            }

            let end_time = timer.ticks();
            delta = Duration::from_millis((end_time - start_time) as u64);
            thread::sleep_ms(1000 / 60);
        }

        self.context = Some(ctx);
        if let Some(active_state) = self.get_active_state() {
            active_state.quit();
        }
        Ok(())
    }
}

