//! The `conf` module contains functions for loading and saving game
//! configurations.
//!
//! A [`Conf`](struct.Conf.html) struct is used to specify hardware setup stuff used to create
//! the window and other context information.
//!
//! By default a ggez game will search its resource paths for a `/conf.toml`
//! file and load values from it when the [`Context`](../struct.Context.html) is created.
//!  This file must be complete (i.e., you cannot just fill in some fields and have the
//! rest be default) and provides a nice way to specify settings that
//! can be tweaked such as window resolution, multisampling options, etc.

use std::io;
use toml;

use GameResult;

/// Possible fullscreen modes.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FullscreenType {
    /// Windowed mode
    Off,
    /// Real fullscreen
    True,
    /// Windowed fullscreen, generally preferred over real fullscreen
    /// these days 'cause it plays nicer with multiple monitors.
    Desktop,
}

use sdl2::video::FullscreenType as SdlFullscreenType;
impl From<SdlFullscreenType> for FullscreenType {
    fn from(other: SdlFullscreenType) -> Self {
        match other {
            SdlFullscreenType::Off => FullscreenType::Off,
            SdlFullscreenType::True => FullscreenType::True,
            SdlFullscreenType::Desktop => FullscreenType::Desktop,
        }
    }
}

impl From<FullscreenType> for SdlFullscreenType {
    fn from(other: FullscreenType) -> Self {
        match other {
            FullscreenType::Off => SdlFullscreenType::Off,
            FullscreenType::True => SdlFullscreenType::True,
            FullscreenType::Desktop => SdlFullscreenType::Desktop,
        }
    }
}

/// A builder structure containing window settings
/// that can be set at runtime and changed with [`graphics::set_mode()`](../graphics/fn.set_mode.html)
///
/// Defaults:
///
/// ```rust
/// # use ggez::conf::*;
/// let wm = WindowMode {
///     width: 800,
///     height: 600,
///     borderless: false,
///     fullscreen_type: FullscreenType::Off,
///     vsync: true,
///     min_width: 0,
///     max_width: 0,
///     min_height: 0,
///     max_height: 0,
/// };
/// assert_eq!(wm, WindowMode::default())
/// ```
#[derive(Debug, Copy, Clone, SmartDefault, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowMode {
    /// Window width
    #[default = r#"800"#]
    pub width: u32,
    /// Window height
    #[default = r#"600"#]
    pub height: u32,
    /// Whether or not to show window decorations
    #[default = r#"false"#]
    pub borderless: bool,
    /// Fullscreen type
    #[default = r#"FullscreenType::Off"#]
    pub fullscreen_type: FullscreenType,
    /// Whether or not to enable vsync
    #[default = r#"true"#]
    pub vsync: bool,
    /// Minimum width for resizable windows; 0 means no limit
    #[default = r#"0"#]
    pub min_width: u32,
    /// Minimum height for resizable windows; 0 means no limit
    #[default = r#"0"#]
    pub min_height: u32,
    /// Maximum width for resizable windows; 0 means no limit
    #[default = r#"0"#]
    pub max_width: u32,
    /// Maximum height for resizable windows; 0 means no limit
    #[default = r#"0"#]
    pub max_height: u32,
}

impl WindowMode {
    /// Set borderless
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.borderless = borderless;
        self
    }

    /// Set the fullscreen type
    pub fn fullscreen_type(mut self, fullscreen_type: FullscreenType) -> Self {
        self.fullscreen_type = fullscreen_type;
        self
    }

    /// Set vsync
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Set default window size, or screen resolution in fullscreen mode
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set minimum window dimensions for windowed mode
    pub fn min_dimensions(mut self, width: u32, height: u32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    /// Set maximum window dimensions for windowed mode
    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_width = width;
        self.max_height = height;
        self
    }
}

/// A builder structure containing window settings
/// that must be set at init time and cannot be changed afterwards.
///
/// Defaults:
///
/// ```rust
/// # use ggez::conf::*;
/// let ws = WindowSetup {
///     title: "An easy, good game".to_owned(),
///     icon: "".to_owned(),
///     resizable: false,
///     allow_highdpi: true,
///     samples: NumSamples::One,
/// };
/// assert_eq!(ws, WindowSetup::default())
/// ```
#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowSetup {
    /// The window title.
    #[default = r#""An easy, good game".to_owned()"#]
    pub title: String,
    /// A file path to the window's icon.
    /// It is rooted in the `resources` directory (see the [`filesystem`](../filesystem/index.html)
    /// module for details), and an empty string results in a blank/default icon.
    #[default = r#""".to_owned()"#]
    pub icon: String,
    /// Whether or not the window is resizable
    #[default = r#"false"#]
    pub resizable: bool,
    /// Whether or not to allow high DPI mode when creating the window
    #[default = r#"true"#]
    pub allow_highdpi: bool,
    /// Number of samples for multisample anti-aliasing
    #[default = r#"NumSamples::One"#]
    pub samples: NumSamples,
}

impl WindowSetup {
    /// Set window title
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    /// Set the window's icon.
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_owned();
        self
    }

    /// Set resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set allow_highdpi
    pub fn allow_highdpi(mut self, allow: bool) -> Self {
        self.allow_highdpi = allow;
        self
    }

    /// Set number of samples
    ///
    /// Returns None if given an invalid value
    /// (valid values are powers of 2 from 1 to 16)
    pub fn samples(mut self, samples: u32) -> Option<Self> {
        match NumSamples::from_u32(samples) {
            Some(s) => {
                self.samples = s;
                Some(self)
            }
            None => None,
        }
    }
}

/// Possible backends.
/// Currently, only OpenGL Core spec is supported,
/// but this lets you specify the version numbers.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, SmartDefault)]
#[serde(tag = "type")]
pub enum Backend {
    /// Defaults to OpenGL 3.2, which is supported by basically
    /// every machine since 2009 or so (apart from the ones that don't)
    #[default]
    OpenGL {
        /// OpenGL major version
        #[default = r#"3"#]
        major: u8,
        /// OpenGL minor version
        #[default = r#"2"#]
        minor: u8,
    },
}

/// The possible number of samples for multisample anti-aliasing
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NumSamples {
    /// One sample
    One = 1,
    /// Two samples
    Two = 2,
    /// Four samples
    Four = 4,
    /// Eight samples
    Eight = 8,
    /// Sixteen samples
    Sixteen = 16,
}

impl NumSamples {
    /// Create a `NumSamples` from a number.
    /// Returns None if `i` is invalid.
    pub fn from_u32(i: u32) -> Option<NumSamples> {
        match i {
            1 => Some(NumSamples::One),
            2 => Some(NumSamples::Two),
            4 => Some(NumSamples::Four),
            8 => Some(NumSamples::Eight),
            16 => Some(NumSamples::Sixteen),
            _ => None,
        }
    }
}

/// A structure containing configuration data
/// for the game engine.
///
/// Defaults:
///
/// ```rust
/// # use ggez::conf::*;
/// let c = Conf {
///     window_mode: WindowMode::default(),
///     window_setup: WindowSetup::default(),
///     backend: Backend::OpenGL{major: 3, minor: 2},
/// };
/// assert_eq!(c, Conf::default())
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, SmartDefault)]
pub struct Conf {
    /// Window setting information that can be set at runtime
    pub window_mode: WindowMode,
    /// Window setting information that must be set at init-time
    pub window_setup: WindowSetup,
    /// Backend configuration
    pub backend: Backend,
}

impl Conf {
    /// Same as `Conf::default()`
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a TOML file from the given `Read` and attempts to parse
    /// a `Conf` from it.
    pub fn from_toml_file<R: io::Read>(file: &mut R) -> GameResult<Conf> {
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let decoded = toml::from_str(&s)?;
        Ok(decoded)
    }

    /// Saves the `Conf` to the given `Write` object,
    /// formatted as TOML.
    pub fn to_toml_file<W: io::Write>(&self, file: &mut W) -> GameResult<()> {
        let s = toml::to_vec(self)?;
        file.write_all(&s)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use conf;

    /// Tries to encode and decode a `Conf` object
    /// and makes sure it gets the same result it had.
    #[test]
    fn encode_round_trip() {
        let c1 = conf::Conf::new();
        let mut writer = Vec::new();
        let _c = c1.to_toml_file(&mut writer).unwrap();
        let mut reader = writer.as_slice();
        let c2 = conf::Conf::from_toml_file(&mut reader).unwrap();
        assert_eq!(c1, c2);
    }
}
