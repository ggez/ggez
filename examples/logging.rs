//! An example showing off `use-log-crate` feature.
//!
//! Uses [`fern`] crate as frontend to [`log`] crate backend.
//! For clarity, only immediately relevant bits are commented.
//!
//! Try running with and without `--features "use-log-crate"` to see effect.
//!
//! [`fern`]: https://docs.rs/fern/0.5.4/fern/
//! [`log`]: https://docs.rs/log/0.4.1/log/

// Importing macros, has to happen at root.
#[macro_use]
extern crate ggez;
#[macro_use]
extern crate log;

// Represents a `ggez`-dependant library utilizing `ggez_*!` logging macros.
mod mini_lib {
    extern crate ggez;
    use ggez::logging;

    // Fairly self-explanatory; refer to `ggez::logging` documentation for additional details.
    pub fn print_some_logs(_x: i32, _y: i32) {
        ggez_info!(
            "I'm a mini_lib info-level log, here are your coords: {}-{}",
            _x,
            _y
        );
        ggez_log!(
            logging::Level::Warn,
            "I'm a mini_lib warning made by `ggez_log!` general macro; coords: {}-{}",
            _x,
            _y
        );
    }
}

// Represents a `ggez`- and `mini_lib`-depending application.
mod application {
    extern crate fern;
    extern crate ggez;
    extern crate log;
    use self::ggez::conf;
    use self::ggez::event;
    use self::ggez::event::MouseButton;
    use self::ggez::{Context, ContextBuilder, GameResult};
    use self::ggez::graphics;
    use std::env;
    use std::io;
    use std::path;

    use mini_lib::print_some_logs;

    struct MainState {
        frames: usize,
    }

    impl MainState {
        fn new(_ctx: &mut Context) -> GameResult<MainState> {
            Ok(MainState { frames: 0 })
        }
    }

    impl event::EventHandler for MainState {
        fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
            graphics::clear(ctx);
            graphics::circle(
                ctx,
                graphics::DrawMode::Fill,
                graphics::Point2::new(200.0, 300.0),
                100.0,
                0.1,
            )?;
            graphics::present(ctx);

            self.frames += 1;
            if (self.frames % 100) == 0 {
                // `log` macro.
                info!("FPS: {}", ggez::timer::get_fps(ctx));
            }

            Ok(())
        }

        fn mouse_button_down_event(
            &mut self,
            _ctx: &mut Context,
            _button: MouseButton,
            _x: i32,
            _y: i32,
        ) {
            // `log` macro.
            info!("Mouse button down: {:?} {}-{}", _button, _x, _y);
            // Calls a function in `mini_lib`.
            print_some_logs(_x, _y);
        }
    }

    pub fn run_app() {
        // `log` macro.
        info!("Initializing!");

        // This creates a `fern` logger that prints `log` crate `Record`s to `stdout`.
        // `gfx_device_gl::factory` spam is filtered out by `level_for()`.
        fern::Dispatch::new()
            .format(|out, msg, rec| {
                out.finish(format_args!(
                    "fern - {} says: {}! {}",
                    rec.target(),
                    rec.level(),
                    msg,
                ))
            })
            .level(log::LevelFilter::Debug)
            .level_for("gfx_device_gl::factory", log::LevelFilter::Warn)
            .chain(io::stdout())
            .apply()
            .unwrap();

        let ctx = &mut ContextBuilder::new("logging", "ggez")
            .window_setup(
                conf::WindowSetup::default()
                    .title("Click me!")
                    .resizable(true),
            )
            .window_mode(conf::WindowMode::default().dimensions(640, 480))
            .build()
            .unwrap();

        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let mut path = path::PathBuf::from(manifest_dir);
            path.push("resources");
            ctx.filesystem.mount(&path, true);
        }

        // `log` macro.
        info!("Running!");

        match MainState::new(ctx) {
            Err(e) => {
                // `log` macro.
                error!("Could not initialize: {}", e);
            }
            Ok(ref mut app) => match event::run(ctx, app) {
                Err(e) => {
                    // `log` macro.
                    error!("Could not exit cleanly: {}", e);
                }
                Ok(_) => {
                    // `log` macro.
                    info!("Exited cleanly.");
                }
            },
        }
    }
}

pub fn main() {
    application::run_app();
}
