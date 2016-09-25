//! A `Context` is an object that holds on to global resources.

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
use GameError;
use GameResult;


pub struct Context<'a> {
    pub sdl_context: Sdl,
    pub ttf_context: Sdl2TtfContext,
    _audio_context: sdl2::AudioSubsystem,
    pub mixer_context: Sdl2MixerContext,
    pub renderer: Renderer<'a>,
    pub filesystem: Filesystem,
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
// So it has to go sdl2::init() -> load config file
// -> init subsystems and create contexts -> pass to gamestate creation function
impl<'a> Context<'a> {
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

        let mut ctx = Context {
            sdl_context: sdl_context,
            ttf_context: ttf_context,
            _audio_context: audio_context,
            mixer_context: mixer_context,
            renderer: renderer,
            filesystem: fs,
        };

        ctx.print_sound_stats();
        ctx.print_resource_stats();
        Ok(ctx)
    }

    // pub fn new(window_title: &str,
    //            screen_width: u32,
    //            screen_height: u32)
    //            -> GameResult<Context<'a>> {

    //     let fs = try!(Filesystem::new());
    //     let sdl_context = try!(sdl2::init());
    //     let video = try!(sdl_context.video());
    //     let window = try!(init_window(video, window_title, screen_width, screen_height));

    //     let renderer = try!(window.renderer()
    //                                   .accelerated()
    //                                   .build());

    //     let ttf_context = try!(init_ttf());
    //     let audio_context = try!(init_audio(&sdl_context));
    //     let mixer_context = try!(init_mixer());

    //     let mut ctx = Context {
    //         sdl_context: sdl_context,
    //         ttf_context: ttf_context,
    //         _audio_context: audio_context,
    //         mixer_context: mixer_context,
    //         renderer: renderer,
    //         filesystem: fs,
    //     };

    //     ctx.print_sound_stats();
    //     ctx.print_resource_stats();
    //     Ok(ctx)
    // }



    fn print_sound_stats(&self) {
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

    fn print_resource_stats(&mut self) {
        self.filesystem.print_all();
    }
}


