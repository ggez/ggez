use sdl2::{self, Sdl};
use sdl2::video::Window;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2_ttf::{self, PartialRendering};

use sdl2_mixer;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};


use rand::distributions::{IndependentSample, Range};
use rand::{self, Rng, Rand};
use std::fmt;

use filesystem::Filesystem;
use resources::{ResourceManager, TextureManager, FontManager};
use GameError;


pub struct Context<'a> {
    pub sdl_context: Sdl,
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

        // Can creating a ResourceManager actually fail?
        // Only if it finds no resource files, perhaps...
        // But even then.
        let resources = ResourceManager::new().unwrap();

        let mut ctx = Context {
            sdl_context: sdl_context,
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


    pub fn print(&mut self, text: &str, x: u32, y: u32) {
        let mut rng = rand::thread_rng();
        let between = Range::new(0, 400);
        let target = Rect::new(between.ind_sample(&mut rng),
                               50,
                               between.ind_sample(&mut rng) as u32,
                               500);

        let mut font_texture = create_font_surface(text, "DejaVuSerif", 72, &mut self.resources)
                                   .unwrap()
                                   .blended(Color::rand(&mut rng))
                                   .map_err(|_| GameError::Lolwtf)
                                   .and_then(|s| {
                                       self.renderer
                                           .create_texture_from_surface(&s)
                                           .map_err(|_| GameError::Lolwtf)
                                   })
                                   .unwrap();

        self.renderer.copy(&mut font_texture, None, Some(target));
    }
}

fn create_font_surface<'a>(text: &'a str,
                           font_name: &str,
                           size: u16,
                           resource_manager: &'a mut ResourceManager)
                           -> Result<PartialRendering<'a>, GameError> {
    let mut font = try!(resource_manager.get_font(font_name, size));
    Ok(font.render(text))
}
