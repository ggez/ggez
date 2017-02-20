extern crate ggez;
use ggez::conf;
use ggez::game;
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

// First we make a structure to contain the game's state
struct MainState {
    image1: graphics::Image,
    image2: graphics::Image,
    zoomlevel: f32,
}

// Then we implement the `ggez::game::GameState` trait on it, which
// requires callbacks for creating the game state, updating it each
// frame, and drawing it.
//
// The `GameState` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {

        let image1 = graphics::Image::new(ctx, "resources/dragon1.png")?;
        let image2 = graphics::Image::new(ctx, "resources/dragon2.png")?;
        let s = MainState {
            image1: image1,
            image2: image2,
            zoomlevel: 1.0,
        };

        graphics::set_screen_coordinates(ctx, -s.zoomlevel, s.zoomlevel, s.zoomlevel, -s.zoomlevel);
        Ok(s)
    }
}


impl game::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        graphics::set_screen_coordinates(ctx, -self.zoomlevel, self.zoomlevel, self.zoomlevel, -self.zoomlevel);
        self.zoomlevel += 1.0;
        println!("Updating");
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        println!("Starting draw");
        // graphics::clear(ctx);
        // graphics::draw(ctx, &mut self.image1, None, None);
        // let dst = graphics::Rect::new(1.0, 1.0, 0.0, 0.0);
        // graphics::draw(ctx, &mut self.image2, None, Some(dst));
        graphics::present(ctx);
        println!("Approx FPS: {}", timer::get_fps(ctx));
        // timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }
}

// Now our main function, which does three things:
//
// * First, create a new `ggez::conf::Conf`
// object which contains configuration info on things such
// as screen resolution and window title,
// * Second, create a `ggez::game::Game` object which will
// do the work of creating our MainState and running our game,
// * then just call `game.run()` which runs the `Game` mainloop.
pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("helloworld", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = game::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
