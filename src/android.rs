//! Android support.
//!
//! Support is limited. See the [Android guide](https://github.com/ggez/ggez/blob/master/docs/BuildingForEveryPlatform.md#android)
//! and [example packager](https://github.com/ggez/ggez/tree/master/examples/android)
//! for project setup and APK build instructions.
//!
//! An Android game is a `cdylib` that exports an `android_main` function instead of a
//! binary with `main`. Hand the [`AndroidApp`] it receives to ggez with [`set_android_app`]
//! before building a [`Context`](crate::Context), then build and run as usual:
//!
//! ```rust,ignore
//! #[no_mangle]
//! fn android_main(app: ggez::android::AndroidApp) {
//!     ggez::android::set_android_app(app);
//!     let (mut ctx, event_loop) = ggez::ContextBuilder::new("game", "author").build().unwrap();
//!     let state = MyGame::new(&mut ctx).unwrap();
//!     ggez::event::run(ctx, event_loop, state).unwrap();
//! }
//! ```
//!
//! Enable either `android-native-activity` or `android-game-activity` features.
//! `NativeActivity` needs no Java code, so it is the simplest place to start.

use std::sync::Mutex;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::WindowId;

use crate::{GameError, GameResult};

pub use winit::platform::android::activity::AndroidApp;

static ANDROID_APP: Mutex<Option<AndroidApp>> = Mutex::new(None);

/// Gives ggez the [`AndroidApp`] that was passed to `android_main`.
///
/// This must be called before [`ContextBuilder::build`](crate::ContextBuilder::build).
/// Android may call `android_main` again in the same process when it recreates the
/// activity, so each call replaces the previous app.
pub fn set_android_app(app: AndroidApp) {
    *ANDROID_APP.lock().unwrap_or_else(|e| e.into_inner()) = Some(app);
}

/// Returns the [`AndroidApp`] given to [`set_android_app`].
pub fn android_app() -> GameResult<AndroidApp> {
    ANDROID_APP
        .lock()
        .map_err(|_| GameError::LockError)?
        .clone()
        .ok_or_else(|| {
            GameError::CustomError(String::from(
                "ggez::android::set_android_app must be called before building a Context",
            ))
        })
}

/// Creates the winit event loop for the app given to [`set_android_app`].
///
/// [`ContextBuilder::build`](crate::ContextBuilder::build) does this for you; it is only
/// needed by `from_conf` functions passed to
/// [`ContextBuilder::custom_build`](crate::ContextBuilder::custom_build), which should also
/// call [`wait_for_native_window`] before creating a
/// [`GraphicsContext`](crate::graphics::GraphicsContext).
pub fn create_event_loop() -> GameResult<EventLoop<()>> {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    Ok(EventLoop::builder()
        .with_android_app(android_app()?)
        .build()?)
}

/// Pumps the event loop until Android hands us a native window, which wgpu needs
/// before it can create a surface.
pub fn wait_for_native_window(event_loop: &mut EventLoop<()>) -> GameResult {
    struct WaitForWindow {
        resumed: bool,
    }

    impl ApplicationHandler<()> for WaitForWindow {
        fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
            self.resumed = true;
        }

        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    }

    let mut handler = WaitForWindow { resumed: false };
    while !handler.resumed {
        if let PumpStatus::Exit(code) = event_loop.pump_app_events(None, &mut handler) {
            return Err(GameError::CustomError(format!(
                "event loop exited with code {code} before the Android window was created"
            )));
        }
    }
    Ok(())
}
