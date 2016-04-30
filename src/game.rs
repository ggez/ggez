use state::State;
use resources::{ResourceManager, TextureManager, FontManager};
use GameError;

use std::path::Path;
use std::thread;
use std::time::Duration;

use sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode::*;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::surface::Surface;
use sdl2_ttf::PartialRendering;
use rand::{self, Rng, Rand};
use rand::distributions::{IndependentSample, Range};


pub struct Game<S: State> {
    states: Vec<S>
}

impl<S: State> Game<S> {
    pub fn new(initial_state: S) -> Game<S> {
        Game {
            states: vec![initial_state]
        }
    }

    pub fn push_state(&mut self, state: S) {
        self.states.push(state);
    }

    pub fn pop_state() {}

    fn get_active_state(&mut self) -> Option<&mut S> {
        self.states.last_mut()
    }

    pub fn run(&mut self) {
        let screen_width = 800;
        let screen_height = 600;

        let mut rng = rand::thread_rng();
        let sdl_context = sdl2::init().unwrap();
        let mut timer = sdl_context.timer().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let video = sdl_context.video().unwrap();

        let mut resource_manager = ResourceManager::new().unwrap();

        let window = video.window("Ruffel", 800, 600)
                          .position_centered()
                          .opengl()
                          .build()
                          .unwrap();

        let mut renderer = window.renderer()
                                 .accelerated()
                                 .build()
                                 .unwrap();

        resource_manager.load_font("DejaVuSerif", "resources/DejaVuSerif.ttf").unwrap();

        let mut font_texture1 =
            create_font_surface("roffl", "DejaVuSerif", 128, &mut resource_manager)
                            .unwrap()
                            .blended(Color::rand(&mut rng))
                            .map_err(|_| GameError::Lolwtf)
                            .and_then(|s| renderer.create_texture_from_surface(&s)
                                                  .map_err(|_| GameError::Lolwtf)).unwrap();

        let mut font_texture2 =
            create_font_surface("fizzbazz", "DejaVuSerif", 72, &mut resource_manager)
                            .unwrap()
                            .blended(Color::rand(&mut rng))
                            .map_err(|_| GameError::Lolwtf)
                            .and_then(|s| renderer.create_texture_from_surface(&s)
                                                  .map_err(|_| GameError::Lolwtf)).unwrap();

        // If the example text is too big for the screen, downscale it (and center irregardless)
        let padding = 64;

        // Initialize State handlers
        for s in &mut self.states {
            s.load();
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

fn create_font_surface<'a>(text: &'a str,
                       font_name: &str,
                       size: u16,
                       resource_manager: &'a mut ResourceManager) -> Result<PartialRendering<'a>, GameError> {
    let mut font = try!(resource_manager.get_font(font_name, size));
    Ok(font.render(text))
}
