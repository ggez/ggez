use sdl2::{self, Sdl};
use sdl2::surface;
use sdl2::pixels;
use image::{self, GenericImage};

use std::fmt;
use std::io::Read;

use audio;
use conf;
use filesystem::Filesystem;
use graphics;
use input;
use timer;
use GameError;
use GameResult;


/// A `Context` is an object that holds on to global resources.
/// It basically tracks hardware state such as the screen, audio
/// system, timers, and so on.  Generally this type is **not** thread-
/// safe and only one `Context` can exist at a time.  Trying to create
/// another one will fail.
///
/// Most functions that interact with the hardware, for instance
/// drawing things, playing sounds, or loading resources (which then
/// need to be transformed into a format the hardware likes) will need
/// to access the `Context`.
pub struct Context {
    /// The Conf object the Context was created with
    pub conf: conf::Conf,
    /// SDL context
    pub sdl_context: Sdl,
    /// Filesystem state
    pub filesystem: Filesystem,
    /// Graphics state
    pub(crate) gfx_context: graphics::GraphicsContext,
    /// Event context
    pub event_context: sdl2::EventSubsystem,
    /// Timer state
    pub timer_context: timer::TimeContext,
    /// Audio context
    pub audio_context: audio::AudioContext,
    /// Gamepad context
    pub gamepad_context: input::GamepadContext,
    /// Default font
    pub default_font: graphics::Font,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Context: {:p}>", self)
    }
}

/// Sets the window icon from the Conf `window_icon` field.
/// An empty string in the conf's `window_icon`
/// means to do nothing.
fn set_window_icon(context: &mut Context) -> GameResult<()> {
    // This clone is a little annoying, but, borrowing is inconvenient.
    let icon = &context.conf.window_setup.icon.clone();
    if !icon.is_empty() {
        let mut f = context.filesystem.open(icon)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let image = image::load_from_memory(&buf)?;
        let image_data = &mut image.to_rgba();
        // The "pitch" parameter here is not the count
        // between pixels, but the count between rows.
        // For some retarded reason.
        // Also SDL seems to have strange ideas of what
        // "RGBA" means.
        let surface = surface::Surface::from_data(image_data,
                                                  image.width(),
                                                  image.height(),
                                                  image.width() * 4,
                                                  pixels::PixelFormatEnum::ABGR8888)?;
        let window = graphics::get_window_mut(context);
        window.set_icon(surface);
    };
    Ok(())
}

impl Context {
    /// Tries to create a new Context using settings from the given config file.
    /// Usually called by `Context::load_from_conf()`.
    fn from_conf(conf: conf::Conf, fs: Filesystem, sdl_context: Sdl) -> GameResult<Context> {
        let video = sdl_context.video()?;

        let audio_context = audio::AudioContext::new()?;
        let event_context = sdl_context.event()?;
        let timer_context = timer::TimeContext::new();
        let font = graphics::Font::default_font()?;
        let backend_spec = graphics::GlBackendSpec::from(conf.backend);
        let graphics_context = graphics::GraphicsContext::new(video,
                                                              &conf.window_setup,
                                                              conf.window_mode,
                                                              backend_spec)?;
        let gamepad_context = input::GamepadContext::new(&sdl_context)?;

        let mut ctx = Context {
            conf: conf,
            sdl_context: sdl_context,
            filesystem: fs,
            gfx_context: graphics_context,
            event_context: event_context,
            timer_context: timer_context,
            audio_context: audio_context,
            gamepad_context: gamepad_context,

            default_font: font,
        };

        set_window_icon(&mut ctx)?;

        Ok(ctx)
    }

    /// Tries to create a new Context by loading a config
    /// file from its default path, using the given `Conf`
    /// object as a default if none is found.
    ///
    /// The `game_id` and `author` are game-specific strings that
    /// are used to locate the default storage locations for the
    /// platform it looks in, as documented in the `filesystem`
    /// module.  You can also always debug-print the
    /// `Context::filesystem` field to see what paths it is
    /// searching.
    pub fn load_from_conf(game_id: &'static str,
                          author: &'static str,
                          default_config: conf::Conf)
                          -> GameResult<Context> {

        let sdl_context = sdl2::init()?;
        let mut fs = Filesystem::new(game_id, author)?;

        let config = fs.read_config().unwrap_or(default_config);

        Context::from_conf(config, fs, sdl_context)
    }

    /// Prints out information on the resources subsystem.
    pub fn print_resource_stats(&mut self) {
        self.filesystem.print_all();
    }

    /// Triggers a Quit event.
    pub fn quit(&mut self) -> GameResult<()> {
        let now_dur = timer::get_time_since_start(self);
        let now = timer::duration_to_f64(now_dur);
        let e = sdl2::event::Event::Quit { timestamp: now as u32 };
        // println!("Pushing event {:?}", e);
        self.event_context
            .push_event(e)
            .map_err(GameError::from)
    }
}

use std::path;

/// A builder object for creating a context.
///
/// Can do everything the `Context::load_from_conf()` method does, plus you can
/// also specify new paths to add to the resource path list at build time instead
/// of using `filesystem::mount()`.
#[derive(Debug)]
pub struct ContextBuilder {
    game_id: &'static str,
    author: &'static str,
    conf: conf::Conf,
    paths: Vec<path::PathBuf>,
    load_conf_file: bool,
}

impl ContextBuilder {
    /// Create a new ContextBuilder
    pub fn new(game_id: &'static str, author: &'static str) -> Self {
        Self {
            game_id: game_id,
            author: author,
            conf: conf::Conf::default(),
            paths: vec![],
            load_conf_file: true,
        }
    }

    /// Sets the window setup settings
    pub fn window_setup(mut self, setup: conf::WindowSetup) -> Self {
        self.conf.window_setup = setup;
        self
    }

    /// Sets the window mode settings
    pub fn window_mode(mut self, mode: conf::WindowMode) -> Self {
        self.conf.window_mode = mode;
        self
    }

    /// Sets the graphics backend
    pub fn backend(mut self, backend: conf::Backend) -> Self {
        self.conf.backend = backend;
        self
    }


    /// Add a new read-only filesystem path to the places to search
    /// for resources.
    pub fn add_resource_path<T>(mut self, path: T) -> Self
        where T: Into<path::PathBuf>
    {
        self.paths.push(path.into());
        self
    }

    /// Specifies whether or not to load the `conf.toml` file if it
    /// exists and use its settings to override the provided values.
    /// Defaults to `true` which is usually what you want, but being
    /// able to fiddle with it is sometimes useful for debugging.
    pub fn with_conf_file(mut self, load_conf_file: bool) -> Self {
        self.load_conf_file = load_conf_file;
        self
    }

    /// Build the Context.
    pub fn build(self) -> GameResult<Context> {
        let sdl_context = sdl2::init()?;
        let mut fs = Filesystem::new(self.game_id, self.author)?;

        let config = if self.load_conf_file {
            fs.read_config().unwrap_or(self.conf)
        } else {
            self.conf
        };

        for path in &self.paths {
            fs.mount(path, true);
        }

        Context::from_conf(config, fs, sdl_context)
    }
}
