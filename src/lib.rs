extern crate sdl2;
extern crate sdl2_ttf;
extern crate rand;

mod state;
mod game;

pub use state::State;
pub use game::Game;

pub enum GameError {
    Lolwtf
}
