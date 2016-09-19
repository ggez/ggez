use state::State;
use context::Context;
use resources::{ResourceManager, TextureManager, FontManager};
use GameError;

use std::path::Path;
use std::thread;
use std::option;
use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::keyboard::Keycode::*;
use sdl2::surface::Surface;

use rand::{self, Rand};

use sdl2_mixer;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};

pub struct Game<'a, S: State> {
    window_title: &'static str,
    screen_width: u32,
    screen_height: u32,
    states: Vec<S>,
    context: Option<Context<'a>>,
}

impl<'a, S: State> Game<'a, S> {
    pub fn new(initial_state: S) -> Game<'a, S> {
        Game {
            window_title: "Ruffel",
            screen_width: 800,
            screen_height: 600,
            states: vec![initial_state],
            context: None,
        }
    }

    pub fn push_state(&mut self, state: S) {
        self.states.push(state);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    fn get_active_state(&mut self) -> Option<&mut S> {
        self.states.last_mut()
    }

    // Remove verbose debug output
    fn init_sound_system(&mut self) {
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
        let mut ctx = Context::new(self.window_title, self.screen_width, self.screen_height).unwrap();

        self.context = Some(ctx);
        self.init_sound_system();
        let mut ctx = self.context.take().unwrap();
        //let mut rng = rand::thread_rng();
        let mut timer = ctx.sdl_context.timer().unwrap();
        let mut event_pump = ctx.sdl_context.event_pump().unwrap();

        ctx.resources.load_font("DejaVuSerif", "resources/DejaVuSerif.ttf").unwrap();

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

            if let Some(active_state) = self.get_active_state() {
                active_state.update(&mut ctx, delta);

                //ctx.renderer.set_draw_color(Color::rand(&mut rng));
                //ctx.renderer.clear();
                active_state.draw(&mut ctx);
                //ctx.renderer.present();
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
    match resource {
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
