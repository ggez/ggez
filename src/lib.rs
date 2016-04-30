extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;
extern crate rand;

mod state;
mod game;
mod resources;

pub use state::State;
pub use game::Game;
pub use game::Context;

#[derive(Debug)]
pub enum GameError {
    Lolwtf,
    ResourceLoadError(String),
    ResourceNotFound,
}
