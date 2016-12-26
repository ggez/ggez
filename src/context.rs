

use sdl2::{self, Sdl};
use sdl2::render::Renderer;
use sdl2::video::Window;

use sdl2_mixer;
use sdl2_mixer::Sdl2MixerContext;

use std::fmt;
use std::path;

use conf;
use filesystem::Filesystem;
use graphics;
use timer;
use util;
use GameError;
use GameResult;


/// A `Context` is an object that holds on to global resources.
/// It basically tracks hardware state such as the screen, audio
/// system, timers, and so on.  Generally this type is **not** thread-
/// safe and only one `Context` can exist at a time.  Trying to create
/// another one will fail.  In normal usage you don't have to worry
/// about this because it gets created and managed by the `Game` object,
/// and is handed to your `GameState` for use in drawing and such.
///
/// Most functions that interact with the hardware, for instance
/// drawing things, playing sounds, or loading resources (which then
/// need to be transformed into a format the hardware likes) will need
/// to access the `Context`.
pub struct Context<'a> {
    pub sdl_context: Sdl,
    pub mixer_context: Sdl2MixerContext,
    pub renderer: Renderer<'a>,
    pub filesystem: Filesystem,
    pub gfx_context: graphics::GraphicsContext,
    pub event_context: sdl2::EventSubsystem,
    pub timer_context: timer::TimeContext,
    pub dpi: (f32, f32, f32),
    _audio_context: sdl2::AudioSubsystem,
}

impl<'a> fmt::Debug for Context<'a> {
    // TODO: Make this more useful.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Context: {:p}>", self)
    }
}



fn init_audio(sdl_context: &Sdl) -> GameResult<sdl2::AudioSubsystem> {
    sdl_context.audio()
        .map_err(GameError::AudioError)
}

fn init_mixer() -> GameResult<Sdl2MixerContext> {
    let frequency = 44100;
    let format = sdl2_mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = 2; // Stereo
    let chunk_size = 1024;
    try!(sdl2_mixer::open_audio(frequency, format, channels, chunk_size));

    let flags = sdl2_mixer::InitFlag::all();
    sdl2_mixer::init(flags).map_err(GameError::AudioError)
}

fn init_window(video: sdl2::VideoSubsystem,
               window_title: &str,
               screen_width: u32,
               screen_height: u32)
               -> GameResult<Window> {

    // Can't hurt
    let _ = sdl2::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "best");

    video.window(window_title, screen_width, screen_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| GameError::VideoError(format!("{}", e)))
}

/// Sets the window icon from the Conf window_icon field.
/// Assumes an empty string in the conf's window_icon
/// means to do nothing.
fn set_window_icon(context: &mut Context, conf: &conf::Conf) -> GameResult<()> {
    if !conf.window_icon.is_empty() {
        let path = path::Path::new(&conf.window_icon);
        let icon_surface = try!(util::load_surface(context, path));

        if let Some(window) = context.renderer.window_mut() {
            window.set_icon(icon_surface);
        }
    };
    Ok(())
}

impl<'a> Context<'a> {
    /// Tries to create a new Context using settings from the given config file.
    /// Usually called by the engine as part of the set-up code.
    pub fn from_conf(conf: &conf::Conf,
                     fs: Filesystem,
                     sdl_context: Sdl)
                     -> GameResult<Context<'a>> {
        let window_title = &conf.window_title;
        let screen_width = conf.window_width;
        let screen_height = conf.window_height;

        let video = try!(sdl_context.video());
        let window = try!(init_window(video, &window_title, screen_width, screen_height));
        let display_index = try!(window.display_index());
        let dpi = try!(window.subsystem().display_dpi(display_index));

        let renderer = try!(window.renderer()
            .accelerated()
            .build());

        let audio_context = try!(init_audio(&sdl_context));
        let mixer_context = try!(init_mixer());
        let event_context = try!(sdl_context.event());
        let timer_context = timer::TimeContext::new();

        let mut ctx = Context {
            sdl_context: sdl_context,
            mixer_context: mixer_context,
            renderer: renderer,
            filesystem: fs,
            gfx_context: graphics::GraphicsContext::new(),
            dpi: dpi,

            event_context: event_context,
            timer_context: timer_context,

            _audio_context: audio_context,
        };

        try!(set_window_icon(&mut ctx, conf));

        Ok(ctx)
    }


    /// Prints out information on the sound subsystem.
    pub fn print_sound_stats(&self) {
        println!("Allocated {} sound channels",
                 sdl2_mixer::allocate_channels(-1));
        let n = sdl2_mixer::get_chunk_decoders_number();
        println!("available chunk(sample) decoders: {}", n);

        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2_mixer::get_chunk_decoder(i));
        }

        let n = sdl2_mixer::get_music_decoders_number();
        println!("available music decoders: {}", n);
        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2_mixer::get_music_decoder(i));
        }
        println!("query spec => {:?}", sdl2_mixer::query_spec());
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
