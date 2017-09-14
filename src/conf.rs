//! The `conf` module contains functions for loading and saving game
//! configurations.
//!
//! A `Conf` struct is used to specify hardware setup stuff used to create
//! the window and other context information.

use std::io;
use toml;

use GameResult;

/// A structure containing configuration data
/// for the game engine.
///
/// Defaults:
///
/// ```rust,ignore
/// Conf {
///     window_title: "An easy, good game"
///     window_icon: ""
///     window_height: 600
///     window_width: 800
///     vsync: true
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, SmartDefault)]
pub struct Conf {
    /// The window title.
    #[default = r#""An easy, good game".to_owned()"#]
    pub window_title: String,
    /// A file path to the window's icon.
    /// It is rooted in the `resources` directory (see the `filesystem` module for details),
    /// and an empty string results in a blank/default icon.
    #[default = r#""".to_owned()"#]
    pub window_icon: String,
    /// The window's height
    #[default = "600"]
    pub window_height: u32,
    /// The window's width
    #[default = "800"]
    pub window_width: u32,
    /// Whether or not the graphics draw rate should be
    /// synchronized with the monitor's draw rate.
    #[default = "true"]
    pub vsync: bool,
    #[default = "true"]
    pub resizable: bool,
    /* To implement still.
     * window_borderless: bool,
     * window_resizable: bool,
     * window_fullscreen: bool,
     *
     * Modules to enable
     * modules_audio: bool,
     * modules_event: bool,
     * modules_graphics: bool,
     * modules_image: bool,
     * modules_joystic: bool,
     * modules_keyboard: bool,
     * modules_mouse: bool,
     * modules_sound: bool,
     * modules_system: bool,
     * modules_timer: bool,
     * modules_video: bool,
     * modules_window: bool,
     * modules_thread: bool, */
}

// impl Default for Conf {
//     /// Create a new Conf with some vague defaults.
//     ///
//     /// ```rust,ignore
//     /// Conf {
//     ///     window_title: "An easy, good game"
//     ///     window_icon: ""
//     ///     window_height: 600
//     ///     window_width: 800
//     ///     vsync: true
//     /// }
//     /// ```
//     fn default() -> Self {
//         Conf {
//             window_title: String::from("An easy, good game"),
//             window_icon: String::from(""),
//             window_height: 600,
//             window_width: 800,
//             vsync: true,
//             resizable: false,
//         }

//     }
// }

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
        //println!("{}", String::from_utf8_lossy(&writer));
        let mut reader = writer.as_slice();
        let c2 = conf::Conf::from_toml_file(&mut reader).unwrap();
        assert_eq!(c1, c2);
    }
}
