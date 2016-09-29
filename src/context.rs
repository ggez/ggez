//! A `Context` is an object that holds on to global resources.
//! It basically tracks hardware state such as the screen, audio
//! system, timers, and so on.  Generally this type is **not** thread-
//! safe and only one `Context` can exist at a time.  Trying to create
//! another one will fail.
//!
//! Most functions that interact with the hardware, for instance
//! drawing things, playing sounds, or loading resources (which then
//! need to be transformed into a format the hardware likes) will need
//! to access the `Context`.

use sdl2::{self, Sdl};
use sdl2::render::Renderer;
use sdl2::video::Window;
use sdl2_ttf;

use sdl2_mixer;
use sdl2_ttf::Sdl2TtfContext;
use sdl2_mixer::Sdl2MixerContext;

use std::fmt;

use conf;
use filesystem::Filesystem;
use graphics;
use timer;
use GameError;
use GameResult;


/// A `Context` holds all the state needed to interface
/// with the hardware.  Only one `Context` can exist at a
/// time.
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
        write!(f, "Context")
    }
}

fn init_ttf() -> GameResult<Sdl2TtfContext> {
    sdl2_ttf::init()
        .map_err(|e| GameError::TTFError(format!("{}", e)))
}


fn init_audio(sdl_context: &Sdl) -> GameResult<sdl2::AudioSubsystem> {
    sdl_context.audio()
        .map_err(|e| GameError::AudioError(format!("{}", e)))
}

fn init_mixer() -> GameResult<Sdl2MixerContext> {
    let frequency = 44100;
    let format = sdl2_mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = 2; // Stereo
    let chunk_size = 1024;
    try!(sdl2_mixer::open_audio(frequency, format, channels, chunk_size));

    let flags = sdl2_mixer::InitFlag::all();
    sdl2_mixer::init(flags)
        .map_err(|e| GameError::AudioError(format!("{}", e)))
}

fn init_window(video: sdl2::VideoSubsystem, window_title: &str, screen_width: u32, screen_height: u32) -> GameResult<Window> {
    video.window(window_title, screen_width, screen_height)
       .position_centered()
       .opengl()
       .build()
       .map_err(|e| GameError::VideoError(format!("{}", e)))
}

impl<'a> Context<'a> {

    /// Tries to create a new Context using settings from the given config file.
    pub fn from_conf(conf: &conf::Conf, fs: Filesystem, sdl_context: Sdl) -> GameResult<Context<'a>> {
        let window_title =  &conf.window_title;
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

        let ctx = Context {
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
    pub fn quit(&mut self) -> GameResult<()> {
        let e = sdl2::event::Event::Quit{timestamp: 10000};
        println!("Pushing event {:?}", e);
        self.event_context.push_event(e)
            .map_err(|err| GameError::from(err))
    }
}


