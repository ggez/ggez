//! Error types and conversion functions.

use std;
use std::error::Error;
use std::fmt;

use gfx;
use gfx_window_sdl;

use image;
use rodio::decoder::DecoderError;
use sdl2;
use app_dirs2::AppDirsError;
use toml;
use zip;

/// An enum containing all kinds of game framework errors.
#[derive(Debug)]
pub enum GameError {
    /// An error in the filesystem layout
    FilesystemError(String),
    /// An error in the config file
    ConfigError(String),
    /// An error in some part of the underlying SDL library.
    SdlError(String),
    /// An error saying that a an integer overflow/underflow occured
    /// in an underlying library.
    IntegerError(String),
    /// An error trying to parse a resource
    ResourceLoadError(String),
    /// Unable to find a resource; the Vec is the paths it searched for and associated errors
    ResourceNotFound(String, Vec<(std::path::PathBuf, GameError)>),
    /// Something went wrong in the renderer
    RenderError(String),
    /// Something went wrong in the audio playback
    AudioError(String),
    /// Something went wrong trying to create a window
    WindowError(gfx_window_sdl::InitError),
    /// Something went wrong trying to read from a file
    IOError(std::io::Error),
    /// Something went wrong trying to load/render a font
    FontError(String),
    /// Something went wrong applying video settings.
    VideoError(String),
    /// Something went compiling shaders
    ShaderProgramError(gfx::shade::ProgramError),
    /// Something else happened; this is generally a bug.
    UnknownError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameError::ResourceNotFound(ref s, ref paths) => write!(
                f,
                "Resource not found: {}, searched in paths {:?}",
                s, paths
            ),
            GameError::ConfigError(ref s) => write!(f, "Config error: {}", s),
            GameError::ResourceLoadError(ref s) => write!(f, "Error loading resource: {}", s),
            _ => write!(f, "GameError {:?}", self),
        }
    }
}

impl Error for GameError {
    fn description(&self) -> &str {
        match *self {
            GameError::FilesystemError(_) => "Filesystem error",
            GameError::ConfigError(_) => "Config file error",
            GameError::SdlError(_) => "SDL error",
            GameError::IntegerError(_) => "Integer error",
            GameError::ResourceLoadError(_) => "Resource load error",
            GameError::ResourceNotFound(_, _) => "Resource not found",
            GameError::RenderError(_) => "Render error",
            GameError::AudioError(_) => "Audio error",
            GameError::WindowError(_) => "Window error",
            GameError::IOError(_) => "IO error",
            GameError::FontError(_) => "Font error",
            GameError::VideoError(_) => "Video error",
            GameError::ShaderProgramError(_) => "Shader program error",
            GameError::UnknownError(_) => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            GameError::ShaderProgramError(ref e) => Some(e),
            GameError::IOError(ref e) => Some(e),
            _ => None,
        }
    }
}

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T> = Result<T, GameError>;

impl From<String> for GameError {
    fn from(s: String) -> GameError {
        GameError::UnknownError(s)
    }
}

impl From<gfx_window_sdl::InitError> for GameError {
    fn from(s: gfx_window_sdl::InitError) -> GameError {
        GameError::WindowError(s)
    }
}

impl From<sdl2::IntegerOrSdlError> for GameError {
    fn from(e: sdl2::IntegerOrSdlError) -> GameError {
        match e {
            sdl2::IntegerOrSdlError::IntegerOverflows(s, i) => {
                let message = format!("Integer overflow: {}, str {}", i, s);
                GameError::IntegerError(message)
            }
            sdl2::IntegerOrSdlError::SdlError(s) => GameError::SdlError(s),
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

impl From<AppDirsError> for GameError {
    fn from(e: AppDirsError) -> GameError {
        let errmessage = format!("{}", e);
        GameError::FilesystemError(errmessage)
    }
}
impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(e)
    }
}

impl From<toml::de::Error> for GameError {
    fn from(e: toml::de::Error) -> GameError {
        let errstr = format!("TOML decode error: {}", e.description());

        GameError::ConfigError(errstr)
    }
}

impl From<toml::ser::Error> for GameError {
    fn from(e: toml::ser::Error) -> GameError {
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

impl From<DecoderError> for GameError {
    fn from(e: DecoderError) -> GameError {
        let errstr = format!("Audio decoder error: {:?}", e);
        GameError::AudioError(errstr)
    }
}

impl From<image::ImageError> for GameError {
    fn from(e: image::ImageError) -> GameError {
        let errstr = format!("Image load error: {}", e.description());
        GameError::ResourceLoadError(errstr)
    }
}

impl From<gfx::PipelineStateError<std::string::String>> for GameError {
    fn from(e: gfx::PipelineStateError<std::string::String>) -> GameError {
        let errstr = format!(
            "Error constructing pipeline!\nThis should probably not be \
             happening; it probably means an error in a shader or \
             something.\nError was: {:?}",
            e
        );
        GameError::VideoError(errstr)
    }
}

impl From<gfx::mapping::Error> for GameError {
    fn from(e: gfx::mapping::Error) -> GameError {
        let errstr = format!("Buffer mapping error: {:?}", e);
        GameError::VideoError(errstr)
    }
}

impl<S, D> From<gfx::CopyError<S, D>> for GameError
where
    S: fmt::Debug,
    D: fmt::Debug,
{
    fn from(e: gfx::CopyError<S, D>) -> GameError {
        let errstr = format!("Memory copy error: {:?}", e);
        GameError::VideoError(errstr)
    }
}

impl From<gfx::CombinedError> for GameError {
    fn from(e: gfx::CombinedError) -> GameError {
        let errstr = format!("Texture+view load error: {}", e.description());
        GameError::VideoError(errstr)
    }
}

impl From<gfx::texture::CreationError> for GameError {
    fn from(e: gfx::texture::CreationError) -> GameError {
        gfx::CombinedError::from(e).into()
    }
}

impl From<gfx::ResourceViewError> for GameError {
    fn from(e: gfx::ResourceViewError) -> GameError {
        gfx::CombinedError::from(e).into()
    }
}

impl From<gfx::TargetViewError> for GameError {
    fn from(e: gfx::TargetViewError) -> GameError {
        gfx::CombinedError::from(e).into()
    }
}

impl<T> From<gfx::UpdateError<T>> for GameError
where
    T: fmt::Debug + fmt::Display + 'static,
{
    fn from(e: gfx::UpdateError<T>) -> GameError {
        let errstr = format!("Buffer update error: {}", e);
        GameError::VideoError(errstr)
    }
}

impl From<gfx::shade::ProgramError> for GameError {
    fn from(e: gfx::shade::ProgramError) -> GameError {
        GameError::ShaderProgramError(e)
    }
}



