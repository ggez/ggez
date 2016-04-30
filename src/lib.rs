extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;
extern crate rand;

mod state;
pub mod game;
mod resources;
mod context;

pub use state::State;
pub use game::Game;
pub use context::Context;

#[derive(Debug)]
pub enum GameError {
    Lolwtf,
    ResourceLoadError(String),
    ResourceNotFound,
}
