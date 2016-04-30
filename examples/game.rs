extern crate ggez;

//use ggez::Engine;
//use ggez::State;
use ggez::*;
use std::time::Duration;

struct Game
{
    i: i32
}

impl State for Game
{
    fn init(&self) -> Result<(), GameError>
    {
        println!("init");
        Ok(())
    }
    fn update(&self, d: Duration) -> Result<(), GameError>
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
    let mut g: Game = Game { i:5 };
    let mut e: Engine = Engine::new();
    e.add_obj(&mut g);
    e.ralf();
}
