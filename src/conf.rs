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
#[serde(tag = "type")]
pub enum FullscreenType {
    /// Windowed mode.
    Off,
    /// Real fullscreen.
    True(MonitorId),
    /// Windowed fullscreen, generally preferred over real fullscreen
    /// these days 'cause it plays nicer with multiple monitors.
    Desktop(MonitorId),
}

/// Identifies a monitor connected to the system.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "id", content = "index")]
pub enum MonitorId {
    /// Monitor the window is currently in.
    Current,
    /// Monitor retrieved by it's index in the system.
    Index(usize),
}

/// A builder structure containing window settings
/// that can be set at runtime and changed with `graphics::set_mode()`.
///
/// Defaults:
///
/// ```rust
///use ggez::conf::{FullscreenType, WindowMode};
///assert_eq!(
///    WindowMode::default(),
///    WindowMode {
///        dimensions: (800, 600),
///        min_dimensions: None,
///        max_dimensions: None,
///        fullscreen_type: FullscreenType::Off,
///        maximized: false,
///        hidden: false,
///        borderless: false,
///        always_on_top: false,
///    },
///);
/// ```
#[derive(Debug, Copy, Clone, SmartDefault, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowMode {
    /// Whether or not to maximize the window.
    #[default = r#"false"#]
    pub maximized: bool,
    /// Whether or not the window is hidden.
    #[default = r#"false"#]
    pub hidden: bool,
    /// Whether or not to show window decorations.
    #[default = r#"false"#]
    pub borderless: bool,
    /// Whether or not the window is pinned to always be on top of other windows.
    #[default = r#"false"#]
    pub always_on_top: bool,
    /// Window's inner dimensions (drawable area); `None` will use system default.
    #[default = r#"(800, 600)"#]
    #[serde(with = "dimensions_serde")]
    pub dimensions: (u32, u32),
    /// Window's minimum dimensions; `None` is no limit.
    #[default = r#"None"#]
    #[serde(with = "dimensions_serde::option")]
    pub min_dimensions: Option<(u32, u32)>,
    /// Window's maximum dimensions; `None` is no limit.
    #[default = r#"None"#]
    #[serde(with = "dimensions_serde::option")]
    pub max_dimensions: Option<(u32, u32)>,
    /// Fullscreen type.
    #[default = r#"FullscreenType::Off"#]
    pub fullscreen_type: FullscreenType,
}

impl WindowMode {
    /// Set if window should be maximized.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Set if window should be hidden.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Set whether or not to show window decorations.
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.borderless = borderless;
        self
    }

    /// Set if window should pinned to always be on top of other windows.
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    /// Set default window size, or screen resolution in true fullscreen mode.
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.dimensions = (width, height);
        self
    }

    /// Set minimum window dimensions for windowed mode.
    pub fn min_dimensions(mut self, width: u32, height: u32) -> Self {
        self.min_dimensions = Some((width, height));
        self
    }

    /// Set maximum window dimensions for windowed mode.
    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_dimensions = Some((width, height));
        self
    }

    /// Set the fullscreen type.
    pub fn fullscreen_type(mut self, fullscreen_type: FullscreenType) -> Self {
        self.fullscreen_type = fullscreen_type;
        self
    }
}

/// A builder structure containing window settings
/// that must be set at init time and (mostly) cannot be changed afterwards.
///
/// Defaults:
///
/// ```rust
///use ggez::conf::{NumSamples, WindowSetup};
///assert_eq!(
///    WindowSetup::default(),
///    WindowSetup {
///        title: "An easy, good game".to_owned(),
///        icon: "".to_owned(),
///        resizable: false,
///        transparent: false,
///        vsync: true,
///        samples: NumSamples::One,
///    },
///);
/// ```
#[derive(Debug, Clone, SmartDefault, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowSetup {
    /// The window title.
    #[default = r#""An easy, good game".to_owned()"#]
    pub title: String,
    /// A file path to the window's icon.
    /// It is rooted in the `resources` directory (see the `filesystem` module for details),
    /// and an empty string results in a blank/default icon.
    #[default = r#""".to_owned()"#]
    pub icon: String,
    /// Whether or not the window is resizable.
    #[default = r#"false"#]
    pub resizable: bool,
    /// Whether or not should the window's background be transparent.
    #[default = r#"false"#]
    pub transparent: bool,
    /// Whether or not to enable vsync (vertical synchronization).
    #[default = r#"true"#]
    pub vsync: bool,
    /// Number of samples for multisample anti-aliasing.
    #[default = r#"NumSamples::One"#]
    pub samples: NumSamples,
}

impl WindowSetup {
    /// Set window's title.
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    /// Set the window's icon.
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_owned();
        self
    }

    /// Set whether or not the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set if window's background should be transparent.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Set if vsync is enabled.
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Set number of samples for multisample anti-aliasing.
    pub fn samples(mut self, samples: NumSamples) -> Self {
        self.samples = samples;
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
///     backend: Backend::OpenGL(3, 2),
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
        file.read_to_string(&mut s)?;
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

/// Custom serialization for `(u32, u32)` and `Option<(u32, u32)>`,
/// to allow using these with `toml` crate.
mod dimensions_serde {
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    #[derive(Deserialize, Serialize)]
    #[serde(tag = "option")]
    enum Dim {
        Some { width: u32, height: u32 },
        None,
    }

    pub(crate) fn serialize<S>(dimensions: &(u32, u32), serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dim = Dim::Some {
            width: dimensions.0,
            height: dimensions.1,
        };
        dim.serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<(u32, u32), D::Error>
    where
        D: Deserializer<'de>,
    {
        let result = Dim::deserialize(deserializer).map(|dim| {
            if let Dim::Some { width, height } = dim {
                (width, height)
            } else {
                super::WindowMode::default().dimensions
            }
        });
        result
    }

    pub(crate) mod option {
        use super::*;

        pub(crate) fn serialize<S>(
            dimensions: &Option<(u32, u32)>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match dimensions {
                Some(dimensions) => {
                    let dim = Dim::Some {
                        width: dimensions.0,
                        height: dimensions.1,
                    };
                    dim.serialize(serializer)
                }
                None => Dim::None.serialize(serializer),
            }
        }

        pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<(u32, u32)>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let result = Dim::deserialize(deserializer).map(|dim| {
                if let Dim::Some { width, height } = dim {
                    Some((width, height))
                } else {
                    None
                }
            });
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde;
    use std::cmp::PartialEq;
    use std::fmt::Debug;

    #[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
    struct Test {
        #[serde(with = "dimensions_serde")]
        dims: (u32, u32),
    }

    #[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
    struct TestOption {
        #[serde(with = "dimensions_serde::option")]
        dims_opt: Option<(u32, u32)>,
    }

    #[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
    struct TestBunch {
        #[serde(with = "dimensions_serde::option")]
        dims_opt1: Option<(u32, u32)>,
        #[serde(with = "dimensions_serde")]
        dims: (u32, u32),
        #[serde(with = "dimensions_serde::option")]
        dims_opt2: Option<(u32, u32)>,
        #[serde(with = "dimensions_serde::option")]
        dims_opt3: Option<(u32, u32)>,
        #[serde(with = "dimensions_serde::option")]
        dims_opt4: Option<(u32, u32)>,
    }

    #[derive(SmartDefault, Debug, Serialize, Deserialize, PartialEq)]
    struct TestMix {
        a_bool: bool,
        #[default = r#"FullscreenType::Off"#]
        fs_type: FullscreenType,
        #[serde(with = "dimensions_serde")]
        dims: (u32, u32),
        #[serde(with = "dimensions_serde::option")]
        dims_opt: Option<(u32, u32)>,
    }

    fn round_trip_str<T>(thing: T)
    where
        T: PartialEq + Debug + serde::Serialize + serde::de::DeserializeOwned,
    {
        let string = toml::to_string(&thing).unwrap();
        //println!("-----\n{:?}\n\n{}", thing, string);
        assert_eq!(toml::from_str::<T>(&string).unwrap(), thing);
    }

    #[test]
    fn encode_individual_types() {
        round_trip_str(FullscreenType::Off);
        round_trip_str(FullscreenType::Desktop(MonitorId::Current));
        round_trip_str(FullscreenType::Desktop(MonitorId::Index(1)));
        round_trip_str(Test::default());
        round_trip_str(TestOption::default());
        round_trip_str(TestOption {
            dims_opt: Some((1, 2)),
        });
        round_trip_str(TestBunch::default());
        round_trip_str(TestBunch {
            dims_opt1: Some((1, 2)),
            dims: (3, 4),
            dims_opt2: Some((5, 6)),
            dims_opt3: None,
            dims_opt4: Some((7, 8)),
        });
        round_trip_str(TestMix::default());
        round_trip_str(TestMix {
            a_bool: true,
            fs_type: FullscreenType::Desktop(MonitorId::Index(1)),
            dims: (1, 2),
            dims_opt: Some((3, 4)),
        });
    }

    /// Tries to encode and decode a `Conf` object
    /// and makes sure it gets the same result it had.
    #[test]
    fn encode_round_trip() {
        round_trip_str(Conf::new());
        let c1 = Conf::new();
        let mut writer = Vec::new();
        let _c = c1.to_toml_file(&mut writer).unwrap();
        let mut reader = writer.as_slice();
        let c2 = Conf::from_toml_file(&mut reader).unwrap();
        assert_eq!(c1, c2);
    }
}
