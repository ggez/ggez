//! An example of how to play with various graphics modes settings,
//! resize windows, etc.
//!
//! Prints instructions to the console.

extern crate ggez;
extern crate clap;
use clap::{Arg, App};
use ggez::*;
use ggez::graphics::{DrawMode, Point2};
use ggez::event::{Keycode, Mod};

use std::env;
use std::path;

enum WindowToggle {
    NONE,
    FORWARD,
    REVERSE,
}

struct WindowSettings {
    window_size_toggle: WindowToggle,
    toggle_fullscreen: bool,
    is_fullscreen: bool,
    num_of_resolutions: usize,
    resolution_index: usize,
    resize_projection: bool,
}

struct MainState {
    angle: f32, // in radians
    zoom: f32,
    image: graphics::Image,
    window_settings: WindowSettings,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            angle: 0.0,
            zoom: 1.0,
            image: graphics::Image::new(ctx, "/tile.png")?,
            window_settings: WindowSettings {
                toggle_fullscreen: false,
                window_size_toggle: WindowToggle::NONE,
                is_fullscreen: false,
                resolution_index: 0,
                num_of_resolutions: 0,
                resize_projection: false,
            },
        };


        let resolutions = ggez::graphics::get_fullscreen_modes(ctx, 0)?;
        s.window_settings.num_of_resolutions = resolutions.len();

        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.angle += 0.01;

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
                _ => {}
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_background_color(ctx, graphics::BLACK);
        graphics::clear(ctx);
        let rotation = timer::get_ticks(ctx) % 1000;
        graphics::set_color(ctx, graphics::WHITE)?;
        let circle = graphics::Mesh::new_circle(ctx,
                                                DrawMode::Line(3.0),
                                                Point2::new(0.0, 0.0),
                                                100.0,
                                                4.0)?;
        graphics::draw(ctx, &self.image, Point2::new(400.0, 300.0), 0.0)?;
        graphics::draw(ctx, &circle, Point2::new(400.0, 300.0), rotation as f32)?;

        // Let's draw a grid so we can see where the window bounds are.
        const COUNT: i32 = 10;
        for x in -COUNT..COUNT {
            for y in -COUNT..COUNT {
                const SPACING: i32 = 100;
                let fx = (x * SPACING) as f32;
                let fy = (y * SPACING) as f32;
                // println!("POS: {},{}", fx, fy);
                let r = (x as f32) / (COUNT as f32);
                let b = (y as f32) / (COUNT as f32);
                // println!("R: {}", r);
                let color = graphics::Color::new(r, 0.0, b, 1.0);
                graphics::set_color(ctx, color)?;
                graphics::rectangle(ctx, graphics::DrawMode::Fill, graphics::Rect::new(fx, fy, 5.0, 5.0))?
            }
        }
        graphics::present(ctx);
        Ok(())
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, repeat: bool) {
        if !repeat {
            match keycode {
                Keycode::F => {
                    self.window_settings.toggle_fullscreen = true;
                    self.window_settings.is_fullscreen = !self.window_settings.is_fullscreen;
                }
                Keycode::H => {
                    self.window_settings.window_size_toggle = WindowToggle::FORWARD;
                    self.window_settings.resolution_index += 1;
                    self.window_settings.resolution_index %=
                        self.window_settings.num_of_resolutions;
                }
                Keycode::G => {
                    if self.window_settings.resolution_index > 0 {
                        self.window_settings.window_size_toggle = WindowToggle::REVERSE;
                        self.window_settings.resolution_index -= 1;
                        self.window_settings.resolution_index %=
                            self.window_settings.num_of_resolutions;
                    }
                },
                Keycode::Up => {
                    self.zoom += 0.1;
                    println!("Zoom is now {}", self.zoom);
                    let (w, h) = graphics::get_size(ctx);
                    let new_rect = graphics::Rect::new(0.0,
                                                    0.0,
                                                    w as f32 * self.zoom,
                                                    h as f32 * self.zoom);
                    graphics::set_screen_coordinates(ctx, new_rect).unwrap();

                },
                Keycode::Down => {
                    self.zoom -= 0.1;
                    println!("Zoom is now {}", self.zoom);
                    let (w, h) = graphics::get_size(ctx);
                    let new_rect = graphics::Rect::new(0.0,
                                                    0.0,
                                                    w as f32 * self.zoom,
                                                    h as f32 * self.zoom);
                    graphics::set_screen_coordinates(ctx, new_rect).unwrap();
                },
                Keycode::Space => {
                    self.window_settings.resize_projection = !self.window_settings.resize_projection;
                    println!("Resizing the projection on window resize is now: {}", self.window_settings.resize_projection);
                }
                _ => {}
            }
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: u32, height: u32) {
        println!("Resized screen to {}, {}", width, height);
        if self.window_settings.resize_projection {
            let new_rect = graphics::Rect::new(0.0,
                                            0.0,
                                            width as f32 * self.zoom,
                                            height as f32 * self.zoom);
            graphics::set_screen_coordinates(ctx, new_rect).unwrap();
        }
    }
}

fn print_help() {
    println!("GRAPHICS SETTING EXAMPLE:");
    println!("    F: toggle fullscreen");
    println!("    H/G: Increase/decrease window sizes");
    println!("    Up/Down: Zoom in/out");
    println!("    Spacebar: Toggle whether or not to resize the projection when the window is resized");
    println!("    ");
    println!("    To see command-line options, run with `cargo run --example graphics_settings -- --help`");
    println!("    ");
}

pub fn main() {
    let matches = App::new("ggez graphics settings example")
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
    c.window_setup.samples =
        conf::NumSamples::from_u32(msaa).expect("Option msaa needs to be 1, 2, 4, 8 or 16!");
    c.window_setup.resizable = true;

    let ctx = &mut Context::load_from_conf("graphics_settings", "ggez", c).unwrap();


    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so
    // we we look in the cargo project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
        println!("Adding path {:?}", path);
    } else {
        println!("not building with cargo?");
    }

    print_help();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
