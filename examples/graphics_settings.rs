extern crate ggez;
extern crate clap;
use clap::{Arg, App};
use ggez::*;
use ggez::graphics::{DrawMode, Point2, Drawable};
use ggez::event::{Keycode, Mod};

enum WindowToggle {
    NONE,
    FORWARD,
    REVERSE
}

struct WindowSettings {
    window_size_toggle: WindowToggle,
    toggle_fullscreen: bool,
    is_fullscreen: bool,
    num_of_resolutions: usize,
    resolution_index: usize,
}

struct MainState {
    angle: f32, // in radians
    window_settings: WindowSettings,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState { 
            angle: 0.0,
            window_settings: WindowSettings {
                toggle_fullscreen: false,
                window_size_toggle: WindowToggle::NONE,
                is_fullscreen: false,
                resolution_index: 0,
                num_of_resolutions: 0,
            }
        };


        let resolutions = ggez::graphics::get_fullscreen_modes(_ctx, 0)?;
        s.window_settings.num_of_resolutions = resolutions.len();

        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.angle = self.angle + 0.01;

            if self.window_settings.toggle_fullscreen {
                ggez::graphics::set_fullscreen(ctx, self.window_settings.is_fullscreen)?;
                self.window_settings.toggle_fullscreen = false;
            }

            match self.window_settings.window_size_toggle {
                WindowToggle::FORWARD | WindowToggle::REVERSE => {
                    let resolutions = ggez::graphics::get_fullscreen_modes(ctx, 0)?;
                    let (width, height) = resolutions[self.window_settings.resolution_index];

                    ggez::graphics::set_resolution(ctx, width, height)?;

                    self.window_settings.window_size_toggle = WindowToggle::NONE;
                }
                _ => {},
            }
      }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_background_color(ctx, graphics::BLACK);
        graphics::clear(ctx);
        let rotation = timer::get_ticks(ctx) % 1000;
        let circle = graphics::Mesh::new_circle(
                        ctx,
                         DrawMode::Line(3.0),
                         Point2::new(0.0, 0.0),
                         100.0,
                         4.0)?;
        graphics::draw(ctx, &circle, Point2::new(400.0, 300.0), rotation as f32)?;
        graphics::present(ctx);
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: Mod, repeat: bool) {

        if !repeat {
            match keycode {
                Keycode::F => {
                    self.window_settings.toggle_fullscreen = true;
                    self.window_settings.is_fullscreen = !self.window_settings.is_fullscreen;
                }
                Keycode::H => {
                    self.window_settings.window_size_toggle = WindowToggle::FORWARD;
                    self.window_settings.resolution_index += 1;
                    self.window_settings.resolution_index %= self.window_settings.num_of_resolutions;
                }
                Keycode::G => {
                    if self.window_settings.resolution_index > 0 {
                        self.window_settings.window_size_toggle = WindowToggle::REVERSE;
                        self.window_settings.resolution_index -= 1;
                        self.window_settings.resolution_index %= self.window_settings.num_of_resolutions;
                    }
                }
                _ => {},
            }
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: u32, height: u32) {
            println!("Resized screen to {}, {}", width, height);
            // BUGGO: Should be able to return an actual error here!
            let new_rect = graphics::Rect::new(
                (width/2) as f32,
                (height/2) as f32,
                width as f32,
                -(height as f32),
            );
            graphics::set_screen_coordinates(ctx, new_rect).unwrap();
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

    let msaa: u32 = matches
        .value_of("msaa")
        .unwrap_or("1")
        .parse()
        .expect("Option msaa needs to be a number!");
    let mut c = conf::Conf::new();
    c.window_mode.samples =
        conf::NumSamples::from_u32(msaa).expect("Option msaa needs to be 1, 2, 4, 8 or 16!");
    c.window_mode.resizable = true;
    // c.window_mode.min_height = 50;
    // c.window_mode.max_height = 5000;
    // c.window_mode.min_width = 50;
    // c.window_mode.max_width = 5000;
    let ctx = &mut Context::load_from_conf("graphics_settings", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
