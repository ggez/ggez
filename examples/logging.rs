//! This example shows two ways to set up logging in games, using the `log` crate macros with `fern`
//! frontend, to display neatly formatted console output and write same output to a file.
//!
//! Output in question is a trace of app's initialization, and keyboard presses when it's running.
//!
//! `fern` provides a way to write log output to a `std::sync::mpsc::Sender`, so we can use a
//! matching `std::sync::mpsc::Receiver` to get formatted log strings for file output.

use log::*;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::filesystem::{self, File};
use ggez::graphics;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use std::io::Write;
use std::path;
use std::sync::mpsc;

/// A basic file writer.
/// Hogs it's log file until dropped, writes to it whenever `update()` is called.
struct FileLogger {
    /// `ggez`' virtual file representation to write log messages to.
    file: File,
    /// Channel to get log messages from.
    receiver: mpsc::Receiver<String>,
}

impl FileLogger {
    /// Initializes a file writer. Needs an initialized `ggez::Context`, to use it's filesystem.
    fn new(
        ctx: &mut Context,
        path: &str,
        receiver: mpsc::Receiver<String>,
    ) -> GameResult<FileLogger> {
        // This (re)creates a file and opens it for appending.
        let file = filesystem::create(ctx, path::Path::new(path))?;
        debug!(
            "Created log file {:?} in {:?}",
            path,
            filesystem::user_config_dir(ctx)
        );
        Ok(FileLogger { file, receiver })
    }

    /// Reads pending messages from the channel and writes them to the file.
    /// Intended to be called in `EventHandler::update()`, to avoid using threads.
    /// (which you totally shouldn't actively avoid, Rust is perfect for concurrency)
    fn update(&mut self) -> GameResult {
        // try_recv() doesn't block, it returns Err if there's no message to receive.
        while let Ok(msg) = self.receiver.try_recv() {
            // std::io::Write::write_all() takes a byte array.
            self.file.write_all(msg.as_bytes())?;
        }
        Ok(())
    }
}

/// Main state struct. In an actual application, this is where your asset handles, etc go.
struct App {
    /// Owned FileLogger instance; there are multiple ways of going about this, but since we
    /// are not interested in logging to a file anything that happens while app
    /// logic isn't running, this will do.
    file_logger: FileLogger,
}

impl App {
    #[allow(clippy::new_ret_no_self, clippy::unnecessary_wraps)]
    /// Creates an instance, takes ownership of passed FileLogger.
    fn new(_ctx: &mut Context, logger: FileLogger) -> GameResult<App> {
        Ok(App {
            file_logger: logger,
        })
    }
}

/// Where the app meets the `ggez`.
impl EventHandler for App {
    /// This is where the logic should happen.
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        // This tries to throttle updates to desired value.
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // Since we don't have any non-callback logic, all we do is append our logs.
            self.file_logger.update()?;
        }
        Ok(())
    }

    /// Draws the screen. We don't really have anything to draw.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    /// Called when `ggez` catches a keyboard key being pressed.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool,
    ) {
        // Log the keypress to info channel!
        info!(
            "Key down event: {:?}, modifiers: {:?}, repeat: {}",
            keycode, keymod, repeat
        );
        if keycode == KeyCode::Escape {
            // Escape key closes the app.
            ggez::event::quit(ctx);
        }
    }

    /// Called when window is resized.
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        match graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height)) {
            Ok(()) => info!("Resized window to {} x {}", width, height),
            Err(e) => error!("Couldn't resize window: {}", e),
        }
    }
}

pub fn main() -> GameResult {
    // This creates a channel that can be used to asynchronously pass things between parts of the
    // app. There's some overhead, so using it somewhere that doesn't need async (read: threads)
    // is suboptimal. But, `fern`'s arbitrary logging requires a channel.
    let (log_tx, log_rx) = mpsc::channel();

    // `log` is not initialized yet.
    debug!("I will not be logged!");

    // This sets up a `fern` logger and initializes `log`.
    fern::Dispatch::new()
        // Formats logs
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{:<5}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level().to_string(),
                record.target(),
                message
            ))
        })
        // `gfx_device_gl` is very chatty on info loglevel, so
        // filter that a bit more strictly.
        .level_for("gfx_device_gl", log::LevelFilter::Warn)
        .level(log::LevelFilter::Trace)
        // Hooks up console output.
        .chain(std::io::stdout())
        // Hooks up the channel.
        .chain(log_tx)
        .apply()
        .unwrap();

    // Note, even though our file logger hasn't been initialized in any way yet, logs starting
    // from here will still appear in the log file.
    debug!("I am logged!");
    info!("I am too!");

    trace!("Creating ggez context.");

    // This sets up `ggez` guts (including filesystem) and creates a window.
    let (mut ctx, events_loop) = ContextBuilder::new("logging", "ggez")
        .window_setup(WindowSetup::default().title("Pretty console output!"))
        .window_mode(
            WindowMode::default()
                .dimensions(640.0, 480.0)
                .resizable(true),
        )
        .build()?;

    trace!("Context created, creating a file logger.");

    let file_logger = FileLogger::new(&mut ctx, "/out.log", log_rx)?;

    trace!("File logger created, starting loop.");

    // Creates our state, and starts `ggez`' loop.
    match App::new(&mut ctx, file_logger) {
        Err(e) => {
            error!("Could not initialize: {}", e);
        }
        Ok(app) => ggez::event::run(ctx, events_loop, app),
    }

    trace!("Since file logger is dropped with App, this line will cause an error in fern!");
    Ok(())
}
