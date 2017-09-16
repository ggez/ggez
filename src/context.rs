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
    pub gfx_context: graphics::GraphicsContext,
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
    // TODO: Make this include more information?
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Context: {:p}>", self)
    }
}

/// Sets the window icon from the Conf `window_icon` field.
/// An empty string in the conf's `window_icon`
/// means to do nothing.
fn set_window_icon(context: &mut Context) -> GameResult<()> {
    if !context.conf.window_icon.is_empty() {
        let icon_path = &context.conf.window_icon;
        let mut f = context.filesystem.open(icon_path)?;
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
        let window = context.gfx_context.get_window_mut();
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
        let graphics_context = graphics::GraphicsContext::new(video,
                                                              &conf.window_title,
                                                              conf.window_width,
                                                              conf.window_height,
                                                              conf.window_mode)?;
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

    /// Tries to create a new Context loading a config
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
        if let Err(e) = self.filesystem.print_all() {
            println!("Error printing out filesystem info: {}", e)
        }
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
