use sdl2::{self, Sdl};
use sdl2::video::Window;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2_ttf::{self, PartialRendering};

use sdl2_mixer;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};
use sdl2_ttf::Sdl2TtfContext;

use rand::distributions::{IndependentSample, Range};
use rand::{self, Rng, Rand};
use std::fmt;

use filesystem::Filesystem;
use resources::{ResourceManager, TextureManager};
use GameError;


pub struct Context<'a> {
    pub sdl_context: Sdl,
    pub ttf_context: Sdl2TtfContext,
    // TODO add mixer and ttf systems to enginestate
    pub resources: ResourceManager,
    pub renderer: Renderer<'a>,
    pub filesystem: Filesystem,
}

impl<'a> fmt::Debug for Context<'a> {
    // TODO: Make this more useful.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context")
    }
}

// For some reason I can't just implement From<Sdl2_ttf::context::InitError>
// for GameError, sooooo...
fn init_ttf() -> Result<Sdl2TtfContext, GameError> {
    match sdl2_ttf::init() {
        Ok(x) => Ok(x),
        Err(e) => Err(GameError::TTFError(format!("{}", e)))
    }
}

// So it has to go sdl2::init() -> load config file
// -> init subsystems and create contexts -> pass to gamestate creation function
impl<'a> Context<'a> {
    pub fn new(window_title: &str,
               screen_width: u32,
               screen_height: u32)
               -> Result<Context<'a>, GameError> {

        let fs = Filesystem::new();
        let sdl_context = try!(sdl2::init());
        let video = try!(sdl_context.video());
        let window = try!(video.window(window_title, screen_width, screen_height)
                               .position_centered()
                               .opengl()
                               .build());

        let mut renderer = try!(window.renderer()
                                      .accelerated()
                                      .build());

        let ttf_context = try!(init_ttf());
        // Can creating a ResourceManager actually fail?
        // Only if it finds no resource files, perhaps...
        // But even then.
        let resources = ResourceManager::new().unwrap();

        let mut ctx = Context {
            sdl_context: sdl_context,
            ttf_context: ttf_context,
            resources: resources,
            renderer: renderer,
            filesystem: fs,
        };

        // By default, unable to init sound is not a fatal error.
        // (Because I'm testing this on a device with no working sound.)
        // We probably want to be able to pass a list of REQUIRED modules
        // to Context::new, and warn if there are ones we can't init unless
        // they're required.
        ctx.init_sound_system().or_else(::warn);
        Ok(ctx)
    }

    // Remove verbose debug output
    fn init_sound_system(&mut self) -> Result<(), GameError> {
        let _audio = try!(self.sdl_context.audio());
        let mut timer = try!(self.sdl_context.timer());
        let _mixer_context = try!(sdl2_mixer::init(INIT_MP3 | INIT_FLAC | INIT_MOD |
                                                   INIT_FLUIDSYNTH |
                                                   INIT_MODPLUG |
                                                   INIT_OGG));

        let frequency = 44100;
        let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
        let channels = 2; // Stereo
        let chunk_size = 1024;
        let _ = try!(sdl2_mixer::open_audio(frequency, format, channels, chunk_size));
        sdl2_mixer::allocate_channels(0);

        {
            let n = sdl2_mixer::get_chunk_decoders_number();
            println!("available chunk(sample) decoders: {}", n);
            for i in 0..n {
                println!("  decoder {} => {}", i, sdl2_mixer::get_chunk_decoder(i));
            }
        }

        {
            let n = sdl2_mixer::get_music_decoders_number();
            println!("available music decoders: {}", n);
            for i in 0..n {
                println!("  decoder {} => {}", i, sdl2_mixer::get_music_decoder(i));
            }
        }

        println!("query spec => {:?}", sdl2_mixer::query_spec());
        Ok(())
    }
}


