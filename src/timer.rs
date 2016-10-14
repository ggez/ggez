//! Timing and measurement functions.
//!
//! ggez does not try to do any framerate limitation by default.
//! If you want to run at anything other than full-bore max speed all the time,
//! you should use either the `sleep()` or `sleep_until_next_frame()` function in
//! this module at the end of your `GameState.draw()` callback.
//!
//! `sleep()` with a duration of 0 will just yield to the OS so it has a chance
//! to breathe before continuing with your game,  while
//! `sleep_until_next_frame()` will attempt to calculate how long it should
//! wait to hit the desired FPS and sleep that long.

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
struct LogBuffer<T> {
    head: usize,
    size: usize,
    contents: Vec<T>,
}

impl<T> LogBuffer<T>
    where T: Clone + Copy
{
    fn new(size: usize, init_val: T) -> LogBuffer<T> {
        let mut v = Vec::with_capacity(size);
        v.resize(size, init_val);
        println!("Vec length: {}", v.len());
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
// This is independent of SDL.
// According to the rust-sdl2 maintainers,
// SDL's time functions are of dubious thread-safety,
// while Rust's are pretty solid.
pub struct TimeContext {
    init_instant: time::Instant,
    last_instant: time::Instant,
    frame_durations: LogBuffer<time::Duration>,
}


// How many frames we log update times for.
// Nominally, one second, give or take.
const TIME_LOG_FRAMES: u32 = 60;

impl TimeContext {
    /// Creates a new `TimeContext` and initializes the start to this instant.
    pub fn new() -> TimeContext {
        TimeContext {
            init_instant: time::Instant::now(),
            last_instant: time::Instant::now(),
            frame_durations: LogBuffer::new(TIME_LOG_FRAMES as usize, time::Duration::new(0, 0)),
        }
    }

    /// Update the state of the TimeContext to record that
    /// another frame has taken place.
    ///
    /// It's usually not necessary to call this function yourself,
    /// the `Game` will do it for you.
    pub fn tick(&mut self) {
        let now = time::Instant::now();
        let time_since_last = now - self.last_instant;
        self.frame_durations.push(time_since_last);
        self.last_instant = now;
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
/// over the last 60 frames.
// TODO: We COULD make it so that a TimeContext has a set_average_window
// function.  Do we care?
pub fn get_average_delta(ctx: &Context) -> time::Duration {
    let tc = &ctx.timer_context;
    let init = time::Duration::new(0, 0);
    let sum = tc.frame_durations.contents().iter().fold(init, |d1, d2| d1 + *d2);
    sum / TIME_LOG_FRAMES
}

/// A convenience function to convert a Rust `Duration` type
/// to a (less precise but more useful) f64.
pub fn duration_to_f64(d: time::Duration) -> f64 {
    let seconds = d.as_secs() as f64;
    let nanos = d.subsec_nanos() as f64;
    seconds + (nanos * 1e-9)
}

/// Gets the FPS of the game, averaged over the last
/// 60 frames.
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

/// This function will *attempt* to sleep the current
/// thread until the beginning of the next frame should
/// occur, to reach the desired FPS.
///
/// This is a bit of a prototype; it may not work well,
/// it may not work reliably cross-platform, and it may
/// not be a good idea in the first place.
/// It depends on how accurate Rust's `std::thread::sleep()`
/// is, which is to some extent at the mercy of what the
/// OS decides to do.
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
