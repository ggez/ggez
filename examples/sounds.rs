use ggez::audio;
use ggez::audio::SoundSource;
use ggez::coroutine::Loading;
use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};

use ggez::glam::*;

use ggez::input::keyboard::KeyInput;
use std::env;
use std::path;
use std::time::Duration;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

struct MainState {
    // Sound data is loaded async the for web, where synchronous file reads aren't supported
    data: Loading<audio::SoundData>,
    sound: Option<audio::Source>,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let data = audio::SoundData::new_async::<Context, _>("/sound.ogg");
        let s = MainState { data, sound: None };
        Ok(s)
    }

    // To test: play, play_later, play_detached(),
    // set_repeat, set_fade_in, set_pitch,
    // basically every method on Source, actually,
    // then the same ones for `SpatialSource`.

    /// Plays the sound multiple times
    fn play_detached(&mut self, ctx: &mut Context) {
        let Some(data) = self.data.result() else {
            return;
        };
        let sound = audio::Source::from_data(ctx, data.clone()).unwrap();
        // "detached" sounds keep playing even after they are dropped
        sound.play_detached();
    }

    /// Waits until the sound is done playing before playing again.
    fn play_later(&self) {
        if let Some(s) = &self.sound {
            s.play_later();
        }
    }

    /// Fades the sound in over a second
    /// Which isn't really ideal 'cause the sound is barely a second long, but still.
    fn play_fadein(&mut self) {
        if let Some(s) = &mut self.sound {
            s.set_fade_in(Duration::from_secs(1));
            s.play();
            s.set_fade_in(Duration::ZERO);
        }
    }

    fn play_highpitch(&mut self) {
        if let Some(s) = &mut self.sound {
            s.set_pitch(2.0);
            s.play();
            s.set_pitch(1.0);
        }
    }
    fn play_lowpitch(&mut self) {
        if let Some(s) = &mut self.sound {
            s.set_pitch(0.5);
            s.play();
            s.set_pitch(1.0);
        }
    }

    /// Plays the sound and prints how long it had been playing at call time.
    fn play_stats(&self) {
        let Some(sound) = &self.sound else { return };
        sound.play();
        println!("Elapsed time at play: {:?}", sound.elapsed());
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.sound.is_none() {
            if let Some(data) = self.data.poll(ctx)? {
                self.sound = Some(audio::Source::from_data(ctx, data.clone())?);
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(
            &graphics::Text::new("Press number keys 1-6 to play a sound, or escape to quit."),
            [100., 100.],
        );

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.event.physical_key {
            PhysicalKey::Code(KeyCode::Digit1) => self.play_detached(ctx),
            PhysicalKey::Code(KeyCode::Digit2) => self.play_later(),
            PhysicalKey::Code(KeyCode::Digit3) => self.play_fadein(),
            PhysicalKey::Code(KeyCode::Digit4) => self.play_highpitch(),
            PhysicalKey::Code(KeyCode::Digit5) => self.play_lowpitch(),
            PhysicalKey::Code(KeyCode::Digit6) => self.play_stats(),
            PhysicalKey::Code(KeyCode::Escape) => ctx.request_quit(),
            _ => (),
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("sounds", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
