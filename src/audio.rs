//! Provides an interface to output sound to the user's speakers.
//!
//! This departs from the Love2D API a bit because SDL2_mixer is opinionated
//! about the difference between samples and music files.
//! This seems a bit dumb but we'll roll with it for now.

use sdl2;
use sdl2_mixer;
use util::rwops_from_path;

/// A trait for general operations on sound objects.
pub trait Audio {
    fn play(&self) {
    }

    fn pause(&self) {
    }
      
    fn stop(&self) {
    }
    
    fn resume(&self) {
    }
    
    fn rewind(&self) {
    }
}


/// A source of audio data.
pub struct Sound {
}


/// A source of music data.
pub struct Music {
}

impl Audio for Sound {
}

impl Audio for Music {
}
