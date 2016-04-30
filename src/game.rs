use state::State;
use resources::{ResourceManager, TextureManager, FontManager};
use GameError;

use std::path::Path;
use std::thread;
use std::option;
use std::time::Duration;

use sdl2;
use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode::*;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::surface::Surface;
use sdl2_ttf::{self, PartialRendering};

use rand::{self, Rng, Rand};
use rand::distributions::{IndependentSample, Range};

use sdl2_mixer;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};


pub struct Context {
    sdl_context: Sdl,
    // TODO add mixer and ttf systems to enginestate
    pub resources: ResourceManager
}

pub struct Game<S: State> {
    window_title: String,
    screen_width: u32,
    screen_height: u32,
    states: Vec<S>,
    context: Option<Context>
}

impl<S: State> Game<S> {
    pub fn new(initial_state: S) -> Game<S> {
        Game
        {
            window_title: String::from("Ruffel"),
            screen_width: 800,
            screen_height: 600,
            states: vec![initial_state],
            context: None
        }
    }

    pub fn push_state(&mut self, state: S) {
        self.states.push(state);
    }

    pub fn pop_state() {}

    fn get_active_state(&mut self) -> Option<&mut S> {
        self.states.last_mut()
    }

    /// Remove verbose debug output
    fn init_sound_system(&mut self)
    {
        let mut ctx = self.context.take().unwrap();
        let _audio = ctx.sdl_context.audio().unwrap();
        let mut timer = ctx.sdl_context.timer().unwrap();
        let _mixer_context = sdl2_mixer::init(INIT_MP3 | INIT_FLAC | INIT_MOD | INIT_FLUIDSYNTH |
                                              INIT_MODPLUG |
                                              INIT_OGG)
                                .unwrap();

        let frequency = 44100;
        let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
        let channels = 2; // Stereo
        let chunk_size = 1024;
        let _ = sdl2_mixer::open_audio(frequency, format, channels, chunk_size).unwrap();
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

        self.context = Some(ctx);
    }

    pub fn run(&mut self) {

        let sdl_context = sdl2::init().unwrap();
        let resources = ResourceManager::new();
        let mut ctx = Context {
            sdl_context: sdl_context,
            resources: resources.unwrap()
        };

        self.context = Some(ctx);
        self.init_sound_system();
        let mut ctx = self.context.take().unwrap();
        let mut rng = rand::thread_rng();
        let mut timer = ctx.sdl_context.timer().unwrap();
        let mut event_pump = ctx.sdl_context.event_pump().unwrap();
        let video = ctx.sdl_context.video().unwrap();



        let window = video.window(self.window_title.as_str(), self.screen_width, self.screen_height)
                          .position_centered()
                          .opengl()
                          .build()
                          .unwrap();

        let mut renderer = window.renderer()
                                 .accelerated()
                                 .build()
                                 .unwrap();


        // let resource_manager = &mut ctx.resources;
        ctx.resources.load_font("DejaVuSerif", "resources/DejaVuSerif.ttf").unwrap();

        let mut font_texture1 =
            create_font_surface("roffl", "DejaVuSerif", 128, &mut ctx.resources)
                            .unwrap()
                            .blended(Color::rand(&mut rng))
                            .map_err(|_| GameError::Lolwtf)
                            .and_then(|s| renderer.create_texture_from_surface(&s)
                                                  .map_err(|_| GameError::Lolwtf)).unwrap();

        let mut font_texture2 =
            create_font_surface("fizzbazz", "DejaVuSerif", 72, &mut ctx.resources)
                            .unwrap()
                            .blended(Color::rand(&mut rng))
                            .map_err(|_| GameError::Lolwtf)
                            .and_then(|s| renderer.create_texture_from_surface(&s)
                                                  .map_err(|_| GameError::Lolwtf)).unwrap();

        // TODO move the context into context option
        // let TextureQuery { width, height, .. } = font_texture.query();

        // If the example text is too big for the screen, downscale it (and center irregardless)
        let padding = 64;

        // Initialize State handlers
        for s in &mut self.states {
            s.load(&mut ctx);
        }

        let mut done = false;
        let mut delta = Duration::new(0, 0);

        while !done {
            let start_time = timer.ticks();

            for event in event_pump.poll_iter() {
                match event {
                    Quit { .. } => done = true,
                    KeyDown { keycode, .. } => {
                        match keycode {
                            Some(Escape) => done = true,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            let between = Range::new(0, 400);
            let target = Rect::new(between.ind_sample(&mut rng),
                                   50,
                                   between.ind_sample(&mut rng) as u32,
                                   500);
            renderer.set_draw_color(Color::rand(&mut rng));
            renderer.clear();
            renderer.copy(&mut font_texture1, None, Some(target));
            renderer.copy(&mut font_texture2, None, Some(target));
            renderer.present();

            if let Some(active_state) = self.get_active_state() {
                active_state.update(&mut ctx, delta);
                active_state.draw();
            } else {
                done = true;
            }

            let end_time = timer.ticks();
            delta = Duration::from_millis((end_time - start_time) as u64);
            thread::sleep_ms(1000 / 60);
        }

        self.context = Some(ctx);
    }
}

    pub fn play_sound(ctx: &mut Context, sound: &str) -> Result<(), GameError> {
        let resource = ctx.resources.get_sound(sound);
        match resource
        {
            Some(music) => {
                println!("music => {:?}", music);
                println!("music type => {:?}", music.get_type());
                println!("music volume => {:?}", sdl2_mixer::Music::get_volume());
                println!("play => {:?}", music.play(1));
                println!("You've played well");
            }
            None => {
                println!("No such resource!");
            }
        }
        Ok(())
    }

fn create_font_surface<'a>(text: &'a str,
                       font_name: &str,
                       size: u16,
                       resource_manager: &'a mut ResourceManager) -> Result<PartialRendering<'a>, GameError> {
    let mut font = try!(resource_manager.get_font(font_name, size));
    Ok(font.render(text))
}
