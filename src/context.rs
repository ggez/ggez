use std::fmt;
/// We re-export winit so it's easy for people to use the same version as we are
/// without having to mess around figuring it out.
pub use winit;

#[cfg(feature = "audio")]
use crate::audio;
use crate::conf;
use crate::error::GameResult;
use crate::filesystem::Filesystem;
use crate::graphics;
use crate::graphics::GraphicsContext;
use crate::input;
use crate::timer;

/// A `Context` is an object that holds on to global resources.
/// It basically tracks hardware state such as the screen, audio
/// system, timers, and so on.  Generally this type can **not**
/// be shared/sent between threads and only one `Context` can exist at a time.  Trying
/// to create a second one will fail.  It is fine to drop a `Context`
/// and create a new one, but this will also close and re-open your
/// game's window.
///
/// Most functions that interact with the hardware, for instance
/// drawing things, playing sounds, or loading resources will need
/// to access one, or rarely even two, of its sub-contexts.
/// It is an error to create some type that
/// relies upon a `Context`, such as `Image`, and then drop the `Context`
/// and try to draw the old `Image` with the new `Context`.
///
/// The fields in this struct used to be basically undocumented features,
/// only here to make it easier to debug, or to let advanced users
/// hook into the guts of ggez and make it do things it normally
/// can't. Now that `ggez`'s module-level functions, taking the whole `Context`
/// have been deprecated, calling their methods directly is recommended.
pub struct Context {
    /// Filesystem state.
    pub fs: Filesystem,
    /// Graphics state.
    pub gfx: GraphicsContext,
    /// Timer state.
    pub time: timer::TimeContext,
    /// Audio context.
    #[cfg(feature = "audio")]
    pub audio: audio::AudioContext,
    /// Keyboard input context.
    pub keyboard: input::keyboard::KeyboardContext,
    /// Mouse input context.
    pub mouse: input::mouse::MouseContext,
    /// Gamepad input context.
    #[cfg(feature = "gamepad")]
    pub gamepad: input::gamepad::GamepadContext,

    /// The Conf object the Context was created with.
    /// It's here just so that we can see the original settings,
    /// updating it will have no effect.
    pub(crate) conf: conf::Conf,
    /// Controls whether or not the event loop should be running.
    /// This is internally controlled by the outcome of [`quit_event`](crate::event::EventHandler::quit_event), requested through [`event::quit()`](crate::event::quit()).
    pub continuing: bool,
    /// Whether or not a `quit_event` has been requested.
    /// Set this with `ggez::event::quit()`.
    ///
    /// It's exposed here for people who want to roll their own event loop.
    pub quit_requested: bool,
}

// This is ugly and hacky but greatly improves ergonomics.

pub trait Has<T> {
    fn get(&self) -> &T;
}

impl<T> Has<T> for T {
    #[inline]
    fn get(&self) -> &T {
        self
    }
}

impl Has<Filesystem> for Context {
    #[inline]
    fn get(&self) -> &Filesystem {
        &self.fs
    }
}

impl Has<GraphicsContext> for Context {
    #[inline]
    fn get(&self) -> &GraphicsContext {
        &self.gfx
    }
}

#[cfg(feature = "audio")]
impl Has<audio::AudioContext> for Context {
    #[inline]
    fn get(&self) -> &audio::AudioContext {
        &self.audio
    }
}

pub trait HasMut<T> {
    fn get_mut(&mut self) -> &mut T;
}

impl<T> HasMut<T> for T {
    #[inline]
    fn get_mut(&mut self) -> &mut T {
        self
    }
}

impl HasMut<GraphicsContext> for Context {
    #[inline]
    fn get_mut(&mut self) -> &mut GraphicsContext {
        &mut self.gfx
    }
}

pub trait HasTwo<T, U> {
    fn get_first(&self) -> &T;
    fn get_second(&self) -> &U;
}

impl<T, U> HasTwo<T, U> for (&T, &U) {
    #[inline]
    fn get_first(&self) -> &T {
        self.0
    }

    #[inline]
    fn get_second(&self) -> &U {
        self.1
    }
}

impl HasTwo<Filesystem, GraphicsContext> for Context {
    #[inline]
    fn get_first(&self) -> &Filesystem {
        &self.fs
    }

    #[inline]
    fn get_second(&self) -> &GraphicsContext {
        &self.gfx
    }
}

#[cfg(feature = "audio")]
impl HasTwo<Filesystem, crate::audio::AudioContext> for Context {
    fn get_first(&self) -> &Filesystem {
        &self.fs
    }

    fn get_second(&self) -> &crate::audio::AudioContext {
        &self.audio
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Context: {:p}>", self)
    }
}

impl Context {
    /// Tries to create a new Context using settings from the given [`Conf`](../conf/struct.Conf.html) object.
    /// Usually called by [`ContextBuilder::build()`](struct.ContextBuilder.html#method.build).
    fn from_conf(
        conf: conf::Conf,
        mut fs: Filesystem,
    ) -> GameResult<(Context, winit::event_loop::EventLoop<()>)> {
        #[cfg(feature = "audio")]
        let audio_context = audio::AudioContext::new()?;
        let events_loop = winit::event_loop::EventLoop::new();
        let timer_context = timer::TimeContext::new();
        let graphics_context =
            graphics::context::GraphicsContext::new(&events_loop, &conf, &mut fs)?;

        let ctx = Context {
            conf,
            fs,
            gfx: graphics_context,
            continuing: true,
            quit_requested: false,
            time: timer_context,
            #[cfg(feature = "audio")]
            audio: audio_context,
            keyboard: input::keyboard::KeyboardContext::new(),
            mouse: input::mouse::MouseContext::new(),
            #[cfg(feature = "gamepad")]
            gamepad: input::gamepad::GamepadContext::new()?,
        };

        Ok((ctx, events_loop))
    }
}

use std::borrow::Cow;
use std::path;

/// A builder object for creating a [`Context`](struct.Context.html).
#[derive(Debug, Clone, PartialEq)]
pub struct ContextBuilder {
    pub(crate) game_id: String,
    pub(crate) author: String,
    pub(crate) conf: conf::Conf,
    pub(crate) resources_dir_name: String,
    pub(crate) resources_zip_name: String,
    pub(crate) paths: Vec<path::PathBuf>,
    pub(crate) memory_zip_files: Vec<Cow<'static, [u8]>>,
    pub(crate) load_conf_file: bool,
}

impl ContextBuilder {
    /// Create a new `ContextBuilder` with default settings.
    pub fn new(game_id: &str, author: &str) -> Self {
        Self {
            game_id: game_id.to_string(),
            author: author.to_string(),
            conf: conf::Conf::default(),
            resources_dir_name: "resources".to_string(),
            resources_zip_name: "resources.zip".to_string(),
            paths: vec![],
            memory_zip_files: vec![],
            load_conf_file: true,
        }
    }

    /// Sets the window setup settings.
    #[must_use]
    pub fn window_setup(mut self, setup: conf::WindowSetup) -> Self {
        self.conf.window_setup = setup;
        self
    }

    /// Sets the window mode settings.
    #[must_use]
    pub fn window_mode(mut self, mode: conf::WindowMode) -> Self {
        self.conf.window_mode = mode;
        self
    }

    /// Sets the graphics backend.
    #[must_use]
    pub fn backend(mut self, backend: conf::Backend) -> Self {
        self.conf.backend = backend;
        self
    }

    /// Sets all the config options, overriding any previous
    /// ones from [`window_setup()`](#method.window_setup),
    /// [`window_mode()`](#method.window_mode), and
    /// [`backend()`](#method.backend).  These are used as
    /// defaults and are overridden by any external config
    /// file found.
    #[must_use]
    pub fn default_conf(mut self, conf: conf::Conf) -> Self {
        self.conf = conf;
        self
    }

    /// Sets resources dir name.
    /// Default resources dir name is `resources`.
    #[must_use]
    pub fn resources_dir_name(mut self, new_name: impl ToString) -> Self {
        self.resources_dir_name = new_name.to_string();
        self
    }

    /// Sets resources zip name.
    /// Default resources dir name is `resources.zip`.
    #[must_use]
    pub fn resources_zip_name(mut self, new_name: impl ToString) -> Self {
        self.resources_zip_name = new_name.to_string();
        self
    }

    /// Add a new read-only filesystem path to the places to search
    /// for resources.
    #[must_use]
    pub fn add_resource_path<T>(mut self, path: T) -> Self
    where
        T: Into<path::PathBuf>,
    {
        self.paths.push(path.into());
        self
    }

    /// Add a new zip file from bytes whose contents will be searched
    /// for resources. The zip file will be stored in-memory.
    /// You can pass it a static slice, a `Vec` of bytes, etc.
    ///
    /// ```ignore
    /// use ggez::context::ContextBuilder;
    /// let _ = ContextBuilder::new()
    ///     .add_zipfile_bytes(include_bytes!("../resources.zip").to_vec())
    ///     .build();
    /// ```
    #[must_use]
    pub fn add_zipfile_bytes<B>(mut self, bytes: B) -> Self
    where
        B: Into<Cow<'static, [u8]>>,
    {
        let cow = bytes.into();
        self.memory_zip_files.push(cow);
        self
    }

    /// Specifies whether or not to load the `conf.toml` file if it
    /// exists and use its settings to override the provided values.
    /// Defaults to `true` which is usually what you want, but being
    /// able to fiddle with it is sometimes useful for debugging.
    #[must_use]
    pub fn with_conf_file(mut self, load_conf_file: bool) -> Self {
        self.load_conf_file = load_conf_file;
        self
    }

    /// Build the `Context`.
    pub fn build(self) -> GameResult<(Context, winit::event_loop::EventLoop<()>)> {
        let mut fs = Filesystem::new(
            self.game_id.as_ref(),
            self.author.as_ref(),
            &self.resources_dir_name,
            &self.resources_zip_name,
        )?;

        for path in &self.paths {
            fs.mount(path, true);
        }

        for zipfile_bytes in self.memory_zip_files {
            fs.add_zip_file(std::io::Cursor::new(zipfile_bytes))?;
        }

        let config = if self.load_conf_file {
            fs.read_config().unwrap_or(self.conf)
        } else {
            self.conf
        };

        Context::from_conf(config, fs)
    }
}
