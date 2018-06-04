//! Error types and conversion functions.

use std;
use std::error::Error;
use std::fmt;

use gfx;
use glutin;
use winit;

use app_dirs2::AppDirsError;
use image;
use rodio::decoder::DecoderError;
use toml;
use zip;

/// An enum containing all kinds of game framework errors.
#[derive(Debug)]
pub enum GameError {
    /// An error in the filesystem layout
    FilesystemError(String),
    /// An error in the config file
    ConfigError(String),
    /// Happens when an `EventsLoopProxy` attempts to
    /// wake up an `EventsLoop` that no longer exists.
    EventLoopError(String),
    /// An error trying to parse a resource
    ResourceLoadError(String),
    /// Unable to find a resource; the Vec is the paths it searched for and associated errors
    ResourceNotFound(String, Vec<(std::path::PathBuf, GameError)>),
    /// Something went wrong in the renderer
    RenderError(String),
    /// Something went wrong in the audio playback
    AudioError(String),
    /// Something went wrong trying to create a window
    WindowError(glutin::CreationError),
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
            GameError::ConfigError(ref s) => write!(f, "Config error: {}", s),
            GameError::ResourceLoadError(ref s) => write!(f, "Error loading resource: {}", s),
            GameError::ResourceNotFound(ref s, ref paths) => write!(
                f,
                "Resource not found: {}, searched in paths {:?}",
                s, paths
            ),
            GameError::WindowError(ref e) => write!(f, "Window creation error: {}", e),
            _ => write!(f, "GameError {:?}", self),
        }
    }
}

impl Error for GameError {
    fn description(&self) -> &str {
        match *self {
            GameError::FilesystemError(_) => "Filesystem error",
            GameError::ConfigError(_) => "Config file error",
            GameError::EventLoopError(_) => "Event loop error",
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
            GameError::WindowError(ref e) => Some(e),
            GameError::IOError(ref e) => Some(e),
            GameError::ShaderProgramError(ref e) => Some(e),
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

impl From<winit::EventsLoopClosed> for GameError {
    fn from(_: glutin::EventsLoopClosed) -> GameError {
        let e = "An event loop proxy attempted to wake up an event loop that no longer exists."
            .to_owned();
        GameError::EventLoopError(e)
    }
}

impl From<glutin::CreationError> for GameError {
    fn from(s: glutin::CreationError) -> GameError {
        GameError::WindowError(s)
    }
}

impl From<glutin::ContextError> for GameError {
    fn from(s: glutin::ContextError) -> GameError {
        GameError::RenderError(format!("OpenGL context error: {}", s))
    }
}
