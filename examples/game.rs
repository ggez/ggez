extern crate ggez;

//use ggez::Engine;
//use ggez::State;
use ggez::*;

struct Game
{
    i: i32
}

impl State for Game
{
    fn init(&self) -> Result<(), GameError>
    {
        print!("init");
        Ok(())
    }
    fn update(&self) -> Result<(), GameError>
    {
        print!("update");
        Ok(())
    }
    fn draw(&self) -> Result<(), GameError>
    {
        print!("draw");
        Ok(())
    }
}

pub fn main() {
    let mut g: Game = Game { i:5 };
    let mut e: Engine = Engine::new();
    e.add_obj(&mut g);
    e.ralf();
}
