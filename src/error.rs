
use std;
use std::error::Error;
use std::fmt;

use sdl2;
use toml;
use zip;

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
    FontError(String),
    VideoError(String),
    UnknownError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GameError {:?}", self)
    }
}

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T> = Result<T, GameError>;

/// Emit a non-fatal warning message
/// Ideally we probably want some sort of real logging interface here...
// fn warn(err: GameError) -> GameResult<()> {
//     println!("WARNING: Encountered error: {:?}", err);
//     Ok(())
// }

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

impl From<sdl2::filesystem::PrefPathError> for GameError {
    fn from(e: sdl2::filesystem::PrefPathError) -> GameError {
        let msg = match e {
            sdl2::filesystem::PrefPathError::InvalidOrganizationName(e) => {
                format!("Invalid organization name, {}", e)
            }
            sdl2::filesystem::PrefPathError::InvalidApplicationName(e) => {
                format!("Invalid application name, {}", e)
            }
            sdl2::filesystem::PrefPathError::SdlError(e) => e,
        };
        GameError::ConfigError(msg)
    }
}

impl From<sdl2::render::TextureValueError> for GameError {
    fn from(e: sdl2::render::TextureValueError) -> GameError {
        let msg = e.description();
        GameError::ResourceLoadError(msg.to_owned())
    }
}

// impl From<sdl2_ttf::FontError> for GameError {
//     fn from(e: sdl2_ttf::FontError) -> GameError {
//         let msg = e.description();
//         GameError::ResourceLoadError(msg.to_owned())
//     }
// }

// impl From<sdl2_ttf::InitError> for GameError {
//     fn from(e: sdl2_ttf::InitError) -> GameError {
//         let msg = e.description();
//         GameError::ResourceLoadError(msg.to_owned())
//     }
// }


impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(e)
    }
}

impl From<toml::DecodeError> for GameError {
    fn from(e: toml::DecodeError) -> GameError {
        let errstr = format!("TOML decode error: {}", e.description());

        GameError::ConfigError(errstr)
    }
}

impl From<toml::Error> for GameError {
    fn from(e: toml::Error) -> GameError {
        let errstr = format!("TOML error (possibly encoding?): {}", e.description());
        GameError::ConfigError(errstr)
    }
}

impl From<zip::result::ZipError> for GameError {
    fn from(e: zip::result::ZipError) -> GameError {
        let errstr = format!("Zip error: {}", e.description());
        GameError::ResourceLoadError(errstr)
    }
}
