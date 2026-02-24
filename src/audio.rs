//! Provides an interface to output sound to the user's speakers.
//!
//! It consists of two main types: [`SoundData`](struct.SoundData.html)
//! is just an array of raw sound data bytes, and a [`Source`](struct.Source.html) is a
//! `SoundData` connected to a particular sound channel ready to be played.
#![cfg(feature = "audio")]

use std::fmt;
use std::io;
use std::path;
use std::time;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::context::Has;
use crate::error::GameError;
use crate::error::GameResult;
use crate::filesystem::Filesystem;

/// A struct that contains all information for tracking sound info.
///
/// You generally don't have to create this yourself, it will be part
/// of your `Context` object.
pub struct AudioContext {
    fs: Filesystem,
    stream: rodio::MixerDeviceSink,
}

impl AudioContext {
    /// Create new `AudioContext`.
    pub fn new(fs: &Filesystem) -> GameResult<Self> {
        let stream = rodio::DeviceSinkBuilder::open_default_sink().map_err(|_e| {
            GameError::AudioError(String::from(
                "Could not initialize sound system using default output device (for some reason)",
            ))
        })?;
        Ok(Self {
            fs: fs.clone(),
            stream,
        })
    }
}

impl AudioContext {
    /// Returns the audio device.
    #[inline]
    pub fn device(&self) -> &rodio::MixerDeviceSink {
        &self.stream
    }
}

impl fmt::Debug for AudioContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<AudioContext: {self:p}>")
    }
}

/// Static sound data stored in memory.
/// It is `Arc`'ed, so cheap to clone.
#[derive(Clone, Debug)]
pub struct SoundData(Arc<[u8]>);

impl SoundData {
    /// Load the file at the given path and create a new `SoundData` from it.
    pub fn new<P: AsRef<path::Path>>(fs: &impl Has<Filesystem>, path: P) -> GameResult<Self> {
        let data = fs.retrieve().read(path.as_ref())?;
        Self::from_bytes(&data)
    }

    /// Copies the data in the given slice into a new `SoundData` object.
    pub fn from_bytes(data: &[u8]) -> GameResult<Self> {
        let this = Self(Arc::from(data));
        if let Err(err) = this.decoder() {
            return Err(GameError::AudioError(format!(
                "Could not decode the given audio data: {err}"
            )));
        }
        Ok(this)
    }

    fn decoder(&self) -> Result<rodio::Decoder<io::Cursor<Self>>, rodio::decoder::DecoderError> {
        let cursor = io::Cursor::new(self.clone());
        rodio::Decoder::new(cursor)
    }
}

impl AsRef<[u8]> for SoundData {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// A trait defining the operations possible on a sound;
/// it is implemented by both `Source` and `SpatialSource`.
pub trait SoundSource {
    /// Plays the audio source; restarts the sound if currently playing
    fn play(&self) {
        self.stop();
        self.play_later();
        self.resume();
    }

    /// Plays the `SoundSource`; waits until done if the sound is currently playing
    fn play_later(&self);

    /// Play source "in the background"; cannot be stopped
    fn play_detached(self);

    /// Sets the source to repeat playback infinitely on next [`play()`](#method.play)
    fn set_repeat(&mut self, repeat: bool);

    /// Sets the fade-in time of the source
    fn set_fade_in(&mut self, dur: time::Duration);

    /// Sets the time from which playback begins, skipping audio up to that point.
    ///
    /// Calls to [`elapsed()`](#tymethod.elapsed) will measure from this point, ignoring skipped time.
    ///
    /// Effects such as [`set_fade_in()`](#tymethod.set_fade_in) or [`set_pitch()`](#tymethod.set_pitch)
    /// will apply from this new start.
    ///
    /// If [`set_repeat()`](#tymethod.set_repeat) is set to true, then after looping, the audio will return
    /// to the original beginning of the source, rather than the time specified here.
    fn set_start(&mut self, dur: time::Duration);

    /// Sets the speed ratio (by adjusting the playback speed)
    fn set_pitch(&mut self, ratio: f32);

    /// Gets whether or not the source is set to repeat.
    fn repeat(&self) -> bool;

    /// Pauses playback
    fn pause(&self);

    /// Resumes playback
    fn resume(&self);

    /// Stops playback
    fn stop(&self);

    /// Returns whether or not the source is stopped
    /// -- that is, has no more data to play.
    fn stopped(&self) -> bool;

    /// Gets the current volume.
    fn volume(&self) -> f32;

    /// Sets the current volume.
    fn set_volume(&mut self, value: f32);

    /// Get whether or not the source is paused.
    fn paused(&self) -> bool;

    /// Get whether or not the source is playing (ie, not paused
    /// and not stopped).
    fn playing(&self) -> bool;

    /// Get the time the source has been playing since the last call to [`play()`](#method.play).
    ///
    /// Time measurement is based on audio samples consumed, so it may drift from the system
    fn elapsed(&self) -> time::Duration;

    /// Set the update interval of the internal sample counter.
    ///
    /// This parameter determines the precision of the time measured by [`elapsed()`](#method.elapsed).
    fn set_query_interval(&mut self, t: time::Duration);
}

/// Internal state used by audio sources.
#[derive(Debug)]
struct SourceState {
    data: SoundData,
    repeat: bool,
    fade_in: time::Duration,
    skip_duration: time::Duration,
    speed: f32,
    query_interval: time::Duration,
    play_time: Arc<AtomicU64>,
}

impl SourceState {
    /// Create a new `SourceState` based around the given `SoundData`
    pub fn new(data: SoundData) -> Self {
        SourceState {
            data,
            repeat: false,
            fade_in: time::Duration::from_millis(0),
            skip_duration: time::Duration::from_millis(0),
            speed: 1.0,
            query_interval: time::Duration::from_millis(100),
            play_time: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Sets the source to repeat playback infinitely on next [`play()`](#method.play)
    pub fn set_repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    /// Sets the fade-in time of the source.
    pub fn set_fade_in(&mut self, dur: time::Duration) {
        self.fade_in = dur;
    }

    pub fn set_start(&mut self, dur: time::Duration) {
        self.skip_duration = dur;
    }

    /// Sets the pitch ratio (by adjusting the playback speed).
    pub fn set_pitch(&mut self, ratio: f32) {
        self.speed = ratio;
    }

    /// Gets whether or not the source is set to repeat.
    pub fn repeat(&self) -> bool {
        self.repeat
    }

    /// Get the time the source has been playing since the last call to [`play()`](#method.play).
    ///
    /// Time measurement is based on audio samples consumed, so it may drift from the system
    /// clock over longer periods of time.
    pub fn elapsed(&self) -> time::Duration {
        let t = self.play_time.load(Ordering::Relaxed);
        time::Duration::from_micros(t)
    }

    /// Set the update interval of the internal sample counter.
    ///
    /// This parameter determines the precision of the time measured by [`elapsed()`](#method.elapsed).
    pub fn set_query_interval(&mut self, t: time::Duration) {
        self.query_interval = t;
    }

    fn to_source(&self) -> impl rodio::Source + Send + 'static {
        use rodio::Source;

        let counter = self.play_time.clone();
        let period_mus = self.query_interval.as_micros() as u64;
        // We can't give zero here so give 1µs which is quite the same
        let fade_in = self.fade_in.max(time::Duration::from_micros(1));

        // Creating a new Decoder each time seems a little messy,
        // since it may do checking and data-type detection that is
        // redundant, but it's not super expensive.
        // See https://github.com/ggez/ggez/issues/98 for discussion
        let decoder = rodio::Decoder::new(io::Cursor::new(self.data.clone())).unwrap();

        let source: Box<dyn rodio::Source + Send> = if self.repeat {
            Box::new(decoder.repeat_infinite())
        } else {
            Box::new(decoder)
        };

        source
            .skip_duration(self.skip_duration)
            .speed(self.speed)
            .fade_in(fade_in)
            .periodic_access(self.query_interval, move |_| {
                let _ = counter.fetch_add(period_mus, Ordering::Relaxed);
            })
    }
}

/// A source of audio data that is connected to an output
/// channel and ready to play.  It will stop playing when
/// dropped.
// TODO LATER: Check and see if this matches Love2d's semantics!
// Eventually it might read from a streaming decoder of some kind,
// but for now it is just an in-memory SoundData structure.
pub struct Source {
    sink: rodio::Player,
    state: SourceState,
}

impl Source {
    /// Create a new `Source` from the given file.
    pub fn new(ctx: &impl Has<AudioContext>, path: impl AsRef<path::Path>) -> GameResult<Self> {
        let audio = ctx.retrieve();
        let data = SoundData::new(&audio.fs, path.as_ref())?;
        Self::from_data(audio, data)
    }

    /// Creates a new `Source` using the given `SoundData` object.
    pub fn from_data(audio: &impl Has<AudioContext>, data: SoundData) -> GameResult<Self> {
        let state = SourceState::new(data);
        let sink = rodio::Player::connect_new(audio.retrieve().stream.mixer());
        Ok(Source { sink, state })
    }
}

impl SoundSource for Source {
    fn play_later(&self) {
        self.sink.append(self.state.to_source());
    }

    fn play_detached(self) {
        self.play();
        self.sink.detach();
    }

    fn set_repeat(&mut self, repeat: bool) {
        self.state.set_repeat(repeat)
    }
    fn set_fade_in(&mut self, dur: time::Duration) {
        self.state.set_fade_in(dur)
    }
    fn set_start(&mut self, dur: time::Duration) {
        self.state.set_start(dur)
    }
    fn set_pitch(&mut self, ratio: f32) {
        self.state.set_pitch(ratio)
    }
    fn repeat(&self) -> bool {
        self.state.repeat()
    }
    fn pause(&self) {
        self.sink.pause()
    }
    fn resume(&self) {
        self.sink.play()
    }

    fn stop(&self) {
        self.state.play_time.store(0, Ordering::SeqCst);
        self.sink.clear();
    }

    fn stopped(&self) -> bool {
        self.sink.empty()
    }

    fn volume(&self) -> f32 {
        self.sink.volume()
    }

    fn set_volume(&mut self, value: f32) {
        self.sink.set_volume(value)
    }

    fn paused(&self) -> bool {
        self.sink.is_paused()
    }

    fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }

    fn elapsed(&self) -> time::Duration {
        self.state.elapsed()
    }

    fn set_query_interval(&mut self, t: time::Duration) {
        self.state.set_query_interval(t)
    }
}

impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Audio source: {self:p}>")
    }
}

/// A source of audio data located in space relative to a listener's ears.
/// Will stop playing when dropped.
pub struct SpatialSource {
    sink: rodio::SpatialPlayer,
    state: SourceState,
}

impl SpatialSource {
    /// Create a new `SpatialSource` from the given file.
    pub fn new(ctx: &impl Has<AudioContext>, path: impl AsRef<path::Path>) -> GameResult<Self> {
        let audio = ctx.retrieve();
        let data = SoundData::new(&audio.fs, path.as_ref())?;
        Self::from_data(audio, data)
    }

    /// Creates a new `SpatialSource` using the given `SoundData` object.
    pub fn from_data(audio: &impl Has<AudioContext>, data: SoundData) -> GameResult<Self> {
        let audio = audio.retrieve();

        let state = SourceState::new(data);
        let sink = rodio::SpatialPlayer::connect_new(
            audio.stream.mixer(),
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        );

        Ok(SpatialSource { sink, state })
    }
}

impl SoundSource for SpatialSource {
    /// Plays the `SpatialSource`; waits until done if the sound is currently playing.
    fn play_later(&self) {
        self.sink.append(self.state.to_source());
    }

    fn play_detached(self) {
        self.play();
        self.sink.detach();
    }

    fn set_repeat(&mut self, repeat: bool) {
        self.state.set_repeat(repeat)
    }

    fn set_fade_in(&mut self, dur: time::Duration) {
        self.state.set_fade_in(dur)
    }

    fn set_start(&mut self, dur: time::Duration) {
        self.state.set_start(dur)
    }

    fn set_pitch(&mut self, ratio: f32) {
        self.state.set_pitch(ratio)
    }

    fn repeat(&self) -> bool {
        self.state.repeat()
    }

    fn pause(&self) {
        self.sink.pause()
    }

    fn resume(&self) {
        self.sink.play()
    }

    fn stop(&self) {
        self.state.play_time.store(0, Ordering::SeqCst);
        self.sink.clear();
    }

    fn stopped(&self) -> bool {
        self.sink.empty()
    }

    fn volume(&self) -> f32 {
        self.sink.volume()
    }

    fn set_volume(&mut self, value: f32) {
        self.sink.set_volume(value)
    }

    fn paused(&self) -> bool {
        self.sink.is_paused()
    }

    fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }

    fn elapsed(&self) -> time::Duration {
        self.state.elapsed()
    }

    fn set_query_interval(&mut self, t: time::Duration) {
        self.state.set_query_interval(t)
    }
}

impl SpatialSource {
    /// Set location of the sound.
    pub fn set_position<P>(&self, pos: P)
    where
        P: Into<mint::Point3<f32>>,
    {
        self.sink.set_emitter_position(pos.into().into());
    }

    /// Set locations of the listener's ears
    pub fn set_ears<P>(&self, left: P, right: P)
    where
        P: Into<mint::Point3<f32>>,
    {
        self.sink.set_left_ear_position(left.into().into());
        self.sink.set_right_ear_position(right.into().into());
    }
}

impl fmt::Debug for SpatialSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Spatial audio source: {self:p}>")
    }
}
