use std::fmt;
/// We re-export winit so it's easy for people to use the same version as we are
/// without having to mess around figuring it out.
pub use winit;

use crate::audio;
use crate::conf;
use crate::error::GameResult;
use crate::event::winit_event;
use crate::filesystem::Filesystem;
use crate::graphics::{self, Point2};
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
    pub(crate) gfx_context: crate::graphics::context::GraphicsContext,
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

    /// Context-specific unique ID.
    /// Compiles to nothing in release mode, and so
    /// vanishes; meanwhile we get dead-code warnings.
    #[allow(dead_code)]
    debug_id: DebugId,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Context: {:p}>", self)
    }
}

impl Context {
    /// Tries to create a new Context using settings from the given [`Conf`](../conf/struct.Conf.html) object.
    /// Usually called by [`ContextBuilder::build()`](struct.ContextBuilder.html#method.build).
    fn from_conf(conf: conf::Conf, mut fs: Filesystem) -> GameResult<(Context, winit::EventsLoop)> {
        let debug_id = DebugId::new();
        let audio_context: Box<dyn audio::AudioContext> = if conf.modules.audio {
            Box::new(audio::RodioAudioContext::new()?)
        } else {
            Box::new(audio::NullAudioContext::default())
        };
        let events_loop = winit::EventsLoop::new();
        let timer_context = timer::TimeContext::new();
        let backend_spec = graphics::GlBackendSpec::from(conf.backend);
        let graphics_context = graphics::context::GraphicsContext::new(
            &mut fs,
            &events_loop,
            &conf.window_setup,
            conf.window_mode,
            backend_spec,
            debug_id,
        )?;
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

            debug_id,
        };

        Ok((ctx, events_loop))
    }

    // TODO LATER: This should be a function in `ggez::event`, per the
    // "functions are stable, methods and fields are unstable" promise
    // given above.
    /// Feeds an `Event` into the `Context` so it can update any internal
    /// state it needs to, such as detecting window resizes.  If you are
    /// rolling your own event loop, you should call this on the events
    /// you receive before processing them yourself.
    pub fn process_event(&mut self, event: &winit::Event) {
        match event.clone() {
            winit_event::Event::WindowEvent { event, .. } => match event {
                winit_event::WindowEvent::Resized(logical_size) => {
                    let hidpi_factor = self.gfx_context.window.window().get_hidpi_factor();
                    let physical_size = logical_size.to_physical(hidpi_factor as f64);
                    self.gfx_context.window.resize(physical_size);
                    self.gfx_context.resize_viewport();
                }
                winit_event::WindowEvent::CursorMoved {
                    position: logical_position,
                    ..
                } => {
                    self.mouse_context.set_last_position(Point2::new(
                        logical_position.x as f32,
                        logical_position.y as f32,
                    ));
                }
                winit_event::WindowEvent::MouseInput { button, state, .. } => {
                    let pressed = match state {
                        winit_event::ElementState::Pressed => true,
                        winit_event::ElementState::Released => false,
                    };
                    self.mouse_context.set_button(button, pressed);
                }
                winit_event::WindowEvent::KeyboardInput {
                    input:
                        winit::KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            modifiers,
                            ..
                        },
                    ..
                } => {
                    let pressed = match state {
                        winit_event::ElementState::Pressed => true,
                        winit_event::ElementState::Released => false,
                    };
                    self.keyboard_context
                        .set_modifiers(keyboard::KeyMods::from(modifiers));
                    self.keyboard_context.set_key(keycode, pressed);
                }
                winit_event::WindowEvent::HiDpiFactorChanged(_) => {
                    // Nope.
                }
                _ => (),
            },
            winit_event::Event::DeviceEvent { event, .. } => {
                if let winit_event::DeviceEvent::MouseMotion { delta: (x, y) } = event {
                    self.mouse_context
                        .set_last_delta(Point2::new(x as f32, y as f32));
                }
            }

            _ => (),
        };
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
    /// [`backend()`](#method.backend).
    pub fn conf(mut self, conf: conf::Conf) -> Self {
        self.conf = conf;
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
    pub fn build(self) -> GameResult<(Context, winit::EventsLoop)> {
        let mut fs = Filesystem::new(self.game_id.as_ref(), self.author.as_ref())?;

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

#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(debug_assertions)]
static DEBUG_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// This is a type that contains a unique ID for each `Context` and
/// is contained in each thing created from the `Context` which
/// becomes invalid when the `Context` goes away (for example, `Image` because
/// it contains texture handles).  When compiling without assertions
/// (in release mode) it is replaced with a zero-size type, compiles
/// down to nothing, disappears entirely with a puff of optimization logic.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(debug_assertions)]
pub(crate) struct DebugId(u32);
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(not(debug_assertions))]
pub(crate) struct DebugId;

#[cfg(debug_assertions)]
impl DebugId {
    pub fn new() -> Self {
        let id = DEBUG_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as u32;
        // fetch_add() wraps on overflow so we check for overflow explicitly.
        // JUST IN CASE YOU TRY TO CREATE 2^32 CONTEXTS IN ONE PROGRAM!  muahahahahaaa
        assert!(DEBUG_ID_COUNTER.load(Ordering::SeqCst) as u32 > id);
        DebugId(id)
    }

    pub fn get(ctx: &Context) -> Self {
        DebugId(ctx.debug_id.0)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn assert(&self, ctx: &Context) {
        if *self != ctx.debug_id {
            panic!("Tried to use a resource with a Context that did not create it; this should never happen!");
        }
    }
}

#[cfg(not(debug_assertions))]
impl DebugId {
    pub fn new() -> Self {
        DebugId
    }

    pub fn get(_ctx: &Context) -> Self {
        DebugId
    }

    pub fn assert(&self, _ctx: &Context) {
        // Do nothing.
    }
}
