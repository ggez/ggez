use state::State;

use std::path::Path;
use std::thread;
use std::time::Duration;

use sdl2;
use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode::*;
use sdl2::render::TextureQuery;
use sdl2_ttf;

use sdl2_mixer;
use sdl2_mixer::Music;
use sdl2_mixer::MusicType;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};

use rand::{self, Rng, Rand};
use rand::distributions::{IndependentSample, Range};

use resources::ResourceManager;

pub struct Game<'e>
{
    states: Vec<Box<State + 'e>>,
    sdl_context: Sdl,
    // TODO add mixer and ttf systems to enginestate
    resources: ResourceManager
}

impl<'e> Game<'e> {
    pub fn new<T: State + 'e>(initial_state: T) -> Game<'e>
    {
        let screen_width = 800;
        let screen_height = 600;
        let sdl_context = sdl2::init().unwrap();

        Game
        {
            states: vec![Box::new(initial_state)],
            sdl_context: sdl_context,
            resources: ResourceManager::new()
        }
    }

    pub fn push_state<T: State + 'e>(&mut self, state: T) {
        self.states.push(Box::new(state));
    }

    pub fn pop_state() {}

    fn get_active_state(&mut self) -> Option<&mut Box<State + 'e>> {
        self.states.last_mut()
    }

    /// Remove verbose debug output
    fn init_sound_system(&mut self)
    {
        let _audio = self.sdl_context.audio().unwrap();
        let mut timer = self.sdl_context.timer().unwrap();
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
    }

    pub fn play_sound(&self, sound: &str)
    {
        let resource = self.resources.get_sound(sound);
        match resource
        {
            Some(music) => {
                println!("music => {:?}", music);
                println!("music type => {:?}", music.get_type());
                println!("music volume => {:?}", sdl2_mixer::Music::get_volume());
                println!("play => {:?}", music.play(1));
            }
            None => {
                println!("No such resource!");
            }
        }
    }

    pub fn run(&mut self) {
        self.init_sound_system();
        let mut rng = rand::thread_rng();
        let mut timer = self.sdl_context.timer().unwrap();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let video = self.sdl_context.video().unwrap();
        let ttf_context = sdl2_ttf::init().unwrap();

        let mut font = ttf_context.load_font(Path::new("resources/DejaVuSerif.ttf"), 128).unwrap();
        let surface = font.render("ruffel")
                          .blended(Color::rand(&mut rng))
                          .unwrap();

        let window = video.window("Ruffel", 800, 600)
                          .position_centered()
                          .opengl()
                          .build()
                          .unwrap();

        let mut renderer = window.renderer()
                                 .accelerated()
                                 .build()
                                 .unwrap();

        let mut font_texture = renderer.create_texture_from_surface(&surface).unwrap();

        let TextureQuery { width, height, .. } = font_texture.query();

        // If the example text is too big for the screen, downscale it (and center irregardless)
        let padding = 64;

        // Initialize State handlers
        for s in &mut self.states {
            s.init();
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
            renderer.copy(&mut font_texture, None, Some(target));
            renderer.present();

            if let Some(active_state) = self.get_active_state() {
                active_state.update(delta);
                active_state.draw();
            } else {
                done = true;
            }

            let end_time = timer.ticks();
            delta = Duration::from_millis((end_time - start_time) as u64);
            thread::sleep_ms(1000 / 60);
        }
    }
}
