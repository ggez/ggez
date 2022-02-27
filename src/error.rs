//! Error types and conversion functions.
use rodio::decoder::DecoderError;
use rodio::PlayError;
use std::sync::Arc;
use thiserror::Error;

/// An enum containing all kinds of game framework errors.
#[derive(Error, Debug, Clone)]
pub enum GameError {
    /// An error in the filesystem layout
    #[error("error in filesystem layout: {0}")]
    FilesystemError(String),
    /// An error in the config file
    #[error("error in config file: {0}")]
    ConfigError(String),
    /// Happens when an `winit::event_loop::EventLoopProxy` attempts to
    /// wake up an `winit::event_loop::EventLoop` that no longer exists.
    #[error("error in event loop: {0}")]
    EventLoopError(String),
    /// An error trying to load a resource, such as getting an invalid image file.
    #[error("failed to load resource: {0}")]
    ResourceLoadError(String),
    /// Unable to find a resource; the `Vec` is the paths it searched for and associated errors
    #[error("resource at '{1:#?}' not found: {0}")]
    ResourceNotFound(String, Vec<(std::path::PathBuf, GameError)>),
    /// Something went wrong in the renderer
    #[error("renderer error: {0}")]
    RenderError(String),
    /// Something went wrong when requesting a logical device from the graphics API.
    #[error("failed to request logical device")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    /// Something went wrong in the audio playback
    #[error("failed to play audio: {0}")]
    AudioError(String),
    /// Something went wrong trying to set or get window properties.
    #[error("windowing error: {0}")]
    WindowError(String),
    /// Something went wrong trying to create a window
    #[error("failed to create window")]
    WindowCreationError(#[from] Arc<winit::error::OsError>),
    /// Something went wrong trying to read from a file
    #[allow(clippy::upper_case_acronyms)]
    #[error("failed to read file")]
    IOError(#[from] Arc<std::io::Error>),
    /// Something went wrong trying to load a font
    #[error("failed to load font")]
    FontError(#[from] wgpu_glyph::ab_glyph::InvalidFont),
    /// Something went wrong applying video settings.
    #[error("failed to apply video settings: {0}")]
    VideoError(String),
    /// Something went wrong with the `gilrs` gamepad-input library.
    #[error("gamepad error: {0}")]
    GamepadError(String),
    /// Something went wrong with the `lyon` shape-tesselation library.
    #[error("lyon tesellation error: {0}")]
    LyonError(String),
    /// A custom error type for use by users of ggez.
    /// This lets you handle custom errors that may happen during your game (such as, trying to load a malformed file for a level)
    /// using the same mechanism you handle ggez's other errors.
    ///
    /// Please include an informative message with the error.
    #[error("error: {0}")]
    CustomError(String),
}

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T = ()> = Result<T, GameError>;

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(Arc::new(e))
    }
}

impl From<toml::de::Error> for GameError {
    fn from(e: toml::de::Error) -> GameError {
        let errstr = format!("TOML decode error: {}", e.to_string());

        GameError::ConfigError(errstr)
    }
}

impl From<toml::ser::Error> for GameError {
    fn from(e: toml::ser::Error) -> GameError {
        let errstr = format!("TOML error (possibly encoding?): {}", e.to_string());
        GameError::ConfigError(errstr)
    }
}

impl From<zip::result::ZipError> for GameError {
    fn from(e: zip::result::ZipError) -> GameError {
        let errstr = format!("Zip error: {}", e.to_string());
        GameError::ResourceLoadError(errstr)
    }
}

impl From<DecoderError> for GameError {
    fn from(e: DecoderError) -> GameError {
        let errstr = format!("Audio decoder error: {:?}", e);
        GameError::AudioError(errstr)
    }
}

impl From<PlayError> for GameError {
    fn from(e: PlayError) -> GameError {
        let errstr = format!("Audio playing error: {:?}", e);
        GameError::AudioError(errstr)
    }
}

impl From<image::ImageError> for GameError {
    fn from(e: image::ImageError) -> GameError {
        let errstr = format!("Image load error: {}", e.to_string());
        GameError::ResourceLoadError(errstr)
    }
}

impl From<winit::error::OsError> for GameError {
    fn from(s: winit::error::OsError) -> GameError {
        GameError::WindowCreationError(Arc::new(s))
    }
}

impl From<gilrs::Error> for GameError {
    fn from(s: gilrs::Error) -> GameError {
        let errstr = format!("Gamepad error: {}", s);
        GameError::GamepadError(errstr)
    }
}

impl From<lyon::lyon_tessellation::TessellationError> for GameError {
    fn from(s: lyon::lyon_tessellation::TessellationError) -> GameError {
        let errstr = format!(
            "Error while tesselating shape (did you give it an infinity or NaN?): {:?}",
            s
        );
        GameError::LyonError(errstr)
    }
}

impl From<lyon::lyon_tessellation::geometry_builder::GeometryBuilderError> for GameError {
    fn from(s: lyon::lyon_tessellation::geometry_builder::GeometryBuilderError) -> GameError {
        let errstr = format!(
            "Error while building geometry (did you give it too many vertices?): {:?}",
            s
        );
        GameError::LyonError(errstr)
    }
}
