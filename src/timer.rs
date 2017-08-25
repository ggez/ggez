//! Timing and measurement functions.
//!
//! ggez does not try to do any framerate limitation by default. If
//! you want to run at anything other than full-bore max speed all the
//! time, calling `sleep()` with a duration of 0 will yield to the OS
//! so it has a chance to breathe before continuing with your game,
//! which will prevent it from using 100% CPU unless it really needs
//! to.  Enabling vsync by setting `vsync` in your `Conf` object is
//! generally the best way to cap your displayed framerate.
//!
//! For a more detailed tutorial in how to handle frame timings in games,
//! see <http://gafferongames.com/game-physics/fix-your-timestep/>

use context::Context;

use std::time;
use std::thread;

/// A simple buffer that fills
/// up to a limit and then holds the last
/// N items that have been inserted into it,
/// overwriting old ones in a round-robin fashion.
///
/// It's not quite a ring buffer 'cause you can't
/// remove items from it, it just holds the last N
/// things.
#[derive(Debug, Clone)]
struct LogBuffer<T>
where
    T: Clone,
{
    head: usize,
    size: usize,
    contents: Vec<T>,
}

impl<T> LogBuffer<T>
where
    T: Clone + Copy,
{
    fn new(size: usize, init_val: T) -> LogBuffer<T> {
        let mut v = Vec::with_capacity(size);
        v.resize(size, init_val);
        LogBuffer {
            head: 0,
            size: size,
            contents: v,
        }
    }

    /// Pushes a new item into the logbuffer, overwriting
    /// the oldest item in it.
    fn push(&mut self, item: T) {
        self.head = (self.head + 1) % self.size;
        self.contents[self.head] = item;
    }

    /// Returns a slice pointing at the contents of the buffer.
    /// They are in *no particular order*, and if not all the
    /// slots are filled, the empty slots will be present but
    /// contain the initial value given to `new()`
    ///
    /// We're only using this to log FPS for a short time,
    /// so we don't care for the second or so when it's inaccurate.
    fn contents(&self) -> &[T] {
        &self.contents
    }

    /// Returns the most recent value in the buffer.
    fn latest(&self) -> T {
        self.contents[self.head]
    }
}

/// A structure that contains our time-tracking state.
#[derive(Debug)]
pub struct TimeContext {
    init_instant: time::Instant,
    last_instant: time::Instant,
    frame_durations: LogBuffer<time::Duration>,
    residual_update_dt: time::Duration,
    frame_count: usize,
}


// How many frames we log update times for.
const TIME_LOG_FRAMES: usize = 200;

impl TimeContext {
    /// Creates a new `TimeContext` and initializes the start to this instant.
    pub fn new() -> TimeContext {
        TimeContext {
            init_instant: time::Instant::now(),
            last_instant: time::Instant::now(),
            frame_durations: LogBuffer::new(TIME_LOG_FRAMES, time::Duration::new(0, 0)),
            residual_update_dt: time::Duration::from_secs(0),
            frame_count: 0
        }
    }

    /// Update the state of the TimeContext to record that
    /// another frame has taken place.
    ///
    /// It's usually not necessary to call this function yourself,
    /// the `EventHandler` will do it for you.
    pub fn tick(&mut self) {
        let now = time::Instant::now();
        let time_since_last = now - self.last_instant;
        self.frame_durations.push(time_since_last);
        self.last_instant = now;
        self.frame_count += 1;
    }
}

impl Default for TimeContext {
    fn default() -> Self {
        Self::new()
    }
}


/// Get the time between the start of the last frame and the current one;
/// in other words, the length of the last frame.
pub fn get_delta(ctx: &Context) -> time::Duration {
    let tc = &ctx.timer_context;
    tc.frame_durations.latest()
}


/// Gets the average time of a frame, averaged
/// over the last 200 frames.
pub fn get_average_delta(ctx: &Context) -> time::Duration {
    let tc = &ctx.timer_context;
    let init = time::Duration::new(0, 0);
    let sum = tc.frame_durations
        .contents()
        .iter()
        .fold(init, |d1, d2| d1 + *d2);
    sum / (TIME_LOG_FRAMES as u32)
}

/// A convenience function to convert a Rust `Duration` type
/// to a (less precise but more useful) f64.
///
/// Does not make sure that the `Duration` is within the bounds
/// of the `f64`.
pub fn duration_to_f64(d: time::Duration) -> f64 {
    let seconds = d.as_secs() as f64;
    let nanos = d.subsec_nanos() as f64;
    seconds + (nanos * 1e-9)
}

/// A convenience function to create a Rust `Duration` type
/// from a (less precise but more useful) f64.
///
/// Only handles positive numbers correctly.
pub fn f64_to_duration(t: f64) -> time::Duration {
    debug_assert!(t >= 0.0, "f64_to_duration passed a negative number!");
    let seconds = t.trunc();
    let nanos = t.fract() * 1e9;
    time::Duration::new(seconds as u64, nanos as u32)
}

/// Returns a `Duration` representing how long each
/// frame should be to match the given fps.
///
/// Approximately.
fn fps_as_duration(fps: u64) -> time::Duration {
    let target_dt_seconds = 1.0 / (fps as f64);
    f64_to_duration(target_dt_seconds)
}

/// Gets the FPS of the game, averaged over the last
/// 200 frames.
pub fn get_fps(ctx: &Context) -> f64 {
    let duration_per_frame = get_average_delta(ctx);
    let seconds_per_frame = duration_to_f64(duration_per_frame);
    1.0 / seconds_per_frame
}

/// Returns the time since the game was initialized.
pub fn get_time_since_start(ctx: &Context) -> time::Duration {
    let tc = &ctx.timer_context;
    time::Instant::now() - tc.init_instant
}

/// This function will return true if the time since the
/// last `update()` call has been equal to or greater to
/// the update FPS indicated by the `desired_update_rate`.
/// It keeps track of fractional frames, and does not
/// do any sleeping.
pub fn check_update_time(ctx: &mut Context, desired_update_rate: u64) -> bool {
    let dt = get_delta(ctx);
    let timedata = &mut ctx.timer_context;
    let target_dt = fps_as_duration(desired_update_rate);
    timedata.residual_update_dt += dt;
    if timedata.residual_update_dt > target_dt {
        timedata.residual_update_dt -= target_dt;
        true
    } else {
        false
    }
}

/// This function will *attempt* to sleep the current
/// thread until the beginning of the next frame should
/// occur, to reach the desired FPS.
///
/// This is not an especially precise way to do timing;
/// see the `astroblasto` example for how to do it better.
/// However, this is very convenient for prototyping,
/// so I'm leaving it in.
pub fn sleep_until_next_frame(ctx: &Context, desired_fps: u32) {
    // We assume we'll never sleep more than a second!
    // Using an integer FPS target helps enforce this.
    assert!(desired_fps > 0);
    let tc = &ctx.timer_context;
    let fps_delay = 1.0 / (desired_fps as f64);
    let nanos_per_frame = fps_delay * 1e9;
    let duration_per_frame = time::Duration::new(0, nanos_per_frame as u32);
    let now = time::Instant::now();
    let time_spent_this_frame = now - tc.last_instant;
    if time_spent_this_frame >= duration_per_frame {
        // We don't even yield to the OS in this case
        ()
    } else {
        let duration_to_sleep = duration_per_frame - time_spent_this_frame;
        // println!("Sleeping for {:?}", duration_to_sleep);
        thread::sleep(duration_to_sleep);
    }
}

/// Pauses the current thread for the target duration.
/// Just calls `std::thread::sleep()` so it's as accurate
/// as that is.
pub fn sleep(duration: time::Duration) {
    thread::sleep(duration);
}


/// Gets the number of times the game has gone through its event loop.
///
/// Specifically, the number of times that TimeContext::tick() has been
/// called by it.
pub fn get_ticks(ctx: &Context) -> usize {
    ctx.timer_context.frame_count
}
