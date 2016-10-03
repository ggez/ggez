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
    ArbitraryError(String),
    ResourceLoadError(String),
    ResourceNotFound,
    WindowError(sdl2::video::WindowBuildError),
}

fn warn(err: GameError) -> Result<(), GameError> {
    println!("WARNING: Encountered error: {:?}", err);
    Ok(())
}

impl From<String> for GameError {
    fn from(s: String) -> GameError {
        GameError::ArbitraryError(s)
    }
}

impl From<sdl2::video::WindowBuildError> for GameError {
    fn from(s: sdl2::video::WindowBuildError) -> GameError {
        GameError::WindowError(s)
    }
}

impl From<sdl2::IntegerOrSdlError> for GameError {
    fn from(e: sdl2::IntegerOrSdlError) -> GameError {
        match e {
            sdl2::IntegerOrSdlError::IntegerOverflows(s, i) => {
                let message = format!("Integer overflow: {}, str {}", i, s);
                GameError::ArbitraryError(message)
            }
            sdl2::IntegerOrSdlError::SdlError(s) => GameError::ArbitraryError(s),
        }
    }
}
