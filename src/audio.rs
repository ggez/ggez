//! Provides an interface to output sound to the user's speakers.
//!
//! This departs from the Love2D API a bit because SDL2_mixer is opinionated
//! about the difference between samples and music files, and also makes channel
//! management and such explicit.
//! This seems a bit awkward but we'll roll with it for now.

use std::path;

use sdl2_mixer;
use sdl2_mixer::LoaderRWops;

use context::Context;
use util::rwops_from_path;
use GameError;

/// An object representing a channel that may be playing a particular Sound.
pub type Channel = sdl2_mixer::Channel;


/// A trait for general operations on sound objects.
pub trait AudioOps {
    fn new_channel() -> Channel;

    fn play_sound(&self, sound: &Sound) -> Channel;

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
    pub fn new(context: &Context, path: &path::Path) -> Sound {
        let mut buffer: Vec<u8> = Vec::new();
        let rwops = rwops_from_path(context, path, &mut buffer);
        // SDL2_image SNEAKILY adds this method to RWops.
        let chunk = rwops.load_wav().unwrap();

        Sound {
            chunk: chunk,
        }
    }

    /// Play a sound.
    ///
    /// Returns a `Channel`, which can be used to manipulate the 
    /// playback, eg pause, stop, restart, etc.
    pub fn play(&self) -> Channel {
        let channel = sdl2_mixer::channel(-1);
        channel.play(&self.chunk, 0).unwrap()
    }
}



impl AudioOps for Channel {
    /// Return a new channel that is not playing anything.
    fn new_channel() -> Channel {
        sdl2_mixer::channel(-1)
    }
    /// Plays the given Sound on the Channel
    fn play_sound(&self, sound: &Sound) -> Channel {
        let channel = self;
        channel.play(&sound.chunk, 0).unwrap()
    }

    fn pause(&self) {
        Channel::pause(*self)
    }
      
    fn stop(&self) {
        self.halt()
    }
    
    fn resume(&self) {
        Channel::resume(*self)
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
    pub fn new(context: &Context, path: &path::Path) -> Music {
        let mut buffer: Vec<u8> = Vec::new();
        let rwops = rwops_from_path(context, path, &mut buffer);
        // SDL2_image SNEAKILY adds this method to RWops.
        let music = load_music(rwops).unwrap();

        Music {
            music: music,
        }
    }
}

pub fn play_music(music: &Music) {
    music.music.play(-1).unwrap()
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