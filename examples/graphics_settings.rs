extern crate ggez;
extern crate clap;
use clap::{Arg, App};
use ggez::*;
use ggez::graphics::{DrawMode, Point2, Drawable};
use ggez::event::{Keycode, Mod};
use std::time::Duration;

enum WindowToggle {
    NONE,
    FORWARD,
    REVERSE
}

struct WindowSettings {
    window_size_toggle: WindowToggle,
    toggle_fullscreen: bool,
    is_fullscreen: usize,
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
                toggle_fullscreen: false,
                window_size_toggle: WindowToggle::NONE,
                is_fullscreen : 0,
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
            ggez::graphics::set_fullscreen(_ctx, self.window_settings.is_fullscreen != 0);
            self.window_settings.toggle_fullscreen = false;
        }

        match self.window_settings.window_size_toggle {
            WindowToggle::FORWARD => {
                let resolution = ggez::graphics::get_fullscreen_modes(_ctx, 0);
                self.window_settings.window_size_toggle = WindowToggle::NONE;
            }
            WindowToggle::REVERSE => {

                self.window_settings.window_size_toggle = WindowToggle::NONE;
            }
            _ => {},
        }

       // ggez::graphics::set_mode(ctx, width, height, mode)?;
       // ggez::graphics::set_screen_coordinates();

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
                Keycode::F => {self.window_settings.toggle_fullscreen = true; self.window_settings.is_fullscreen ^= 1;},
                Keycode::H => self.window_settings.window_size_toggle = WindowToggle::FORWARD,
                Keycode::G => self.window_settings.window_size_toggle = WindowToggle::REVERSE,
                _ => {},
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
