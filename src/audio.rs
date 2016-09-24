//! Provides an interface to output sound to the user's speakers.
//!
//! This departs from the Love2D API a bit because SDL2_mixer is opinionated
//! about the difference between samples and music files.
//! This seems a bit dumb but we'll roll with it for now.

use std::path;

use sdl2;
use sdl2_mixer;
use sdl2_mixer::LoaderRWops;

use context::Context;
use util::rwops_from_path;

/// A trait for general operations on sound objects.
pub trait Audio {
    fn play(&self);

    fn pause(&self);
      
    fn stop(&self);
    
    fn resume(&self);
    
    fn rewind(&self);
}


/// A source of audio data.
pub struct Sound {
    chunk: sdl2_mixer::Chunk,
    channel: Option<sdl2_mixer::Channel>,
}

impl Sound {
    pub fn new(context: &Context, path: &path::Path) -> Sound {
        let mixer = &context.mixer_context;

        let mut buffer: Vec<u8> = Vec::new();
        let rwops = rwops_from_path(context, path, &mut buffer);
        // SDL2_image SNEAKILY adds this method to RWops.
        let chunk = rwops.load_wav().unwrap();

        Sound {
            chunk: chunk,
            channel: None,
        }
    }
}


/// A source of music data.
pub struct Music {
    music: sdl2_mixer::Music,
}

use util::load_music;

impl Music {
    pub fn new(context: &Context, path: &path::Path) -> Music {
        let mixer = &context.mixer_context;

        let mut buffer: Vec<u8> = Vec::new();
        let rwops = rwops_from_path(context, path, &mut buffer);
        // SDL2_image SNEAKILY adds this method to RWops.
        let music = load_music(rwops).unwrap();

        Music {
            music: music,
        }
    }

}

impl Audio for Sound {
    fn play(&self) {
        let channel = sdl2_mixer::channel(-1);
        let _c = channel.play(&self.chunk, 0);
        //println!("Playing channel {:?}", c);
        //self.channel = Some(c);
    }

    fn pause(&self) {
        if let Some(channel) = self.channel {
            channel.pause()
        }
    }
      
    fn stop(&self) {
        if let Some(channel) = self.channel {
            channel.halt()
        }
    }
    
    fn resume(&self) {
        if let Some(channel) = self.channel {
            channel.resume()
        }
    }
    
    fn rewind(&self) {
        // BUGGO: Not sure if this is right...
        self.stop();
        self.play();
    }
}

impl Audio for Music {
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
