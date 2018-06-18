//! The `conf` module contains functions for loading and saving game
//! configurations.
//!
//! A `Conf` struct is used to specify hardware setup stuff used to create
//! the window and other context information.
//!
//! By default a `ggez` game will search its resource paths for a `/conf.toml`
//! file and load values from it when the `Context` is created.  This file
//! must be complete (ie you cannot just fill in some fields and have the
//! rest be default) and provides a nice way to specify settings that
//! can be tweaked such as window resolution, multisampling options, etc.

use std::io;
use toml;

use GameResult;

/// Possible fullscreen modes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
/// use ggez::conf::{FullscreenType, WindowMode};
/// assert_eq!(
///     WindowMode::default(),
///     WindowMode {
///         dimensions: (800, 600),
///         min_dimensions: None,
///         max_dimensions: None,
///         fullscreen_type: FullscreenType::Off,
///         maximized: false,
///         hidden: false,
///         borderless: false,
///         always_on_top: false,
///     },
/// );
/// ```
#[derive(Debug, Copy, Clone, SmartDefault, PartialEq, Eq)]
pub struct WindowMode {
    /// Window's inner dimensions (drawable area).
    #[default = r#"(800, 600)"#]
    pub dimensions: (u32, u32),
    /// Window's minimum dimensions; `None` is no limit.
    #[default = r#"None"#]
    pub min_dimensions: Option<(u32, u32)>,
    /// Window's maximum dimensions; `None` is no limit.
    #[default = r#"None"#]
    pub max_dimensions: Option<(u32, u32)>,
    /// Fullscreen type.
    #[default = r#"FullscreenType::Off"#]
    pub fullscreen_type: FullscreenType,
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
/// use ggez::conf::{NumSamples, WindowSetup};
/// assert_eq!(
///     WindowSetup::default(),
///     WindowSetup {
///         title: "An easy, good game".to_owned(),
///         icon: "".to_owned(),
///         resizable: false,
///         transparent: false,
///         compatibility_profile: false,
///         vsync: true,
///         samples: NumSamples::One,
///         srgb: false,
///     },
/// );
/// ```
#[derive(Debug, Clone, SmartDefault, PartialEq, Eq)]
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
    /// Whether or not should the GL compatibility profile be used.
    #[default = r#"false"#]
    pub compatibility_profile: bool,
    /// Whether or not to enable vsync (vertical synchronization).
    #[default = r#"true"#]
    pub vsync: bool,
    /// Number of samples for multisample anti-aliasing.
    #[default = r#"NumSamples::One"#]
    pub samples: NumSamples,
    /// Whether or not to enable sRGB.
    #[default = r#"false"#]
    pub srgb: bool,
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

    /// Set if the GL compatibility profile should be used.
    pub fn compatibility_profile(mut self, compatibility_profile: bool) -> Self {
        self.compatibility_profile = compatibility_profile;
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

    /// Set if sRGB should be enabled.
    pub fn srgb(mut self, srgb: bool) -> Self {
        self.srgb = srgb;
        self
    }
}

/// Possible backends.
/// Currently, only OpenGL Core spec is supported,
/// but this lets you specify the version numbers.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Backend {
    /// Classical OpenGL, available on Windows, Linux, OS/X.
    OpenGL(u8, u8),
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
/// ```rust
/// use ggez::conf::{Backend, Conf, WindowMode, WindowSetup};
/// assert_eq!(
///     Conf::default(),
///     Conf {
///         window_mode: WindowMode::default(),
///         window_setup: WindowSetup::default(),
///         backend: Backend::OpenGL(3, 2),
///     },
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct Conf {
    /// Window setting information that can be set at runtime
    pub window_mode: WindowMode,
    /// Window setting information that must be set at init-time
    pub window_setup: WindowSetup,
    /// Backend configuration
    #[default = r#"Backend::OpenGL(3, 2)"#]
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

/// Custom serialization/deserialization.
mod custom_ser_de {
    use super::*;
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    /// Helper function to filter out defaults.
    fn some_if_ne<T: PartialEq>(result: T, cmp: T) -> Option<T> {
        match result == cmp {
            false => Some(result),
            true => None,
        }
    }

    /// Custom serialization/deserialization for `WindowMode`.
    mod window_mode_ser_de {
        use super::*;

        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
        struct DimensionsToml {
            width: u32,
            height: u32,
        }

        impl Into<(u32, u32)> for DimensionsToml {
            fn into(self) -> (u32, u32) {
                (self.width, self.height)
            }
        }

        impl From<(u32, u32)> for DimensionsToml {
            fn from(tuple: (u32, u32)) -> DimensionsToml {
                DimensionsToml {
                    width: tuple.0,
                    height: tuple.1,
                }
            }
        }

        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
        enum FullscreenTomlType {
            True,
            Desktop,
        }

        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
        struct FullscreenToml {
            #[serde(rename = "type")]
            fullscreen_type: FullscreenTomlType,
            monitor: Option<usize>,
        }

        impl Into<FullscreenType> for Option<FullscreenToml> {
            fn into(self) -> FullscreenType {
                if let Some(fs_type) = self {
                    let monitor = match fs_type.monitor {
                        Some(i) => MonitorId::Index(i),
                        None => MonitorId::Current,
                    };
                    match fs_type.fullscreen_type {
                        FullscreenTomlType::True => FullscreenType::True(monitor),
                        FullscreenTomlType::Desktop => FullscreenType::Desktop(monitor),
                    }
                } else {
                    FullscreenType::Off
                }
            }
        }

        impl From<FullscreenType> for Option<FullscreenToml> {
            fn from(fs_type: FullscreenType) -> Option<FullscreenToml> {
                match fs_type {
                    FullscreenType::Off => None,
                    FullscreenType::True(monitor) => Some(FullscreenToml {
                        fullscreen_type: FullscreenTomlType::True,
                        monitor: match monitor {
                            MonitorId::Index(i) => Some(i),
                            MonitorId::Current => None,
                        },
                    }),
                    FullscreenType::Desktop(monitor) => Some(FullscreenToml {
                        fullscreen_type: FullscreenTomlType::Desktop,
                        monitor: match monitor {
                            MonitorId::Index(i) => Some(i),
                            MonitorId::Current => None,
                        },
                    }),
                }
            }
        }

        impl Serialize for FullscreenType {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                <Option<FullscreenToml>>::from(*self).serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for FullscreenType {
            fn deserialize<D>(deserializer: D) -> Result<FullscreenType, D::Error>
            where
                D: Deserializer<'de>,
            {
                <Option<FullscreenToml>>::deserialize(deserializer).map(|w| w.into())
            }
        }

        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
        struct WindowModeToml {
            maximized: Option<bool>,
            hidden: Option<bool>,
            borderless: Option<bool>,
            always_on_top: Option<bool>,
            dimensions: Option<DimensionsToml>,
            min_dimensions: Option<DimensionsToml>,
            max_dimensions: Option<DimensionsToml>,
            fullscreen: Option<FullscreenType>,
        }

        impl Into<WindowMode> for WindowModeToml {
            fn into(self) -> WindowMode {
                let def = WindowMode::default();
                WindowMode {
                    dimensions: self.dimensions.map_or(def.dimensions, |dim| dim.into()),
                    min_dimensions: self.min_dimensions.map(|dim| dim.into()),
                    max_dimensions: self.max_dimensions.map(|dim| dim.into()),
                    fullscreen_type: self.fullscreen.unwrap_or(def.fullscreen_type),
                    maximized: self.maximized.unwrap_or(def.maximized),
                    hidden: self.hidden.unwrap_or(def.hidden),
                    borderless: self.borderless.unwrap_or(def.borderless),
                    always_on_top: self.always_on_top.unwrap_or(def.always_on_top),
                }
            }
        }

        impl From<WindowMode> for WindowModeToml {
            fn from(win_mode: WindowMode) -> Self {
                let def = WindowMode::default();
                WindowModeToml {
                    maximized: some_if_ne(win_mode.maximized, def.maximized),
                    hidden: some_if_ne(win_mode.hidden, def.hidden),
                    borderless: some_if_ne(win_mode.borderless, def.borderless),
                    always_on_top: some_if_ne(win_mode.always_on_top, def.always_on_top),
                    dimensions: some_if_ne(win_mode.dimensions, def.dimensions).map(|d| d.into()),
                    min_dimensions: win_mode.min_dimensions.map(|dim| dim.into()),
                    max_dimensions: win_mode.max_dimensions.map(|dim| dim.into()),
                    fullscreen: some_if_ne(win_mode.fullscreen_type, def.fullscreen_type),
                }
            }
        }

        impl Serialize for WindowMode {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                WindowModeToml::from(*self).serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for WindowMode {
            fn deserialize<D>(deserializer: D) -> Result<WindowMode, D::Error>
            where
                D: Deserializer<'de>,
            {
                WindowModeToml::deserialize(deserializer).map(|w| w.into())
            }
        }
    }

    /// Custom serialization/deserialization for `WindowSetup`.
    mod window_setup_ser_de {
        use super::*;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct WindowSetupToml {
            title: Option<String>,
            icon: Option<String>,
            resizable: Option<bool>,
            transparent: Option<bool>,
            compat: Option<bool>,
            vsync: Option<bool>,
            samples: Option<NumSamples>,
            srgb: Option<bool>,
        }

        impl Into<WindowSetup> for WindowSetupToml {
            fn into(self) -> WindowSetup {
                let def = WindowSetup::default();
                WindowSetup {
                    title: self.title.unwrap_or(def.title),
                    icon: self.icon.unwrap_or(def.icon),
                    resizable: self.resizable.unwrap_or(def.resizable),
                    transparent: self.transparent.unwrap_or(def.transparent),
                    compatibility_profile: self.compat.unwrap_or(def.compatibility_profile),
                    vsync: self.vsync.unwrap_or(def.vsync),
                    samples: self.samples.unwrap_or(def.samples),
                    srgb: self.srgb.unwrap_or(def.srgb),
                }
            }
        }

        impl From<WindowSetup> for WindowSetupToml {
            fn from(win_setup: WindowSetup) -> Self {
                let def = WindowSetup::default();
                WindowSetupToml {
                    title: some_if_ne(win_setup.title, def.title),
                    icon: some_if_ne(win_setup.icon, def.icon),
                    resizable: some_if_ne(win_setup.resizable, def.resizable),
                    transparent: some_if_ne(win_setup.transparent, def.transparent),
                    compat: some_if_ne(win_setup.compatibility_profile, def.compatibility_profile),
                    vsync: some_if_ne(win_setup.vsync, def.vsync),
                    samples: some_if_ne(win_setup.samples, def.samples),
                    srgb: some_if_ne(win_setup.srgb, def.srgb),
                }
            }
        }

        impl Serialize for WindowSetup {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                WindowSetupToml::from(self.clone()).serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for WindowSetup {
            fn deserialize<D>(deserializer: D) -> Result<WindowSetup, D::Error>
            where
                D: Deserializer<'de>,
            {
                WindowSetupToml::deserialize(deserializer).map(|w| w.into())
            }
        }
    }

    /// Custom serialization/deserialization for `Backend`.
    mod backend_ser_de {
        use super::*;

        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
        enum BackendType {
            OpenGL,
        }

        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
        struct BackendToml {
            #[serde(rename = "type")]
            backend_type: BackendType,
            version_major: u8,
            version_minor: u8,
        }

        impl Into<Backend> for BackendToml {
            fn into(self) -> Backend {
                match self.backend_type {
                    BackendType::OpenGL => Backend::OpenGL(self.version_major, self.version_minor),
                }
            }
        }

        impl From<Backend> for BackendToml {
            fn from(backend: Backend) -> BackendToml {
                match backend {
                    Backend::OpenGL(major, minor) => BackendToml {
                        backend_type: BackendType::OpenGL,
                        version_major: major,
                        version_minor: minor,
                    },
                }
            }
        }

        impl Serialize for Backend {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                BackendToml::from(*self).serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for Backend {
            fn deserialize<D>(deserializer: D) -> Result<Backend, D::Error>
            where
                D: Deserializer<'de>,
            {
                BackendToml::deserialize(deserializer).map(|w| w.into())
            }
        }
    }

    /// Custom serialization/deserialization for `Conf`.
    mod conf_ser_de {
        use super::*;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct ConfToml {
            window_mode: Option<WindowMode>,
            window_setup: Option<WindowSetup>,
            backend: Option<Backend>,
        }

        impl Into<Conf> for ConfToml {
            fn into(self) -> Conf {
                let def = Conf::default();
                Conf {
                    window_mode: self.window_mode.unwrap_or(def.window_mode),
                    window_setup: self.window_setup.unwrap_or(def.window_setup),
                    backend: self.backend.unwrap_or(def.backend),
                }
            }
        }

        impl From<Conf> for ConfToml {
            fn from(conf: Conf) -> Self {
                let def = Conf::default();
                ConfToml {
                    window_mode: some_if_ne(conf.window_mode, def.window_mode),
                    window_setup: some_if_ne(conf.window_setup, def.window_setup),
                    backend: some_if_ne(conf.backend, def.backend),
                }
            }
        }

        impl Serialize for Conf {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                ConfToml::from(self.clone()).serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for Conf {
            fn deserialize<D>(deserializer: D) -> Result<Conf, D::Error>
            where
                D: Deserializer<'de>,
            {
                ConfToml::deserialize(deserializer).map(|w| w.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde;
    use std::cmp::PartialEq;
    use std::fmt::Debug;

    fn round_trip_str<T>(thing: &T)
    where
        T: PartialEq + Debug + serde::Serialize + serde::de::DeserializeOwned,
    {
        let string = toml::to_string(&thing).unwrap();
        println!("-----\n{:?}\n\n{}", thing, string);
        assert_eq!(&toml::from_str::<T>(&string).unwrap(), thing);
    }

    fn round_trip_file(conf: &Conf) {
        let mut writer = Vec::new();
        conf.to_toml_file(&mut writer).unwrap();
        let mut reader = writer.as_slice();
        assert_eq!(&Conf::from_toml_file(&mut reader).unwrap(), conf);
    }

    #[test]
    fn encode_round_trip_conf() {
        let mut conf = Conf::new();
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.window_mode.maximized = true;
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.window_mode.fullscreen_type = FullscreenType::True(MonitorId::Current);
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.window_mode.fullscreen_type = FullscreenType::Desktop(MonitorId::Index(1));
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.window_mode.dimensions = (640, 480);
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.window_mode.max_dimensions = Some((640, 480));
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.window_setup.title = "Testing, testing, one, two".to_owned();
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.window_setup.samples = NumSamples::Sixteen;
        round_trip_str(&conf);
        round_trip_file(&conf);
        conf.backend = Backend::OpenGL(4, 5);
        round_trip_str(&conf);
        round_trip_file(&conf);
    }
}
