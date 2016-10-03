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

/// An enum containing all kinds of game engine error.
#[derive(Debug)]
pub enum GameError {
    FilesystemError(String),
    ConfigError(String),
    ResourceLoadError(String),
    ResourceNotFound(String),
    RenderError(String),
    AudioError(String),
    WindowError(sdl2::video::WindowBuildError),
    IOError(std::io::Error),
    TTFError(String),
    VideoError(String),
    UnknownError(String),
}

pub type GameResult<T> = Result<T, GameError>;

fn warn(err: GameError) -> GameResult<()> {
    println!("WARNING: Encountered error: {:?}", err);
    Ok(())
}

impl From<String> for GameError {
    fn from(s: String) -> GameError {
        GameError::UnknownError(s)
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
                GameError::UnknownError(message)
            }
            sdl2::IntegerOrSdlError::SdlError(s) => GameError::UnknownError(s),
        }
    }
}

// Annoyingly, PrefPathError doesn't implement Debug or Display in
// version 0.23
// It at least has Debug in the latest tip.
impl From<sdl2::filesystem::PrefPathError> for GameError {
    fn from(e: sdl2::filesystem::PrefPathError) -> GameError {
        let msg = match e {
            sdl2::filesystem::PrefPathError::InvalidOrganizationName(e) => format!("Invalid organization name, {}", e),
            sdl2::filesystem::PrefPathError::InvalidApplicationName(e) => format!("Invalid application name, {}", e),
            sdl2::filesystem::PrefPathError::SdlError(e) =>
            e
        };
        GameError::ConfigError(msg)
    }
}

impl From<sdl2::render::TextureValueError> for GameError {
    fn from(e: sdl2::render::TextureValueError) -> GameError {
        let msg = format!("{}", e);
        GameError::ResourceLoadError(msg)
    }
}

impl From<sdl2_ttf::FontError> for GameError {
    fn from(e: sdl2_ttf::FontError) -> GameError {
        let msg = format!("{}", e);
        GameError::ResourceLoadError(msg)
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

