extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;
extern crate rand;
extern crate rustc_serialize;
extern crate toml;
#[macro_use]
extern crate lazy_static;
extern crate zip;


pub mod audio;
mod state;
pub mod game;
pub mod graphics;
pub mod filesystem;
//mod resources;
mod context;
pub mod conf;
mod util;

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
    RenderError(String),
    AudioError(String),
    WindowError(sdl2::video::WindowBuildError),
    IOError(std::io::Error),
    TTFError(String),
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

/*
use sdl2_ttf;

impl From<sdl2_ttf::InitError> for GameError {
    fn from(e: sdl2_ttf::context::InitError) -> GameError {
        let s = format!("{}", e);
        GameError::TTFError(String::from(s))
            /*
        match e {
            sdl2_ttf::context::InitError::AlreadyInitializedError =>
                GameError::TTFError(String::from("TTF has already been initialized")),
            sdl2_ttf::context::InitError::InitializationError(ref error) =>
                GameError::TTFError(String::from(error.description()))
        }
*/
    }
}
*/


/*
impl From<T> for GameError {
    where T: std::error::Error + std::marker::Sized;
    fn from(e: T) -> GameError {
        GameError::ArbitraryError(e.description())
    }
}
*/

