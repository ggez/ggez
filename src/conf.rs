//! The `conf` module contains functions for loading and saving game
//! configurations.
//! A lot of this is lifted whole-hog from LÖVE because it's stuff
//! we need anyway.

use std::io;
use toml;
use rustc_serialize::{Decodable, Encodable};
use std::collections::BTreeMap;

use {GameError, GameResult};

/// A structure containing configuration data
/// for the game engine.
#[derive(RustcDecodable, RustcEncodable, Debug, PartialEq)]
pub struct Conf {
    /// Version of ggez your game is designed to work with.
    pub version: String,

    /// The window title.
    pub window_title: String,
    /// A file path to the window's icon.
    pub window_icon: String,
    /// The window's default height
    pub window_height: u32,
    /// The window's default width
    pub window_width: u32, /* To implement still.
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

impl Conf {
    /// Create a new Conf with some vague defaults.
    pub fn new() -> Conf {
        Conf {
            version: String::from("0.0.0"),
            window_title: String::from("An easy, good game"),
            window_icon: String::from(""),
            window_height: 600,
            window_width: 800,
        }
    }

    /// Load a TOML file from the given `Read` and attempts to parse
    /// a `Conf` from it.
    ///
    /// It only looks for things under the `[ggez]` section heading,
    /// so you can put your own sections in the file and use them for
    /// your own purposes and they will get ignored here.
    pub fn from_toml_file<R: io::Read>(file: &mut R) -> GameResult<Conf> {
        let mut s = String::new();
        try!(file.read_to_string(&mut s));
        let mut parser = toml::Parser::new(&s);
        let toml = try!(parser.parse()
                              .ok_or(String::from("Could not parse config file?")));
        let config = try!(toml.get("ggez")
                              .ok_or(String::from("Section [ggez] not in config file")));
        let mut decoder = toml::Decoder::new(config.clone());
        Conf::decode(&mut decoder).map_err(GameError::from)
    }

    /// Saves the `Conf` to the given `Write` object,
    /// formatted as TOML.
    pub fn to_toml_file<W: io::Write>(&self, file: &mut W) -> GameResult<()> {
        // This gets a little elaborate because we have to
        // add another level to the TOML object to create
        // the [ggez] section.
        //
        // So we encode the Conf into a toml::Value...
        let mut e = toml::Encoder::new();
        self.encode(&mut e).unwrap();

        // Create another node that is a Table containing it...
        let mut t = BTreeMap::new();
        t.insert("ggez".to_owned(), toml::Value::Table(e.toml));
        let toml_t = toml::Value::Table(t);

        // Then serialize that to a string.
        let toml_str = toml::encode_str(&toml_t);
        let toml_bytes = toml_str.as_bytes();
        file.write_all(toml_bytes).map_err(GameError::from)
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
