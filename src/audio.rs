//! Provides an interface to output sound to the user's speakers.
//!
//! It consists of two main types: [`SoundData`](struct.SoundData.html)
//! is just an array of raw sound data bytes, and a [`Source`](struct.Source.html) is a
//! `SoundData` connected to a particular sound channel ready to be played.

use std::fmt;
use std::io;
use std::io::Read;
use std::mem;
use std::path;
use std::time;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::context::Context;
use crate::error::GameError;
use crate::error::GameResult;
use crate::filesystem;

/// A trait object defining an audio context, allowing us to someday
/// use something other than `rodio` if we really want.
///
/// End-users usually don't need to mess with this, but it's there
/// if you want to bypass `ggez`'s sound functionality and write your
/// own.
pub trait AudioContext {
    /// Returns the audio device.
    fn device(&self) -> &rodio::OutputStreamHandle;
}

/// A struct that contains all information for tracking sound info.
///
/// You generally don't have to create this yourself, it will be part
/// of your `Context` object.
pub(crate) struct RodioAudioContext {
    _stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
}

impl RodioAudioContext {
    /// Create new `RodioAudioContext`.
    pub fn new() -> GameResult<Self> {
        let (stream, stream_handle) = rodio::OutputStream::try_default().map_err(|_e| {
            GameError::AudioError(String::from(
                "Could not initialize sound system using default output device (for some reason)",
            ))
        })?;
        Ok(Self {
            _stream: stream,
            stream_handle,
        })
    }
}

impl AudioContext for RodioAudioContext {
    fn device(&self) -> &rodio::OutputStreamHandle {
        &self.stream_handle
    }
}

impl fmt::Debug for RodioAudioContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<RodioAudioContext: {:p}>", self)
    }
}

/// A structure that implements `AudioContext` but does nothing; serves as a
/// stub for when you don't need audio.  Will panic if you try to actually
/// play sound from it.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NullAudioContext;

impl AudioContext for NullAudioContext {
    fn device(&self) -> &rodio::OutputStreamHandle {
        panic!("Audio module disabled")
    }
}

/// Static sound data stored in memory.
/// It is `Arc`'ed, so cheap to clone.
#[derive(Clone, Debug)]
pub struct SoundData(Arc<[u8]>);

impl SoundData {
    /// Load the file at the given path and create a new `SoundData` from it.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let file = &mut filesystem::open(context, path)?;
        SoundData::from_read(file)
    }

    /// Copies the data in the given slice into a new `SoundData` object.
    pub fn from_bytes(data: &[u8]) -> Self {
        SoundData(Arc::from(data))
    }

    /// Creates a `SoundData` from any `Read` object; this involves
    /// copying it into a buffer.
    pub fn from_read<R>(reader: &mut R) -> GameResult<Self>
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer)?;

        Ok(SoundData::from(buffer))
    }

    /// Indicates if the data can be played as a sound.
    pub fn can_play(&self) -> bool {
        let cursor = io::Cursor::new(self.clone());
        rodio::Decoder::new(cursor).is_ok()
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

/// A trait defining the operations possible on a sound;
/// it is implemented by both `Source` and `SpatialSource`.
pub trait SoundSource {
    /// Plays the audio source; restarts the sound if currently playing
    fn play(&mut self, ctx: &Context) -> GameResult {
        self.stop(ctx)?;
        self.play_later()
    }

    /// Plays the `SoundSource`; waits until done if the sound is currently playing
    fn play_later(&self) -> GameResult;

    /// Play source "in the background"; cannot be stopped
    fn play_detached(&mut self, ctx: &Context) -> GameResult;

    /// Sets the source to repeat playback infinitely on next [`play()`](#method.play)
    fn set_repeat(&mut self, repeat: bool);

    /// Sets the fade-in time of the source
    fn set_fade_in(&mut self, dur: time::Duration);

    /// Sets the speed ratio (by adjusting the playback speed)
    fn set_pitch(&mut self, ratio: f32);

    /// Gets whether or not the source is set to repeat.
    fn repeat(&self) -> bool;

    /// Pauses playback
    fn pause(&self);

    /// Resumes playback
    fn resume(&self);

    /// Stops playback
    fn stop(&mut self, ctx: &Context) -> GameResult;

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
pub(crate) struct SourceState {
    data: io::Cursor<SoundData>,
    repeat: bool,
    fade_in: time::Duration,
    speed: f32,
    query_interval: time::Duration,
    play_time: Arc<AtomicUsize>,
}

impl SourceState {
    /// Create a new `SourceState` based around the given `SoundData`
    pub fn new(cursor: io::Cursor<SoundData>) -> Self {
        SourceState {
            data: cursor,
            repeat: false,
            fade_in: time::Duration::from_millis(0),
            speed: 1.0,
            query_interval: time::Duration::from_millis(100),
            play_time: Arc::new(AtomicUsize::new(0)),
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
        let t = self.play_time.load(Ordering::SeqCst);
        time::Duration::from_micros(t as u64)
    }

    /// Set the update interval of the internal sample counter.
    ///
    /// This parameter determines the precision of the time measured by [`elapsed()`](#method.elapsed).
    pub fn set_query_interval(&mut self, t: time::Duration) {
        self.query_interval = t;
    }
}

/// A source of audio data that is connected to an output
/// channel and ready to play.  It will stop playing when
/// dropped.
// TODO LATER: Check and see if this matches Love2d's semantics!
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
    sink: rodio::Sink,
    state: SourceState,
}

impl Source {
    /// Create a new `Source` from the given file.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let data = SoundData::new(context, path)?;
        Source::from_data(context, data)
    }

    /// Creates a new `Source` using the given `SoundData` object.
    pub fn from_data(context: &mut Context, data: SoundData) -> GameResult<Self> {
        if !data.can_play() {
            return Err(GameError::AudioError(
                "Could not decode the given audio data".to_string(),
            ));
        }
        let sink = rodio::Sink::try_new(&context.audio_context.device())?;
        let cursor = io::Cursor::new(data);
        Ok(Source {
            sink,
            state: SourceState::new(cursor),
        })
    }
}

impl SoundSource for Source {
    fn play_later(&self) -> GameResult {
        // Creating a new Decoder each time seems a little messy,
        // since it may do checking and data-type detection that is
        // redundant, but it's not super expensive.
        // See https://github.com/ggez/ggez/issues/98 for discussion
        use rodio::Source;
        let cursor = self.state.data.clone();

        let counter = self.state.play_time.clone();
        let period_mus = self.state.query_interval.as_secs() as usize * 1_000_000
            + self.state.query_interval.subsec_micros() as usize;

        if self.state.repeat {
            let sound = rodio::Decoder::new(cursor)?
                .repeat_infinite()
                .speed(self.state.speed)
                .fade_in(self.state.fade_in)
                .periodic_access(self.state.query_interval, move |_| {
                    let _ = counter.fetch_add(period_mus, Ordering::SeqCst);
                });
            self.sink.append(sound);
        } else {
            let sound = rodio::Decoder::new(cursor)?
                .speed(self.state.speed)
                .fade_in(self.state.fade_in)
                .periodic_access(self.state.query_interval, move |_| {
                    let _ = counter.fetch_add(period_mus, Ordering::SeqCst);
                });
            self.sink.append(sound);
        }

        Ok(())
    }

    fn play_detached(&mut self, ctx: &Context) -> GameResult {
        self.stop(ctx)?;
        self.play_later()?;

        let new_sink = rodio::Sink::try_new(ctx.audio_context.device())?;
        let old_sink = mem::replace(&mut self.sink, new_sink);
        old_sink.detach();

        Ok(())
    }

    fn set_repeat(&mut self, repeat: bool) {
        self.state.set_repeat(repeat)
    }
    fn set_fade_in(&mut self, dur: time::Duration) {
        self.state.set_fade_in(dur)
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

    fn stop(&mut self, ctx: &Context) -> GameResult {
        // Sinks cannot be reused after calling `.stop()`. See
        // https://github.com/tomaka/rodio/issues/171 for information.
        // To stop the current sound we have to drop the old sink and
        // create a new one in its place.
        // This is most ugly because in order to create a new sink
        // we need a `device`. However, we can only get the default
        // device without having access to a context. Currently that's
        // fine because the `RodioAudioContext` uses the default device too,
        // but it may cause problems in the future if devices become
        // customizable.

        // We also need to carry over information from the previous sink.
        let volume = self.volume();

        let device = ctx.audio_context.device();
        self.sink = rodio::Sink::try_new(&device)?;
        self.state.play_time.store(0, Ordering::SeqCst);

        // Restore information from the previous link.
        self.set_volume(volume);
        Ok(())
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
        write!(f, "<Audio source: {:p}>", self)
    }
}

/// A source of audio data located in space relative to a listener's ears.
/// Will stop playing when dropped.
pub struct SpatialSource {
    sink: rodio::SpatialSink,
    state: SourceState,
    left_ear: mint::Point3<f32>,
    right_ear: mint::Point3<f32>,
    emitter_position: mint::Point3<f32>,
}

impl SpatialSource {
    /// Create a new `SpatialSource` from the given file.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let data = SoundData::new(context, path)?;
        SpatialSource::from_data(context, data)
    }

    /// Creates a new `SpatialSource` using the given `SoundData` object.
    pub fn from_data(context: &mut Context, data: SoundData) -> GameResult<Self> {
        if !data.can_play() {
            return Err(GameError::AudioError(
                "Could not decode the given audio data".to_string(),
            ));
        }
        let sink = rodio::SpatialSink::try_new(
            &context.audio_context.device(),
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        )?;

        let cursor = io::Cursor::new(data);

        Ok(SpatialSource {
            sink,
            state: SourceState::new(cursor),
            left_ear: [-1.0, 0.0, 0.0].into(),
            right_ear: [1.0, 0.0, 0.0].into(),
            emitter_position: [0.0, 0.0, 0.0].into(),
        })
    }
}

impl SoundSource for SpatialSource {
    /// Plays the `SpatialSource`; waits until done if the sound is currently playing.
    fn play_later(&self) -> GameResult {
        // Creating a new Decoder each time seems a little messy,
        // since it may do checking and data-type detection that is
        // redundant, but it's not super expensive.
        // See https://github.com/ggez/ggez/issues/98 for discussion
        use rodio::Source;
        let cursor = self.state.data.clone();

        let counter = self.state.play_time.clone();
        let period_mus = self.state.query_interval.as_secs() as usize * 1_000_000
            + self.state.query_interval.subsec_micros() as usize;

        if self.state.repeat {
            let sound = rodio::Decoder::new(cursor)?
                .repeat_infinite()
                .speed(self.state.speed)
                .fade_in(self.state.fade_in)
                .periodic_access(self.state.query_interval, move |_| {
                    let _ = counter.fetch_add(period_mus, Ordering::SeqCst);
                });
            self.sink.append(sound);
        } else {
            let sound = rodio::Decoder::new(cursor)?
                .speed(self.state.speed)
                .fade_in(self.state.fade_in)
                .periodic_access(self.state.query_interval, move |_| {
                    let _ = counter.fetch_add(period_mus, Ordering::SeqCst);
                });
            self.sink.append(sound);
        }

        Ok(())
    }

    fn play_detached(&mut self, ctx: &Context) -> GameResult {
        self.stop(ctx)?;
        self.play_later()?;

        let device = ctx.audio_context.device();
        let new_sink = rodio::SpatialSink::try_new(
            &device,
            self.emitter_position.into(),
            self.left_ear.into(),
            self.right_ear.into(),
        )?;
        let old_sink = mem::replace(&mut self.sink, new_sink);
        old_sink.detach();

        Ok(())
    }

    fn set_repeat(&mut self, repeat: bool) {
        self.state.set_repeat(repeat)
    }

    fn set_fade_in(&mut self, dur: time::Duration) {
        self.state.set_fade_in(dur)
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

    fn stop(&mut self, ctx: &Context) -> GameResult {
        // Sinks cannot be reused after calling `.stop()`. See
        // https://github.com/tomaka/rodio/issues/171 for information.
        // To stop the current sound we have to drop the old sink and
        // create a new one in its place.
        // This is most ugly because in order to create a new sink
        // we need a `device`. However, we can only get the default
        // device without having access to a context. Currently that's
        // fine because the `RodioAudioContext` uses the default device too,
        // but it may cause problems in the future if devices become
        // customizable.

        // We also need to carry over information from the previous sink.
        let volume = self.volume();

        let device = ctx.audio_context.device();
        self.sink = rodio::SpatialSink::try_new(
            &device,
            self.emitter_position.into(),
            self.left_ear.into(),
            self.right_ear.into(),
        )?;
        self.state.play_time.store(0, Ordering::SeqCst);

        // Restore information from the previous link.
        self.set_volume(volume);
        Ok(())
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
