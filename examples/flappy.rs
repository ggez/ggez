extern crate ggez;

use ggez::{Game, State, GameError};
use std::time::Duration;

struct MainState;

impl State for MainState
{
    fn init(&mut self) -> Result<(), GameError>
    {
        println!("init");
        Ok(())
    }
    fn update(&mut self, dt: Duration) -> Result<(), GameError>
    {
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
    let mut g: MainState = MainState;
    let mut e: Game = Game::new(g);
    e.run();
}
