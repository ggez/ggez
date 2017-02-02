//! Provides an interface to output sound to the user's speakers.
//!
//! This departs from the LÖVE API a bit because `SDL2_mixer` is opinionated
//! about the difference between samples and music files, and also makes channel
//! management and such more explicit.
//! This seems a bit awkward but we'll roll with it for now.

use std::fmt;
use std::io;
use std::io::Read;
use std::path;

use rodio;

use context::Context;
use filesystem;
use util;
use GameError;
use GameResult;

/// An object representing a channel that may be playing a particular Sound.
pub type Channel = rodio::Sink;

/// A trait for general operations on sound objects.
pub trait AudioOps {
    fn new_channel<'a>(ctx: &Context) -> Channel;

    fn play_sound(&self, sound: &Sound) -> GameResult<Channel>;

    fn pause(&self);

    fn stop(&self);

    fn resume(&self);

    fn rewind(&self);
}

/// A struct that contains all information for tracking sound info.
pub struct AudioContext {
    endpoint: rodio::Endpoint,
}

impl AudioContext {
    pub fn new() -> GameResult<AudioContext> {
        let error = GameError::AudioError(String::from("Could not initialize sound system (for \
                                                        some reason)"));
        let e = rodio::get_default_endpoint().ok_or(error)?;
        Ok(AudioContext { endpoint: e })
    }
}

/// A source of audio data.
pub struct Sound {
    chunk: Vec<u8>,
}

impl Sound {
    /// Load a new Sound
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Sound> {
        let path = path.as_ref();
        let file = &mut context.filesystem.open(path)?;
        // The source of a rodio decoder must be Send, which something
        // that contains a reference to a ZipFile is not, so we are going
        // to just slurp all the data into memory for now.
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Ok(Sound { chunk: buffer })
    }

    /// Play a sound on the first available `Channel`.
    ///
    /// Returns a `Channel`, which can be used to manipulate the
    /// playback, eg pause, stop, restart, etc.
    pub fn play(&self, ctx: &Context) {
        let sink = rodio::Sink::new(&ctx.audio_context.endpoint);
        // This clone is wiiiiiiggy.
        // Not sure how I'm SUPPOSED to be
        // handling this, since a Decoder
        // and Sink take ownership of what is
        // passed to them!
        let cursor = io::Cursor::new(self.chunk.clone());
        let source = rodio::Decoder::new(cursor).unwrap();
        sink.append(source);
        sink.detach();

    }
}


impl fmt::Debug for Sound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Sound: {:p}>", self)
    }
}



impl AudioOps for Channel {
    /// Return a new channel that is not playing anything.
    fn new_channel<'a>(ctx: &Context) -> Channel {
        // sdl2::mixer::channel(-1);
        rodio::Sink::new(&ctx.audio_context.endpoint)
    }

    /// Plays the given Sound on this `Channel`
    fn play_sound(&self, sound: &Sound) -> GameResult<Channel> {
        // let channel = self;
        // channel.play(&sound.chunk, 0)
        // map_err(GameError::from)
        unimplemented!()
    }

    /// Pauses playback of the `Channel`
    fn pause(&self) {
        // Channel::pause(*self)
        unimplemented!()
    }

    /// Stops whatever the `Channel` is playing.
    fn stop(&self) {
        // self.halt()
        unimplemented!()
    }

    /// Resumes playback where it left off (if any).
    fn resume(&self) {
        // Channel::resume(*self)
        unimplemented!()
    }

    /// Restarts playing a sound if this channel is currently
    /// playing it.
    fn rewind(&self) {

        // if let Some(chunk) = self.get_chunk() {
        //    self.stop();
        //    let _ = self.play(&chunk, 0);
        //
        unimplemented!()
    }
}

// A source of music data.
// Music is played on a separate dedicated channel from sounds,
// and also has a separate corpus of decoders than sounds do;
// see the `SDL2_mixer` documentation for details or use
// `Context::print_sound_stats()` to print out which decoders
// are supported for your build.
// pub struct Music {
// music: sdl2::mixer::Chunk,
// }
//
//
// impl Music {
// Load the given Music.
// pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Music> {
// let path = path.as_ref();
// let mut buffer: Vec<u8> = Vec::new();
// let rwops = util::rwops_from_path(context, path, &mut buffer)?;
// SDL2_mixer SNEAKILY adds this method to RWops.
// let music = rwops.load_wav()?;
//
// Ok(Music { music: music })
// }
// }
//
//
// impl fmt::Debug for Music {
// fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// write!(f, "<Music: {:p}>", self)
// }
// }
//
// Play the given music n times.  -1 loops forever.
// pub fn play_music_times(ctx: &Context, music: &Music, n: i32) -> GameResult<()> {
// HACK HACK HACK
// let _ = ctx.music_channel.play(&music.music, n);
// Ok(())
// }
// Start playing the given music (looping forever)
// pub fn play_music(ctx: &Context, music: &Music) -> GameResult<()> {
// play_music_times(ctx, music, -1)
// }
//
// Pause currently playing music
// pub fn pause_music(ctx: &Context) {
// ctx.music_channel.pause();
// }
//
// Resume currently playing music, if any
// pub fn resume_music(ctx: &Context) {
// ctx.music_channel.resume();
//
// }
//
// Stop currently playing music
// pub fn stop_music(ctx: &Context) {
// ctx.music_channel.stop();
// }
//
// Rewind the currently playing music to the beginning.
// pub fn rewind_music(ctx: &Context) {
// ctx.music_channel.rewind();
// }
//
