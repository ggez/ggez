//! Provides an interface to output sound to the user's speakers.
//!
//! This departs from the LÖVE API a bit because `SDL2_mixer` is opinionated
//! about the difference between samples and music files, and also makes channel
//! management and such more explicit.
//! This seems a bit awkward but we'll roll with it for now.

use std::fmt;
use std::path;

use sdl2_mixer;
use sdl2_mixer::LoaderRWops;

use context::Context;
use util;
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
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Sound> {
        let path = path.as_ref();
        let mut buffer: Vec<u8> = Vec::new();
        let rwops = try!(util::rwops_from_path(context, path, &mut buffer));
        // SDL2_image SNEAKILY adds this method to RWops.
        let chunk = try!(rwops.load_wav());

        Ok(Sound { chunk: chunk })
    }

    /// Play a sound on the first available `Channel`.
    ///
    /// Returns a `Channel`, which can be used to manipulate the
    /// playback, eg pause, stop, restart, etc.
    pub fn play(&self) -> GameResult<Channel> {
        let channel = sdl2_mixer::channel(-1);
        // This try! is a little redundant but make the
        // GameResult type conversion work right.
        channel.play(&self.chunk, 0)
               .map_err(GameError::from)
    }
}


impl fmt::Debug for Sound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Sound: {:p}>", self)
    }
}



impl AudioOps for Channel {
    /// Return a new channel that is not playing anything.
    fn new_channel() -> Channel {
        sdl2_mixer::channel(-1)
    }

    /// Plays the given Sound on this `Channel`
    fn play_sound(&self, sound: &Sound) -> GameResult<Channel> {
        let channel = self;
        channel.play(&sound.chunk, 0)
               .map_err(GameError::from)
    }

    /// Pauses playback of the `Channel`
    fn pause(&self) {
        Channel::pause(*self)
    }

    /// Stops whatever the `Channel` is playing.
    fn stop(&self) {
        self.halt()
    }

    /// Resumes playback where it left off (if any).
    fn resume(&self) {
        Channel::resume(*self)
    }

    /// Restarts playing a sound if this channel is currently
    /// playing it.
    fn rewind(&self) {
        if let Some(chunk) = self.get_chunk() {
            self.stop();
            let _ = self.play(&chunk, 0);
        }
    }
}


/// A source of music data.
/// Music is played on a separate dedicated channel from sounds,
/// and also has a separate corpus of decoders than sounds do;
/// see the `SDL2_mixer` documentation for details or use
/// `Context::print_sound_stats()` to print out which decoders
/// are supported for your build.
pub struct Music {
    music: sdl2_mixer::Music,
}


impl Music {
    /// Load the given Music.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Music> {
        let path = path.as_ref();
        let mut buffer: Vec<u8> = Vec::new();
        let rwops = try!(util::rwops_from_path(context, path, &mut buffer));
        // SDL2_image SNEAKILY adds this method to RWops.
        let music = try!(rwops.load_music());

        Ok(Music { music: music })
    }
}


impl fmt::Debug for Music {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Music: {:p}>", self)
    }
}

/// Play the given music n times.  -1 loops forever.
pub fn play_music_times(music: &Music, n: isize) -> GameResult<()> {
    music.music
         .play(n)
         .map_err(GameError::from)
}

/// Start playing the given music (looping forever)
pub fn play_music(music: &Music) -> GameResult<()> {
    play_music_times(music, -1)
}

/// Pause currently playing music
pub fn pause_music() {
    sdl2_mixer::Music::pause()
}

/// Resume currently playing music, if any
pub fn resume_music() {
    sdl2_mixer::Music::resume()

}

/// Stop currently playing music
pub fn stop_music() {
    sdl2_mixer::Music::halt()
}

/// Rewind the currently playing music to the beginning.
pub fn rewind_music() {
    sdl2_mixer::Music::rewind()
}
