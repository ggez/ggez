extern crate ggez;

use ggez::{Game, State, GameError};
use std::time::Duration;

struct MainState
{
}

impl State for MainState
{
    fn init(&self) -> Result<(), GameError>
    {
        println!("init");
        Ok(())
    }
    fn update(&self, dt: Duration) -> Result<(), GameError>
    {
        println!("update");
        Ok(())
    }
    fn draw(&self) -> Result<(), GameError>
    {
        println!("draw");
        Ok(())
    }
}

pub fn main() {
    let mut g: MainState = MainState {};
    let mut e: Game = Game::new(&mut g);
    e.run();
}
