
use std::io;
use toml;
use rustc_serialize::Decodable;

use GameError;

/// The `conf` module contains functions for loading and saving game
/// configurations.
/// A lot of this is lifted whole-hog from LÖVE because it's stuff
/// we need anyway.
#[derive(RustcDecodable, Debug)]
pub struct Conf {
    /// The name of the save directory
    // id: String,
    /// Version of ggez your game is designed to work with.
    pub version: String,

    /// The window title.
    pub window_title: String,
    /// A file path to the window's icon.
    pub window_icon: String,
    /// The window's default height
    pub window_height: u32,
    /// The window's default width
    pub window_width: u32, 

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

impl Conf {
    pub fn new() -> Conf {
        Conf {
            version: String::from("0.0.0"),
            window_title: String::from("An easy, good game"),
            window_icon: String::from(""),
            window_height: 600,
            window_width: 800,
        }
    }

    pub fn from_toml_file<R: io::Read>(file: &mut R) -> Result<Conf, GameError> {
        // TODO: Fix these unwraps when it's not midnight.
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        let mut parser = toml::Parser::new(&s);
        let toml = parser.parse().unwrap();
        let config = toml.get("conf").unwrap();
        let mut decoder = toml::Decoder::new(config.clone());
        Conf::decode(&mut decoder).map_err(|e| GameError::from(e))
    }
}
