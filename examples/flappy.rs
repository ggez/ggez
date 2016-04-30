extern crate ggez;

use ggez::{Game, State, GameError, Context};
use std::time::Duration;
use std::path::Path;

struct MainState
{
    a: i32
}

impl MainState {
    fn new() -> MainState {
        MainState { a: 0 }
    }
}

impl State for MainState
{
    fn init(&mut self, ctx: &mut Context) -> Result<(), GameError>
    {
        println!("init");
        ctx.resources.load_sound("sound", Path::new("./resources/sound.mp3"));
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> Result<(), GameError>
    {
        self.a = self.a + 1;
        if self.a > 100
        {
            self.a = 0;
            Game::play_sound(ctx, "sound");
        }
        println!("update");
        Ok(())
    }
    fn draw(&mut self) -> Result<(), GameError>
    {
        println!("draw");
        Ok(())
    }
}

pub fn main() {
    let mut g: MainState = MainState::new();
    let mut e: Game = Game::new(g);
    e.run();
}
