use std::fmt;
/// We re-export winit so it's easy for people to use the same version as we are
/// without having to mess around figuring it out.
pub use winit;

use crate::audio;
use crate::conf;
use crate::error::GameResult;
use crate::filesystem::Filesystem;
use crate::graphics;
use crate::input::{gamepad, keyboard, mouse};
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
/// drawing things, playing sounds, or loading resources (which then
/// need to be transformed into a format the hardware likes) will need
/// to access the `Context`.  It is an error to create some type that
/// relies upon a `Context`, such as `Image`, and then drop the `Context`
/// and try to draw the old `Image` with the new `Context`.  Most types
/// include checks to make this panic in debug mode, but it's not perfect.
///
/// All fields in this struct are basically undocumented features,
/// only here to make it easier to debug, or to let advanced users
/// hook into the guts of ggez and make it do things it normally
/// can't.  Most users shouldn't need to touch these things directly,
/// since implementation details may change without warning.  The
/// public and stable API is `ggez`'s module-level functions and
/// types.
pub struct Context {
    /// Filesystem state
    pub filesystem: Filesystem,
    /// Graphics state
    pub gfx_context: crate::graphics::context::GraphicsContext,
    /// Timer state
    pub timer_context: timer::TimeContext,
    /// Audio context
    pub audio_context: Box<dyn audio::AudioContext>,
    /// Keyboard context
    pub keyboard_context: keyboard::KeyboardContext,
    /// Mouse context
    pub mouse_context: mouse::MouseContext,
    /// Gamepad context
    pub gamepad_context: Box<dyn gamepad::GamepadContext>,

    /// The Conf object the Context was created with.
    /// It's here just so that we can see the original settings,
    /// updating it will have no effect.
    pub(crate) conf: conf::Conf,
    /// Controls whether or not the event loop should be running.
    /// Set this with `ggez::event::quit()`.
    pub continuing: bool,
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
        let audio_context: Box<dyn audio::AudioContext> = if conf.modules.audio {
            Box::new(audio::RodioAudioContext::new()?)
        } else {
            Box::new(audio::NullAudioContext::default())
        };
        let events_loop = winit::event_loop::EventLoop::new();
        let timer_context = timer::TimeContext::new();
        let graphics_context = graphics::context::GraphicsContext::new(&events_loop, &conf)?;
        let mouse_context = mouse::MouseContext::new();
        let keyboard_context = keyboard::KeyboardContext::new();
        let gamepad_context: Box<dyn gamepad::GamepadContext> = if conf.modules.gamepad {
            Box::new(gamepad::GilrsGamepadContext::new()?)
        } else {
            Box::new(gamepad::NullGamepadContext::default())
        };

        let ctx = Context {
            conf,
            filesystem: fs,
            gfx_context: graphics_context,
            continuing: true,
            timer_context,
            audio_context,
            keyboard_context,
            gamepad_context,
            mouse_context,
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
    pub fn window_setup(mut self, setup: conf::WindowSetup) -> Self {
        self.conf.window_setup = setup;
        self
    }

    /// Sets the window mode settings.
    pub fn window_mode(mut self, mode: conf::WindowMode) -> Self {
        self.conf.window_mode = mode;
        self
    }

    /// Sets the graphics backend.
    pub fn backend(mut self, backend: conf::Backend) -> Self {
        self.conf.backend = backend;
        self
    }

    /// Sets the modules configuration.
    pub fn modules(mut self, modules: conf::ModuleConf) -> Self {
        self.conf.modules = modules;
        self
    }

    /// Sets all the config options, overriding any previous
    /// ones from [`window_setup()`](#method.window_setup),
    /// [`window_mode()`](#method.window_mode), and
    /// [`backend()`](#method.backend).  These are used as
    /// defaults and are overridden by any external config
    /// file found.
    pub fn default_conf(mut self, conf: conf::Conf) -> Self {
        self.conf = conf;
        self
    }

    /// Sets resources dir name.
    /// Default resources dir name is `resources`.
    pub fn resources_dir_name(mut self, new_name: impl ToString) -> Self {
        self.resources_dir_name = new_name.to_string();
        self
    }

    /// Sets resources zip name.
    /// Default resources dir name is `resources.zip`.
    pub fn resources_zip_name(mut self, new_name: impl ToString) -> Self {
        self.resources_zip_name = new_name.to_string();
        self
    }

    /// Add a new read-only filesystem path to the places to search
    /// for resources.
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
