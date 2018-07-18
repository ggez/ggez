//! Provides an interface to output sound to the user's speakers.
//!
//! It consists of two main types: `SoundData` is just raw sound data,
//! and a `Source` is a `SoundData` connected to a particular sound
//! channel.

use std::fmt;
use std::io;
use std::io::Read;
use std::mem;
use std::path;
use std::time;

use std::sync::Arc;

use mint;
use rodio;

use context::Context;
use filesystem;
use GameError;
use GameResult;

/// A struct that contains all information for tracking sound info.
///
/// You generally don't have to create this yourself, it will be part
/// of your `Context` object.
pub struct AudioContext {
    device: rodio::Device,
}

impl AudioContext {
    /// Create new AudioContext.
    pub fn new() -> GameResult<AudioContext> {
        let device = rodio::default_output_device().ok_or_else(|| {
            GameError::AudioError(String::from(
                "Could not initialize sound system (for some reason)",
            ))
        })?;
        Ok(AudioContext { device: device })
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
pub struct SoundData(Arc<[u8]>);

impl SoundData {
    /// Create a new SoundData from the file at the given path.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let file = &mut filesystem::open(context, path)?;
        SoundData::from_read(file)
    }

    /// Copies the data in the given slice into a new SoundData object.
    pub fn from_bytes(data: &[u8]) -> Self {
        SoundData(Arc::from(data))
    }

    /// Creates a SoundData from any Read object; this involves
    /// copying it into a buffer.
    pub fn from_read<R>(reader: &mut R) -> GameResult<Self>
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer)?;

        Ok(SoundData::from(buffer))
    }
}

impl From<Arc<[u8]>> for SoundData {
    #[inline]
    fn from(arc: Arc<[u8]>) -> Self {
        SoundData(arc)
    }
}

impl From<Vec<u8>> for SoundData {
    fn from(v: Vec<u8>) -> Self {
        SoundData(Arc::from(v))
    }
}

impl From<Box<[u8]>> for SoundData {
    fn from(b: Box<[u8]>) -> Self {
        SoundData(Arc::from(b))
    }
}

impl AsRef<[u8]> for SoundData {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
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
// but for now it works.
pub struct Source {
    data: io::Cursor<SoundData>,
    sink: rodio::Sink,
    repeat: bool,
    fade_in: time::Duration,
    pitch: f32,
}

impl Source {
    /// Create a new Source from the given file.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let data = SoundData::new(context, path)?;
        Source::from_data(context, data)
    }

    /// Creates a new Source using the given SoundData object.
    pub fn from_data(context: &mut Context, data: SoundData) -> GameResult<Self> {
        let sink = rodio::Sink::new(&context.audio_context.device);
        let cursor = io::Cursor::new(data);
        Ok(Source {
            sink,
            data: cursor,
            repeat: false,
            fade_in: time::Duration::from_millis(0),
            pitch: 1.0,
        })
    }

    /// Plays the Source; defaults to `play_restart`
    #[inline(always)]
    pub fn play(&mut self) -> GameResult {
        self.play_restart()
    }

    /// Plays the Source; restarts the sound if currently playing
    pub fn play_restart(&mut self) -> GameResult {
        self.stop();
        self.play_later()
    }

    /// Plays the Source; waits until done if the sound is currently playing
    pub fn play_later(&self) -> GameResult {
        // Creating a new Decoder each time seems a little messy,
        // since it may do checking and data-type detection that is
        // redundant, but it's not super expensive.
        // See https://github.com/ggez/ggez/issues/98 for discussion
        use rodio::Source;
        let cursor = self.data.clone();
        let sound = rodio::Decoder::new(cursor)?;

        // todo: this is getting ugly but a match wouldn't be much better. any ideas, anyone?
        if (self.pitch - 1.0).abs() < 1e-6 {
            if self.repeat {
                let sound = sound.repeat_infinite();
                if self.fade_in == time::Duration::from_secs(0) {
                    self.sink.append(sound);
                } else {
                    let sound = sound.fade_in(self.fade_in);
                    self.sink.append(sound);
                }
            } else {
                if self.fade_in == time::Duration::from_secs(0) {
                    self.sink.append(sound);
                } else {
                    let sound = sound.fade_in(self.fade_in);
                    self.sink.append(sound);
                }
            }
        } else {
            let sound = sound.speed(self.pitch);
            if self.repeat {
                let sound = sound.repeat_infinite();
                if self.fade_in == time::Duration::from_secs(0) {
                    self.sink.append(sound);
                } else {
                    let sound = sound.fade_in(self.fade_in);
                    self.sink.append(sound);
                }
            } else {
                if self.fade_in == time::Duration::from_secs(0) {
                    self.sink.append(sound);
                } else {
                    let sound = sound.fade_in(self.fade_in);
                    self.sink.append(sound);
                }
            }
        }

        Ok(())
    }

    /// Play source "in the background"; cannot be stopped
    pub fn play_detached(&mut self) -> GameResult {
        self.stop();
        self.play_later()?;

        let device = rodio::default_output_device().unwrap();
        let new_sink = rodio::Sink::new(&device);
        let old_sink = mem::replace(&mut self.sink, new_sink);
        old_sink.detach();

        Ok(())
    }

    /// Sets the source to repeat playback infinitely on next `play()`
    pub fn set_repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    /// Sets the fade-in time of the source
    pub fn set_fade_in(&mut self, dur: time::Duration) {
        self.fade_in = dur;
    }

    /// Sets the pitch ratio (by adjusting the playback speed)
    pub fn set_pitch(&mut self, ratio: f32) {
        self.pitch = ratio;
    }

    /// Gets whether or not the source is set to repeat.
    pub fn repeat(&self) -> bool {
        self.repeat
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
    pub fn stop(&mut self) {
        // Sinks cannot be reused after calling `.stop()`. See
        // https://github.com/tomaka/rodio/issues/171 for information.
        // To stop the current sound we have to drop the old sink and
        // create a new one in its place.
        // This is most ugly because in order to create a new sink
        // we need a `device`. However, we can only get the default
        // device without having access to a context. Currently that's
        // fine because the `AudioContext` uses the default device too,
        // but it may cause problems in the future if devices become
        // customizable.
        let device = rodio::default_output_device().unwrap();
        self.sink = rodio::Sink::new(&device);
    }

    /// Returns whether or not the source is stopped
    /// -- that is, has no more data to play.
    pub fn stopped(&self) -> bool {
        self.sink.empty()
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

    /// Get whether or not the source is playing (ie, not paused
    /// and not stopped)
    pub fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }
}

impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Audio source: {:p}>", self)
    }
}

/// A source of audio data located in space relative to a listener's ears.
/// Will stop playing when dropped.
pub struct SpatialSource {
    data: io::Cursor<SoundData>,
    sink: rodio::SpatialSink,
    repeat: bool,
    fade_in: time::Duration,
    pitch: f32,

    left_ear: mint::Point3<f32>,
    right_ear: mint::Point3<f32>,
    emitter_position: mint::Point3<f32>,
}

impl SpatialSource {
    /// Create a new Source from the given file.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let data = SoundData::new(context, path)?;
        SpatialSource::from_data(context, data)
    }

    /// Creates a new Source using the given SoundData object.
    pub fn from_data(context: &mut Context, data: SoundData) -> GameResult<Self> {
        let sink = rodio::SpatialSink::new(
            &context.audio_context.device,
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        );

        let cursor = io::Cursor::new(data);

        Ok(SpatialSource {
            sink,
            data: cursor,
            repeat: false,
            fade_in: time::Duration::from_millis(0),
            pitch: 1.0,
            left_ear: [-1.0, 0.0, 0.0].into(),
            right_ear: [1.0, 0.0, 0.0].into(),
            emitter_position: [0.0, 0.0, 0.0].into(),
        })
    }

    /// Plays the Source; defaults to `play_restart`
    #[inline(always)]
    pub fn play(&mut self) -> GameResult {
        self.play_restart()
    }

    /// Plays the Source; restarts the sound if currently playing
    pub fn play_restart(&mut self) -> GameResult {
        self.stop();
        self.play_later()
    }

    /// Plays the Source; waits until done if the sound is currently playing
    pub fn play_later(&self) -> GameResult {
        use rodio::Source;
        let cursor = self.data.clone();
        let decoder = rodio::Decoder::new(cursor)?;

        // todo: this is getting ugly but nested ifs wouldn't be much better. any ideas, anyone?
        match (self.repeat, self.fade_in, self.pitch) {
            (true, dur, ratio)
                if dur == time::Duration::from_secs(0) && (ratio - 1.0).abs() < 1e-6 =>
            {
                self.sink.append(decoder.repeat_infinite())
            }
            (false, dur, ratio)
                if dur == time::Duration::from_secs(0) && (ratio - 1.0).abs() < 1e-6 =>
            {
                self.sink.append(decoder)
            }
            (true, dur, ratio) if (ratio - 1.0).abs() < 1e-6 => {
                self.sink.append(decoder.repeat_infinite().fade_in(dur))
            }
            (false, dur, ratio) if (ratio - 1.0).abs() < 1e-6 => {
                self.sink.append(decoder.fade_in(dur))
            }
            (true, dur, ratio) if dur == time::Duration::from_secs(0) => {
                self.sink.append(decoder.speed(ratio).repeat_infinite())
            }
            (false, dur, ratio) if dur == time::Duration::from_secs(0) => {
                self.sink.append(decoder.speed(ratio))
            }
            (true, dur, ratio) => self.sink
                .append(decoder.speed(ratio).repeat_infinite().fade_in(dur)),
            (false, dur, ratio) => self.sink.append(decoder.speed(ratio).fade_in(dur)),
        }

        Ok(())
    }

    /// Play source "in the background"; cannot be stopped
    pub fn play_detached(&mut self) -> GameResult {
        self.stop();
        self.play_later()?;

        let device = rodio::default_output_device().unwrap();
        let new_sink = rodio::SpatialSink::new(
            &device,
            self.emitter_position.into(),
            self.left_ear.into(),
            self.right_ear.into(),
        );
        let old_sink = mem::replace(&mut self.sink, new_sink);
        old_sink.detach();

        Ok(())
    }

    /// Sets the source to repeat playback infinitely on next `play()`
    pub fn set_repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    /// Sets the fade-in time of the source
    pub fn set_fade_in(&mut self, dur: time::Duration) {
        self.fade_in = dur;
    }

    /// Sets the pitch ratio (by adjusting the playback speed)
    pub fn set_pitch(&mut self, ratio: f32) {
        self.pitch = ratio;
    }

    /// Gets whether or not the source is set to repeat.
    pub fn repeat(&self) -> bool {
        self.repeat
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
    pub fn stop(&mut self) {
        // `rodio::SpatialSink` does not have a `.stop()` method at
        // the moment. To stop the current sound we drop the old
        // sink and create a new one in its place.
        // This is most ugly because in order to create a new sink
        // we need a `device`. However, we can only get the default
        // device without having access to a context. Currently that's
        // fine because the `AudioContext` uses the default device too,
        // but it may cause problems in the future if devices become
        // customizable.
        let device = rodio::default_output_device().unwrap();
        self.sink = rodio::SpatialSink::new(
            &device,
            self.emitter_position.into(),
            self.left_ear.into(),
            self.right_ear.into(),
        );
    }

    /// Returns whether or not the source is stopped
    /// -- that is, has no more data to play.
    pub fn stopped(&self) -> bool {
        self.sink.empty()
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

    /// Get whether or not the source is playing (ie, not paused
    /// and not stopped)
    pub fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }

    /// Set location of the sound
    pub fn set_position<P>(&mut self, pos: P)
    where
        P: Into<mint::Point3<f32>>,
    {
        self.emitter_position = pos.into();
        self.sink.set_emitter_position(self.emitter_position.into());
    }

    /// Set locations of the listener's ears
    pub fn set_ears<P>(&mut self, left: P, right: P)
    where
        P: Into<mint::Point3<f32>>,
    {
        self.left_ear = left.into();
        self.right_ear = right.into();
        self.sink.set_left_ear_position(self.left_ear.into());
        self.sink.set_right_ear_position(self.right_ear.into());
    }
}

impl fmt::Debug for SpatialSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Spatial audio source: {:p}>", self)
    }
}
