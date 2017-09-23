extern crate ggez;
extern crate clap;
use clap::{Arg, App};
use ggez::*;
use ggez::graphics::{DrawMode, Point2, Drawable};
use ggez::event::{Keycode, Mod};
use std::time::Duration;

const MAX_WIN_IDX : usize = 5;
const MIN_WIN_IDX : usize = 0;

struct WindowSettings {
    window_size_idx: usize,
    window_size_toggle: bool,
    toggle_fullscreen: bool,
    window_sizes: [(u16, u16); MAX_WIN_IDX],
}

struct MainState {
    pos_x: f32,
    angle: f32, // in radians
    window_settings: WindowSettings,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { 
            pos_x: 0.0,
            angle: 0.0,
            window_settings: WindowSettings {
                window_size_idx: 0,
                toggle_fullscreen: false,
                window_size_toggle: false,
                window_sizes: [(640, 480), (800, 600), (1024, 768), (1280, 800), (1920, 1080)],
            }
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        self.angle = self.angle + 0.01;

        if self.window_settings.toggle_fullscreen {
            // toggle fullscreen here
            self.window_settings.toggle_fullscreen = false;
        }

        if self.window_settings.window_size_toggle {
            // update window resolution here
            self.window_settings.window_size_toggle = false;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let rot_circle = graphics::Mesh::new_circle(ctx, DrawMode::Line(3.0), Point2::new(0.0, 0.0), 100.0, 4.0)?;
        rot_circle.draw(ctx, Point2::new(400.0, 300.0), self.angle)?;
        
        graphics::present(ctx);
        Ok(())
    }

    fn key_up_event(&mut self, keycode: Keycode, _keymod: Mod, repeat: bool) {

        if !repeat {
            match keycode {
                Keycode::F => self.window_settings.toggle_fullscreen = true,
                Keycode::H => if self.window_settings.window_size_idx < MAX_WIN_IDX {
                    self.window_settings.window_size_idx = self.window_settings.window_size_idx + 1;
                    self.window_settings.window_size_toggle = true;
                },
                Keycode::G => if self.window_settings.window_size_idx > MIN_WIN_IDX {
                    self.window_settings.window_size_idx = self.window_settings.window_size_idx - 1;
                    self.window_settings.window_size_toggle = true;
                },
                _ => unimplemented!(),
            }
        }
    }
}

pub fn main() {
    let matches = App::new("graphics settings example")
        .arg(Arg::with_name("msaa")
            .short("m")
            .value_name("N")
            .help("Number of MSAA samples to do (powers of 2 from 1 to 16)")
            .takes_value(true))
        .get_matches();
    
    let msaa: u32 = matches.value_of("msaa")
        .unwrap_or("1")
        .parse()
        .expect("Option msaa needs to be a number!");
    let mut c = conf::Conf::new();
    c.window_mode.samples = conf::NumSamples::from_u32(msaa)
        .expect("Option msaa needs to be 1, 2, 4, 8 or 16!");
    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
