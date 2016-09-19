use state::State;
use context::Context;
use resources::{ResourceManager, TextureManager, FontManager};
use ::GameError;
use ::warn;

use std::path::Path;
use std::thread;
use std::option;
use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::keyboard::Keycode::*;
use sdl2::surface::Surface;

use rand::{self, Rand};


#[derive(Debug)]
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


    pub fn run(&mut self) -> Result<(), GameError> {
        let mut ctx = try!(Context::new(self.window_title, self.screen_width, self.screen_height));

        self.context = Some(ctx);
        //self.init_sound_system().or_else(warn);
        let mut ctx = self.context.take().unwrap();
        let mut timer = try!(ctx.sdl_context.timer());
        let mut event_pump = try!(ctx.sdl_context.event_pump());

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
        Ok(())
    }
}

pub fn play_sound(ctx: &mut Context, sound: &str) -> Result<(), GameError> {
    let resource = ctx.resources.get_sound(sound);
    match resource {
        Some(music) => {
            // println!("music => {:?}", music);
            // println!("music type => {:?}", music.get_type());
            // println!("music volume => {:?}", sdl2_mixer::Music::get_volume());
            // println!("play => {:?}", music.play(1));
            // println!("You've played well");
            ()
        }
        None => {
            println!("No such resource!");
        }
    }
    Ok(())
}
