extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;
extern crate rand;
extern crate rustc_serialize;
extern crate toml;

mod state;
pub mod game;
mod filesystem;
mod resources;
mod context;
pub mod conf;

pub use state::State;
pub use game::Game;
pub use context::Context;

#[derive(Debug)]
pub enum GameError {
    Lolwtf,
    ArbitraryError(String),
    ConfigError(String),
    ResourceLoadError(String),
    ResourceNotFound(String),
    WindowError(sdl2::video::WindowBuildError),
    IOError(std::io::Error),
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

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(e)
    }
}

impl From<toml::DecodeError> for GameError {
    fn from(e: toml::DecodeError) -> GameError {
        let errstr = format!("{}", e);
        GameError::ConfigError(errstr)
    }
}
