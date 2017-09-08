extern crate ggez;
extern crate rand;
extern crate sdl2;


use ggez::conf;
use ggez::event;
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

struct MainState {
    spritebatch: graphics::spritebatch::SpriteBatch,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image = graphics::Image::new(ctx, "/tile.png").unwrap();
        let batch = graphics::spritebatch::SpriteBatch::new(image);
        let s = MainState {
            spritebatch: batch,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {

        if timer::get_ticks(ctx) % 100 == 0{

            println!("Delta frame time: {:?} ", _dt);
            println!("Average FPS: {}", timer::get_fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for x in 0..50 {
            for y in 0..50 {
                let p = graphics::DrawParam {
                    dest: graphics::Point::new(x as f32 * 21.0 + 10.0, y as f32 * 20.0 + 10.0),
                    scale: graphics::Point::new(0.125, 0.125),
                    .. Default::default()
                };
                self.spritebatch.add(p);
            }
        }
        graphics::draw_ex(ctx, &self.spritebatch, Default::default())?;
        self.spritebatch.clear();

        graphics::present(ctx);
        Ok(())
    }
}

// Creating a gamestate depends on having an SDL context to load resources.
// Creating a context depends on loading a config file.
// Loading a config file depends on having FS (or we can just fake our way around it
// by creating an FS and then throwing it away; the costs are not huge.)
pub fn main() {
    let c = conf::Conf::new();
    println!("Starting with default config: {:#?}", c);
    let ctx = &mut Context::load_from_conf("imageview", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
