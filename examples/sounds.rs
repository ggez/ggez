extern crate ggez;

use ggez::audio;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};

use std::env;
use std::path;

struct MainState {
    pos_x: f32,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { pos_x: 0.0 };
        Ok(s)
    }

    // To test: play, play_later, play_detached(),
    // set_repeat, set_fade_in, set_pitch,
    // basically every method on Source, actually,
    // then the same ones for `SpatialSource`.

    fn play_test(ctx: &mut Context) -> GameResult {
        let mut sound = audio::Source::new(ctx, "/sound.ogg")?;

        // "detached" sounds keep playing even after they are dropped
        let _ = sound.play_detached();
        Ok(())
    }

    fn play_next_sound(&mut self, ctx: &mut Context) {
        Self::play_test(ctx).unwrap();
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        graphics::queue_text(
            ctx,
            &graphics::Text::new("Press spacebar to play the next sound, or escape to quit."),
            na::Point2::origin(),
            None,
        );
        graphics::draw_queued_text(ctx, (na::Point2::new(100.0, 100.0),))?;

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
        if keycode == input::keyboard::KeyCode::Space {
            self.play_next_sound(ctx);
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
