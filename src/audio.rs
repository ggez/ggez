//! Provides an interface to output sound to the user's speakers.
//!
//! It consists of two main types: `SoundData` is just raw sound data,
//! and a `Source` is a `SoundData` connected to a particular sound
//! channel.

use std::fmt;
use std::io;
use std::io::Read;
use std::path;

use std::sync::Arc;

use rodio;

use context::Context;
use GameError;
use GameResult;


/// A struct that contains all information for tracking sound info.
///
/// You generally don't have to create this yourself, it will be part
/// of your `Context` object.
pub struct AudioContext {
    endpoint: rodio::Endpoint,
}

impl AudioContext {
    /// Create new AudioContext.
    pub fn new() -> GameResult<AudioContext> {
        let error = GameError::AudioError(String::from("Could not initialize sound system (for \
             some reason)"));
        let e = rodio::get_default_endpoint().ok_or(error)?;
        Ok(AudioContext { endpoint: e })
    }
}

impl fmt::Debug for AudioContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<AudioContext: {:p}>", self)
    }
}


/// Static sound data stored in memory.
/// It is Arc'ed, so cheap to clone.
#[derive(Clone, Debug)]
pub struct SoundData(Arc<Vec<u8>>);

impl SoundData {
    /// Copies the data in the given slice into a new SoundData object.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut buffer = Vec::with_capacity(data.len());
        buffer.extend(data);
        SoundData::from(buffer)

    }

    /// Creates a SoundData from any Read object; this involves
    /// copying it into a buffer.
    pub fn from_read<R>(reader: &mut R) -> GameResult<Self>
        where R: Read
    {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        Ok(SoundData::from(buffer))

    }
}

impl From<Vec<u8>> for SoundData {
    fn from(v: Vec<u8>) -> Self {
        SoundData(Arc::new(v))
    }
}

impl AsRef<[u8]> for SoundData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

/// A source of audio data connected to a particular `Channel`.
/// Will stop playing when dropped.
// TODO: Check and see if this matches Love2d's semantics!
// Eventually it might read from a streaming decoder of some kind,
// but for now it is just an in-memory SoundData structure.
// The source of a rodio decoder must be Send, which something
// that contains a reference to a ZipFile is not, so we are going
// to just slurp all the data into memory for now.
// There's really a lot of work that needs to be done here, since
// rodio has gotten better (if still somewhat arcane) and our filesystem
// code has done the data-slurping-from-zip's for us
// but
// for now it works.
pub struct Source {
    data: io::Cursor<SoundData>,
    sink: rodio::Sink,
}

impl Source {
    /// Create a new Source from the given file.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let data = {
            let file = &mut context.filesystem.open(path)?;
            SoundData::from_read(file)?
        };
        Source::from_data(context, data)
    }

    /// Creates a new Source using the given SoundData object.
    pub fn from_data(context: &mut Context, data: SoundData) -> GameResult<Self> {
        let sink = rodio::Sink::new(&context.audio_context.endpoint);
        let cursor = io::Cursor::new(data);
        Ok(Source {
            data: cursor,
            sink: sink,
        })
    }

    /// Plays the Source.
    pub fn play(&self) -> GameResult<()> {
        // Creating a new Decoder each time seems a little messy,
        // since it may do checking and data-type detection that is
        // redundant, but it's not super expensive.
        // See https://github.com/ggez/ggez/issues/98 for discussion
        let cursor = self.data.clone();
        let decoder = rodio::Decoder::new(cursor)?;
        self.sink.append(decoder);
        Ok(())
    }

    /// Pauses playback
    pub fn pause(&self) {
        self.sink.pause()
    }

    /// Resumes playback
    pub fn resume(&self) {
        self.sink.play()
    }

    /// Stops playback
    pub fn stop(&self) {
        self.sink.stop()
    }

    /// Gets the current volume
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    /// Sets the current volume
    pub fn set_volume(&mut self, value: f32) {
        self.sink.set_volume(value)
    }

    /// Get whether or not the source is paused
    pub fn paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Get whether or not the source is playing
    pub fn playing(&self) -> bool {
        !self.paused() // && !self.stopped()
    }
}


impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Audio source: {:p}>", self)
    }
}


// TODO: global start, stop, volume?
