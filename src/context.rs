

use sdl2::{self, Sdl};
use sdl2::render::Renderer;
use sdl2::video::Window;
use sdl2_ttf;

use sdl2_mixer;
use sdl2_ttf::Sdl2TtfContext;
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
    pub ttf_context: Sdl2TtfContext,
    _audio_context: sdl2::AudioSubsystem,
    pub mixer_context: Sdl2MixerContext,
    pub renderer: Renderer<'a>,
    pub filesystem: Filesystem,
    pub gfx_context: graphics::GraphicsContext,
    pub event_context: sdl2::EventSubsystem,
    pub timer_context: timer::TimeContext,
}

impl<'a> fmt::Debug for Context<'a> {
    // TODO: Make this more useful.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Context: {:p}>", self)
    }
}

fn init_ttf() -> GameResult<Sdl2TtfContext> {
    sdl2_ttf::init().map_err(GameError::from)
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


    // Does not appear to work on my laptop's graphics card,
    // needs more experimentation.
    // let gl_attr = video.gl_attr();
    // //gl_attr.set_context_flags().debug().set();
    // gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    // gl_attr.set_multisample_buffers(1);
    // gl_attr.set_multisample_samples(4);

    // let msaa_buffers = gl_attr.multisample_buffers();
    // let msaa_samples = gl_attr.multisample_samples();

    // println!("buffers: {}, samples {}", msaa_buffers, msaa_samples);

    // Can't hurt
    let _ = sdl2::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "best");
    // let render_quality_hint = sdl2::hint::get("SDL_HINT_RENDER_SCALE_QUALITY");
    // println!("Render quality hint: {:?}", render_quality_hint);

    video.window(window_title, screen_width, screen_height)
         .position_centered()
         .opengl()
         .build()
         .map_err(|e| GameError::VideoError(format!("{}", e)))
}

fn set_window_icon(context: &mut Context, conf: &conf::Conf) -> GameResult<()> {
    if conf.window_icon.len() > 0 {
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
    pub fn from_conf(conf: &conf::Conf,
                     fs: Filesystem,
                     sdl_context: Sdl)
                     -> GameResult<Context<'a>> {
        let window_title = &conf.window_title;
        let screen_width = conf.window_width;
        let screen_height = conf.window_height;

        let video = try!(sdl_context.video());
        let window = try!(init_window(video, &window_title, screen_width, screen_height));


        let renderer = try!(window.renderer()
                                  .accelerated()
                                  .build());

        let ttf_context = try!(init_ttf());
        let audio_context = try!(init_audio(&sdl_context));
        let mixer_context = try!(init_mixer());
        let event_context = try!(sdl_context.event());
        let timer_context = timer::TimeContext::new();

        let mut ctx = Context {
            sdl_context: sdl_context,
            ttf_context: ttf_context,
            _audio_context: audio_context,
            mixer_context: mixer_context,
            renderer: renderer,
            filesystem: fs,
            gfx_context: graphics::GraphicsContext::new(),

            event_context: event_context,
            timer_context: timer_context,
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
    /// BUGGO: This actually doesn't work 'cause
    /// we can't push non-user event types for some reason!
    /// See https://github.com/AngryLawyer/rust-sdl2/issues/530
    /// :-(
    // TODO: Either fix this bug in sdl2 or work around it with
    // a bool in the Context.
    pub fn quit(&mut self) -> GameResult<()> {
        let e = sdl2::event::Event::Quit { timestamp: 10000 };
        println!("Pushing event {:?}", e);
        self.event_context
            .push_event(e)
            .map_err(GameError::from)
    }
}
