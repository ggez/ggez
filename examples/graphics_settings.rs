//! An example of how to play with various graphics modes settings,
//! resize windows, etc.
//!
//! Prints instructions to the console.
use std::convert::TryFrom;

use ggez::conf;
use ggez::event;
use ggez::graphics::Rect;
use ggez::graphics::{self, Color, DrawMode, DrawParam};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, GameResult};

use argh::FromArgs;

use ggez::input::keyboard::KeyInput;
use std::env;
use std::path;

type Point2 = ggez::glam::Vec2;

struct WindowSettings {
    toggle_fullscreen: bool,
    is_fullscreen: bool,
    resize_projection: bool,
}

struct MainState {
    angle: f32, // in radians
    zoom: f32,
    image: graphics::Image,
    window_settings: WindowSettings,
    screen_coords: Rect,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            angle: 0.0,
            zoom: 1.0,
            image: graphics::Image::from_path(ctx, "/tile.png")?,
            window_settings: WindowSettings {
                toggle_fullscreen: false,
                is_fullscreen: false,
                resize_projection: false,
            },
            screen_coords: Rect {
                x: 0.,
                y: 0.,
                w: ctx.gfx.drawable_size().0,
                h: ctx.gfx.drawable_size().1,
            },
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            self.angle += 0.01;

            if self.window_settings.toggle_fullscreen {
                let fullscreen_type = if self.window_settings.is_fullscreen {
                    conf::FullscreenType::Desktop
                } else {
                    conf::FullscreenType::Windowed
                };
                ctx.gfx.set_fullscreen(fullscreen_type)?;
                self.window_settings.toggle_fullscreen = false;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        canvas.set_screen_coordinates(self.screen_coords);

        canvas.draw(
            &self.image,
            DrawParam::new()
                .dest(Point2::new(400.0, 300.0))
                .color(Color::WHITE), //.offset([0.5, 0.5]),
        );

        let rotation = ctx.time.ticks() % 1000;
        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::stroke(3.0),
            Point2::new(0.0, 0.0),
            100.0,
            4.0,
            Color::WHITE,
        )?;
        canvas.draw(
            &circle,
            DrawParam::new()
                .dest(Point2::new(400.0, 300.0))
                .rotation(rotation as f32)
                .color(Color::WHITE),
        );

        // Let's draw a grid so we can see where the window bounds are.
        const COUNT: i32 = 10;
        let mut mb = graphics::MeshBuilder::new();
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
                // graphics::rectangle(
                //     ctx,
                //     color,
                //     graphics::DrawMode::fill(),
                //     graphics::Rect::new(fx, fy, 5.0, 5.0),
                // )?
                mb.rectangle(
                    DrawMode::fill(),
                    graphics::Rect::new(fx, fy, 5.0, 5.0),
                    color,
                )?;
            }
        }
        let mesh = graphics::Mesh::from_data(ctx, mb.build());
        canvas.draw(&mesh, ggez::mint::Point2 { x: 0.0, y: 0.0 });

        canvas.finish(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _btn: event::MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        println!("Button clicked at: {x} {y}");
        Ok(())
    }

    fn key_up_event(&mut self, ctx: &mut Context, input: KeyInput) -> GameResult {
        match input.keycode {
            Some(KeyCode::F) => {
                self.window_settings.toggle_fullscreen = true;
                self.window_settings.is_fullscreen = !self.window_settings.is_fullscreen;
            }
            Some(KeyCode::Up) => {
                self.zoom += 0.1;
                println!("Zoom is now {}", self.zoom);
                let (w, h) = ctx.gfx.drawable_size();
                let new_rect = graphics::Rect::new(0.0, 0.0, w * self.zoom, h * self.zoom);
                self.screen_coords = new_rect;
            }
            Some(KeyCode::Down) => {
                self.zoom -= 0.1;
                println!("Zoom is now {}", self.zoom);
                let (w, h) = ctx.gfx.drawable_size();
                let new_rect = graphics::Rect::new(0.0, 0.0, w * self.zoom, h * self.zoom);
                self.screen_coords = new_rect;
            }
            Some(KeyCode::Space) => {
                self.window_settings.resize_projection = !self.window_settings.resize_projection;
                println!(
                    "Resizing the projection on window resize is now: {}",
                    self.window_settings.resize_projection
                );
            }
            _ => {}
        }
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        println!("Resized screen to {width}, {height}");
        if self.window_settings.resize_projection {
            let new_rect = graphics::Rect::new(0.0, 0.0, width * self.zoom, height * self.zoom);
            self.screen_coords = new_rect;
        }
        Ok(())
    }
}

fn print_help() {
    println!("GRAPHICS SETTING EXAMPLE:");
    println!("    F: toggle fullscreen");
    println!("    Up/Down: Zoom in/out");
    println!(
        "    Spacebar: Toggle whether or not to resize the projection when the window is resized"
    );
    println!("    ");
    println!("    To see command-line options, run with `cargo run --example graphics_settings -- --help`");
    println!("    ");
}

/// Print out graphics settings.
#[derive(FromArgs, Debug)]
struct Opt {
    /// what level of MSAA to try to use (1 or 4)
    #[argh(option, short = 'm', long = "msaa", default = "1")]
    msaa: u8,
}

pub fn main() -> GameResult {
    let opt: Opt = argh::from_env();

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let backend = conf::Backend::default();

    let cb = ggez::ContextBuilder::new("graphics_settings", "ggez")
        .window_mode(
            conf::WindowMode::default()
                .fullscreen_type(conf::FullscreenType::Windowed)
                .resizable(true),
        )
        .window_setup(conf::WindowSetup::default().samples(
            conf::NumSamples::try_from(opt.msaa).expect("Option msaa needs to be 1 or 4!"),
        ))
        .backend(backend)
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;

    print_help();
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
