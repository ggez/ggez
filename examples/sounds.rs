use ggez;

use ggez::audio;
use ggez::audio::SoundSource;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

use std::env;
use std::path;
use std::time::Duration;

struct MainState {
    sound: audio::Source,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let sound = audio::Source::new(ctx, "/sound.ogg")?;
        let s = MainState { sound };
        Ok(s)
    }

    // To test: play, play_later, play_detached(),
    // set_repeat, set_fade_in, set_pitch,
    // basically every method on Source, actually,
    // then the same ones for `SpatialSource`.

    /// Plays the sound multiple times
    fn play_detached(&mut self, _ctx: &mut Context) {
        // "detached" sounds keep playing even after they are dropped
        let _ = self.sound.play_detached();
    }

    /// Waits until the sound is done playing before playing again.
    fn play_later(&mut self, _ctx: &mut Context) {
        let _ = self.sound.play_later();
    }

    /// Fades the sound in over a second
    /// Which isn't really ideal 'cause the sound is barely a second long, but still.
    fn play_fadein(&mut self, ctx: &mut Context) {
        let mut sound = audio::Source::new(ctx, "/sound.ogg").unwrap();
        sound.set_fade_in(Duration::from_millis(1000));
        sound.play_detached().unwrap();
    }

    fn play_highpitch(&mut self, ctx: &mut Context) {
        let mut sound = audio::Source::new(ctx, "/sound.ogg").unwrap();
        sound.set_pitch(2.0);
        sound.play_detached().unwrap();
    }
    fn play_lowpitch(&mut self, ctx: &mut Context) {
        let mut sound = audio::Source::new(ctx, "/sound.ogg").unwrap();
        sound.set_pitch(0.5);
        sound.play_detached().unwrap();
    }

    /// Plays the sound and prints out stats until it's done.
    fn play_stats(&mut self, _ctx: &mut Context) {
        let _ = self.sound.play();
        while self.sound.playing() {
            println!("Elapsed time: {:?}", self.sound.elapsed())
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        graphics::queue_text(
            ctx,
            &graphics::Text::new("Press number keys 1-6 to play a sound, or escape to quit."),
            na::Point2::origin(),
            None,
        );
        graphics::draw_queued_text(
            ctx,
            (na::Point2::new(100.0, 100.0),),
            None,
            graphics::FilterMode::Linear,
        )?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: input::keyboard::KeyCode,
        _keymod: input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            input::keyboard::KeyCode::Key1 => self.play_detached(ctx),
            input::keyboard::KeyCode::Key2 => self.play_later(ctx),
            input::keyboard::KeyCode::Key3 => self.play_fadein(ctx),
            input::keyboard::KeyCode::Key4 => self.play_highpitch(ctx),
            input::keyboard::KeyCode::Key5 => self.play_lowpitch(ctx),
            input::keyboard::KeyCode::Key6 => self.play_stats(ctx),
            input::keyboard::KeyCode::Escape => event::quit(ctx),
            _ => (),
        }
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

    let cb = ggez::ContextBuilder::new("imageview", "ggez").add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
