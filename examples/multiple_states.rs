//! Basic hello world example.

use cgmath;
use ggez;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::event::{KeyMods, KeyCode};
use std::env;
use std::path;

// First we make a structure to contain the game's state
struct MainState {
    frames: usize,
    text: graphics::Text,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;
        let text = graphics::Text::new(("Press P to pause!", font, 48.0));

        let s = MainState { frames: 0, text };
        Ok(s)
    }
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // Drawables are drawn from their top-left corner.
        let offset = self.frames as f32 / 10.0;
        let dest_point = cgmath::Point2::new(offset, offset);
        graphics::draw(ctx, &self.text, (dest_point,))?;
        graphics::present(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::P {
            let mut paused_state = PausedState::new(ctx).unwrap();
            event::push_state(ctx, &mut paused_state).unwrap();
        }
        if keycode == KeyCode::Escape {
            event::pop_state(ctx);
        }
    }
}

// Now, let's create a paused state.
struct PausedState {
    frames: usize,
    text: graphics::Text,
}

impl PausedState {
    fn new(ctx: &mut Context) -> GameResult<PausedState> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;
        let text = graphics::Text::new(("Paused!", font, 48.0));

        let s = PausedState { frames: 0, text };
        Ok(s)
    }
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler for PausedState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.3, 0.2, 0.1, 1.0].into());

        // Drawables are drawn from their top-left corner.
        let rect = ggez::graphics::screen_coordinates(ctx);
        let dest_point = cgmath::Point2::new((rect.w - self.text.width(ctx) as f32) / 2.0, (rect.h - self.text.height(ctx) as f32) / 2.0);
        graphics::draw(ctx, &self.text, (dest_point,))?;
        graphics::present(ctx)?;

        Ok(())
    }
}

// Now our main function, which does three things:
//
// * First, create a new `ggez::ContextBuilder`
// object which contains configuration info on things such
// as screen resolution and window title.
// * Second, create a `ggez::game::Game` object which will
// do the work of creating our MainState and running our game.
// * Then, just call `game.run()` which runs the `Game` mainloop.
pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("multiplestates", "ggez").add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;

    let state = &mut MainState::new(&mut ctx)?;
    event::run(&mut ctx, event_loop, state)
}
