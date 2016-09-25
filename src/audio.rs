//! Provides an interface to output sound to the user's speakers.
//!
//! This departs from the Love2D API a bit because SDL2_mixer is opinionated
//! about the difference between samples and music files, and also makes channel
//! management and such explicit.
//! This seems a bit awkward but we'll roll with it for now.

use std::path;

use sdl2;
use sdl2_mixer;
use sdl2_mixer::LoaderRWops;

use context::Context;
use util::rwops_from_path;
use GameError;
use GameResult;

/// An object representing a channel that may be playing a particular Sound.
pub type Channel = sdl2_mixer::Channel;


/// A trait for general operations on sound objects.
pub trait AudioOps {
    fn new_channel() -> Channel;

    fn play_sound(&self, sound: &Sound) -> GameResult<Channel>;

    fn pause(&self);
      
    fn stop(&self);
    
    fn resume(&self);
    
    fn rewind(&self);
}

/// A source of audio data.
pub struct Sound {
    chunk: sdl2_mixer::Chunk,
}

impl Sound {
    /// Load a new Sound
    pub fn new(context: &Context, path: &path::Path) -> GameResult<Sound> {
        let mixer = &context.mixer_context;

        let mut buffer: Vec<u8> = Vec::new();
        let rwops = try!(rwops_from_path(context, path, &mut buffer));
        // SDL2_image SNEAKILY adds this method to RWops.
        let chunk = try!(rwops.load_wav());

        Ok(Sound {
            chunk: chunk,
        })
    }

    /// Play a sound.
    ///
    /// Returns a `Channel`, which can be used to manipulate the 
    /// playback, eg pause, stop, restart, etc.
    pub fn play(&self) -> GameResult<Channel> {
        let channel = sdl2_mixer::channel(-1);
        // This try! is a little redundant but make the
        // GameResult type conversion work right.
        channel.play(&self.chunk, 0)
            .map_err(|e| GameError::from(e))
    }
}



impl AudioOps for Channel {
    /// Return a new channel that is not playing anything.
    fn new_channel() -> Channel {
        sdl2_mixer::channel(-1)
    }
    /// Plays the given Sound on the Channel
    fn play_sound(&self, sound: &Sound) -> GameResult<Channel> {
        let channel = self;
        channel.play(&sound.chunk, 0)
            .map_err(|e| GameError::from(e))
    }

    fn pause(&self) {
        self.pause()
    }
      
    fn stop(&self) {
        self.halt()
    }
    
    fn resume(&self) {
        self.resume()
    }
    
    /// Restarts playing a sound if this channel is currently
    /// playing it.
    fn rewind(&self) {
        if let Some(chunk) = self.get_chunk() {
            self.stop();
            self.play(&chunk, 0);
        }
    }
}


/// A source of music data.
pub struct Music {
    music: sdl2_mixer::Music,
}

use util::load_music;

impl Music {
    /// Load the given Music.
    pub fn new(context: &Context, path: &path::Path) -> GameResult<Music> {
        let mixer = &context.mixer_context;

        let mut buffer: Vec<u8> = Vec::new();
        let rwops = try!(rwops_from_path(context, path, &mut buffer));
        // SDL2_image SNEAKILY adds this method to RWops.
        let music = try!(load_music(rwops));

        Ok(Music {
            music: music,
        })
    }
}

pub fn play_music(music: &Music) -> GameResult<()> {
    music.music.play(-1)
        .map_err(|e| GameError::from(e))
}

pub fn pause_music() {
    sdl2_mixer::Music::pause()
}

pub fn resume_music() {
    sdl2_mixer::Music::resume()

}

pub fn stop_music() {
    sdl2_mixer::Music::halt()
}

pub fn rewind_music() {
    sdl2_mixer::Music::rewind()
}