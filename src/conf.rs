//! The `conf` module contains functions for loading and saving game
//! configurations.
//!
//! A `Conf` struct is used to specify hardware setup stuff used to create
//! the window and other context information.
//!
//! By default a ggez game will search its resource paths for a `/conf.toml`
//! file and load values from it when the `Context` is created.  This file
//! must be complete (ie you cannot just fill in some fields and have the
//! rest be default) and provides a nice way to specify settings that
//! can be tweaked such as window resolution, multisampling options, etc.

use std::io;
use toml;

use GameResult;

/// Possible fullscreen modes.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FullscreenType {
    /// Windowed mode
    Windowed,
    /// Real fullscreen
    True,
    /// Windowed fullscreen, generally preferred over real fullscreen
    /// these days 'cause it plays nicer with multiple monitors.
    Desktop,
}


/// A builder structure containing window settings
/// that can be set at runtime and changed with `graphics::set_mode()`
///
/// Defaults:
///
/// ```rust,ignore
/// WindowMode {
///     width: 800,
///     height: 600,
///     borderless: false,
///     fullscreen_type: FullscreenType::Windowed,
///     vsync: true,
///     min_width: 0,
///     max_width: 0,
///     min_height: 0,
///     max_height: 0,
/// }
/// ```
#[derive(Debug, Copy, Clone, SmartDefault, Serialize, Deserialize, PartialEq)]
pub struct WindowMode {
    /// Window width
    #[default = r#"800.0"#]
    pub width: f32,
    /// Window height
    #[default = r#"600.0"#]
    pub height: f32,
    /// Whether or not to maximize the window
    #[default = r#"false"#]
    pub maximized: bool,
    /// Fullscreen type
    #[default = r#"FullscreenType::Windowed"#]
    pub fullscreen_type: FullscreenType,
    /// Whether or not to show window decorations
    #[default = r#"false"#]
    pub borderless: bool,
    /// Minimum width for resizable windows; 0 means no limit
    #[default = r#"0.0"#]
    pub min_width: f32,
    /// Minimum height for resizable windows; 0 means no limit
    #[default = r#"0.0"#]
    pub min_height: f32,
    /// Maximum width for resizable windows; 0 means no limit
    #[default = r#"0.0"#]
    pub max_width: f32,
    /// Maximum height for resizable windows; 0 means no limit
    #[default = r#"0.0"#]
    pub max_height: f32,
    /// Whether or not to scale all "pixel" coordinates to deal with
    /// high DPI screens.
    ///
    /// A very good overview of this is available in
    /// [the `winit` docs](https://docs.rs/winit/0.16.1/winit/dpi/index.html).
    /// If this is false (the default), one pixel in ggez equates to one
    /// physical pixel on the screen.  If it is `true`, then ggez will
    /// scale *all* pixel coordinates by the scaling factor returned by
    /// `graphics::get_hidpi_factor()`.
    #[default = r"false"]
    pub hidpi: bool,
}

impl WindowMode {
    /// Set default window size, or screen resolution in true fullscreen mode
    pub fn dimensions(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set whether the window should be maximized
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Set the fullscreen type
    pub fn fullscreen_type(mut self, fullscreen_type: FullscreenType) -> Self {
        self.fullscreen_type = fullscreen_type;
        self
    }

    /// Set borderless
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.borderless = borderless;
        self
    }

    /// Set minimum window dimensions for windowed mode
    pub fn min_dimensions(mut self, width: f32, height: f32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    /// Set maximum window dimensions for windowed mode
    pub fn max_dimensions(mut self, width: f32, height: f32) -> Self {
        self.max_width = width;
        self.max_height = height;
        self
    }

    /// Sets whether or not to allow hidpi.
    pub fn hidpi(mut self, hidpi: bool) -> Self {
        self.hidpi = hidpi;
        self
    }
}

/// A builder structure containing window settings
/// that must be set at init time and cannot be changed afterwards.
///
/// Defaults:
///
/// TODO: Update docs and defaults
///
/// ```rust,ignore
/// WindowSetup {
///     title: "An easy, good game".to_owned(),
///     icon: "".to_owned(),
///     resizable: false,
///     allow_highdpi: true,
///     samples: NumSamples::One,
/// }
/// ```
#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize, PartialEq)]
pub struct WindowSetup {
    /// The window title.
    #[default = r#""An easy, good game".to_owned()"#]
    pub title: String,
    /*/// Whether or not the window is resizable
    #[default = r#"false"#]
    pub resizable: bool,*/ // TODO: winit #540
    /// Number of samples for multisample anti-aliasing
    #[default = r#"NumSamples::One"#]
    pub samples: NumSamples,
    /// Whether or not to enable vsync
    #[default = r#"true"#]
    pub vsync: bool,
    /// Whether or not should the window's background be transparent
    #[default = r#"false"#]
    pub transparent: bool,
    /// A file path to the window's icon.
    /// It is rooted in the `resources` directory (see the `filesystem` module for details),
    /// and an empty string results in a blank/default icon.
    #[default = r#""".to_owned()"#]
    pub icon: String,
    /// Whether or not to enable sRGB (gamma corrected color)
    /// handling on the display.
    #[default = r#"true"#]
    pub srgb: bool,
}

impl WindowSetup {
    /// Set window title
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    /*/// Set resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }*/
    // TODO: winit #540

    /// Set number of samples
    ///
    /// Returns None if given an invalid value
    /// (valid values are powers of 2 from 1 to 16)
    pub fn samples(mut self, samples: NumSamples) -> Self {
        self.samples = samples;
        self
    }

    /// Set if vsync is enabled.
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Set if window background should be transparent.
    ///
    /// TODO: Is this necessary?  Do we ever want this?
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Set the window's icon.
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_owned();
        self
    }

    /// Set sRGB color mode.
    pub fn srgb(mut self, active: bool) -> Self {
        self.srgb = active;
        self
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


impl Backend {
    /// Set OpenGL backend and version.
    pub fn opengl(self, new_major: u8, new_minor: u8) -> Self {
        match self {
            Backend::OpenGL {..} => {
                Backend::OpenGL {
                    major: new_major,
                    minor: new_minor,
                }
            }
        }
    }
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
    /// Create a NumSamples from a number.
    /// Returns None if i is invalid.
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
/// ```rust,ignore
/// Conf {
///     window_mode: WindowMode::default(),
///     window_setup: WindowSetup::default(),
///     backend: Backend::OpenGL{ major: 3, minor: 2, srgb: true},
/// }
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
    /// Same as Conf::default()
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a TOML file from the given `Read` and attempts to parse
    /// a `Conf` from it.
    pub fn from_toml_file<R: io::Read>(file: &mut R) -> GameResult<Conf> {
        let mut s = String::new();
        let _ = file.read_to_string(&mut s)?;
        let decoded = toml::from_str(&s)?;
        Ok(decoded)
    }

    /// Saves the `Conf` to the given `Write` object,
    /// formatted as TOML.
    pub fn to_toml_file<W: io::Write>(&self, file: &mut W) -> GameResult {
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
    fn headless_encode_round_trip() {
        let c1 = conf::Conf::new();
        let mut writer = Vec::new();
        let _c = c1.to_toml_file(&mut writer).unwrap();
        let mut reader = writer.as_slice();
        let c2 = conf::Conf::from_toml_file(&mut reader).unwrap();
        assert_eq!(c1, c2);
    }
}
