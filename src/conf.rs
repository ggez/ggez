//! The `conf` module contains functions for loading and saving game
//! configurations.

use std::io;
use toml;

use GameResult;

/// A structure containing configuration data
/// for the game engine.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Conf {
    /// The window title.
    pub window_title: String,
    /// A file path to the window's icon.
    pub window_icon: String,
    /// The window's default height
    pub window_height: u32,
    /// The window's default width
    pub window_width: u32,
    /// Whether or not the graphics draw rate should be
    /// synchronized with the monitor's draw rate.
    pub vsync: bool, 
    /* To implement still.
                            * window_borderless: bool,
                            * window_resizable: bool,
                            * window_fullscreen: bool,
                            * window_vsync: bool,
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

impl Default for Conf {
    /// Create a new Conf with some vague defaults.
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
    fn default() -> Self {
        Conf {
            window_title: String::from("An easy, good game"),
            window_icon: String::from(""),
            window_height: 600,
            window_width: 800,
            vsync: true,
        }

    }
}

impl Conf {
    /// Same as Conf::default()
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a TOML file from the given `Read` and attempts to parse
    /// a `Conf` from it.
    ///
    /// It only looks for things under the `[ggez]` section heading,
    /// so you can put your own sections in the file and use them for
    /// your own purposes and they will not interfere here.
    pub fn from_toml_file<R: io::Read>(file: &mut R) -> GameResult<Conf> {
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let decoded = toml::from_str(&s)?;
        Ok(decoded)
    }

    /// Saves the `Conf` to the given `Write` object,
    /// formatted as TOML.
    pub fn to_toml_file<W: io::Write>(&self, file: &mut W) -> GameResult<()> {
        // This gets a little elaborate because we have to
        // add another level to the TOML object to create
        // the [ggez] section.
        //
        // So we encode the Conf into a toml::Value...
        let s = toml::to_vec(self)?;
        file.write(&s)?;
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
        c1.to_toml_file(&mut writer).unwrap();
        let mut reader = writer.as_slice();
        let c2 = conf::Conf::from_toml_file(&mut reader).unwrap();
        assert_eq!(c1, c2);
    }
}
