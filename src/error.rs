//! Error types and conversion functions.

use std;
use std::error::Error;
use std::fmt::{self, Display};

use gfx;
use glutin;
use winit;

use app_dirs2::AppDirsError;
use failure::{self, Backtrace, Fail};
use gilrs;
use image;
use lyon;
use rodio::decoder::DecoderError;
use toml;
use zip;

#[derive(Debug)]
pub struct GameError {
    inner: failure::Context<GameErrorKind>,
}

impl GameError {
    pub fn kind(&self) -> GameErrorKind {
        *self.inner.get_context()
    }
}

impl Fail for GameError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<GameErrorKind> for GameError {
    fn from(kind: GameErrorKind) -> Self {
        Self {
            inner: failure::Context::new(kind),
        }
    }
}

impl From<failure::Context<GameErrorKind>> for GameError {
    fn from(inner: failure::Context<GameErrorKind>) -> Self {
        Self { inner }
    }
}

/// An enum containing all kinds of game framework errors.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum GameErrorKind {
    /// An error in the filesystem layout
    #[fail(display = "A filesystem error occurred.")]
    FilesystemError,
    /// An error in the config file
    #[fail(display = "A configuration error occurred.")]
    ConfigError,
    /// Happens when an `EventsLoopProxy` attempts to
    /// wake up an `EventsLoop` that no longer exists.
    #[fail(display = "An event loop error occurred.")]
    EventLoopError,
    /// An error trying to load a resource, such as getting an invalid image file.
    #[fail(display = "A resource load error occurred.")]
    ResourceLoadError,
    /// Unable to find a resource; the Vec is the paths it searched for and associated errors
    #[fail(display = "An resource not found error occurred.")]
    ResourceNotFound,
    /// Something went wrong in the renderer
    #[fail(display = "A render error occurred.")]
    RenderError,
    /// Something went wrong in the audio playback
    #[fail(display = "An audio error occurred.")]
    AudioError,
    /// Something went wrong trying to set or get window properties.
    #[fail(display = "A window error occurred.")]
    WindowError,
    /// Something went wrong trying to create a window
    #[fail(display = "A window creation error occurred.")]
    WindowCreationError,
    /// Something went wrong trying to read from a file
    #[fail(display = "An IO error occurred.")]
    IOError,
    /// Something went wrong trying to load/render a font
    #[fail(display = "A font error occurred.")]
    FontError,
    /// Something went wrong applying video settings.
    #[fail(display = "A video error occurred.")]
    VideoError,
    /// Something went wrong compiling shaders
    #[fail(display = "A shader program error occurred.")]
    ShaderProgramError,
    /// Something went wrong with Gilrs
    #[fail(display = "A gamepad error occurred.")]
    GamepadError,
    /// Something went wrong with the `lyon` shape-tesselation library.
    #[fail(display = "A lyon error occurred.")]
    LyonError,
}

// impl fmt::Display for GameError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             GameError::ConfigError(ref s) => write!(f, "Config error: {}", s),
//             GameError::ResourceLoadError(ref s) => write!(f, "Error loading resource: {}", s),
//             GameError::ResourceNotFound(ref s, ref paths) => write!(
//                 f,
//                 "Resource not found: {}, searched in paths {:?}",
//                 s, paths
//             ),
//             GameError::WindowError(ref e) => write!(f, "Window creation error: {}", e),
//             _ => write!(f, "GameError {:?}", self),
//         }
//     }
// }

// impl Error for GameError {
//     fn cause(&self) -> Option<&dyn Error> {
//         match *self {
//             GameError::WindowCreationError(ref e) => Some(e),
//             GameError::IOError(ref e) => Some(e),
//             GameError::ShaderProgramError(ref e) => Some(e),
//             _ => None,
//         }
//     }
// }

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T = ()> = Result<T, GameError>;

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

// TODO: improve winit/glutin error handling.

impl From<winit::EventsLoopClosed> for GameError {
    fn from(_: glutin::EventsLoopClosed) -> GameError {
        let e = "An event loop proxy attempted to wake up an event loop that no longer exists."
            .to_owned();
        GameError::EventLoopError(e)
    }
}

impl From<glutin::CreationError> for GameError {
    fn from(s: glutin::CreationError) -> GameError {
        GameError::WindowCreationError(s)
    }
}

impl From<glutin::ContextError> for GameError {
    fn from(s: glutin::ContextError) -> GameError {
        GameError::RenderError(format!("OpenGL context error: {}", s))
    }
}

impl From<gilrs::Error> for GameError {
    // TODO: Better error type?
    fn from(s: gilrs::Error) -> GameError {
        let errstr = format!("Gamepad error: {}", s);
        GameError::GamepadError(errstr)
    }
}

impl From<lyon::lyon_tessellation::FillError> for GameError {
    fn from(s: lyon::lyon_tessellation::FillError) -> GameError {
        let errstr = format!(
            "Error while tesselating shape (did you give it an infinity or NaN?): {:?}",
            s
        );
        GameError::LyonError(errstr)
    }
}
