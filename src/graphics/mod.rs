//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the [`Drawable`](trait.Drawable.html) trait.  It also handles
//! basic loading of images and text.
//!
//! This module also manages graphics state, coordinate systems, etc.
//! The default coordinate system has the origin in the upper-left
//! corner of the screen, with Y increasing downwards.
//!
//! This library differs significantly in performance characteristics from the
//! `LÖVE` library that it is based on. Many operations that are batched by default
//! in love (e.g. drawing primitives like rectangles or circles) are *not* batched
//! in `ggez`, so render loops with a large number of draw calls can be very slow.
//! The primary solution to efficiently rendering a large number of primitives is
//! a [`SpriteBatch`](spritebatch/struct.SpriteBatch.html), which can be orders
//! of magnitude more efficient than individual
//! draw calls.

// use std::collections::HashMap;
use std::convert::From;
// use std::fmt;
// use std::path::Path;
// use std::u16;

use ggraphics as gg;
use glutin;

use crate::conf;
use crate::conf::WindowMode;
use crate::context::Context;
//use crate::context::DebugId;
// use crate::GameError;
use crate::GameResult;

pub(crate) mod drawparam;
//pub(crate) mod mesh;
pub(crate) mod image;
pub(crate) mod types;

pub use crate::graphics::drawparam::*;
pub use crate::graphics::image::*;
//pub use crate::graphics::mesh::*;
pub use crate::graphics::types::*;

pub type WindowCtx = glutin::WindowedContext<glutin::PossiblyCurrent>;

#[derive(Debug)]
pub struct GraphicsContext {
    // TODO: OMG these names
    pub ctx: gg::GlContext,
    pub(crate) win_ctx: WindowCtx,
    pub(crate) screen_pass: gg::RenderPass,
}

impl GraphicsContext {
    pub(crate) fn new(gl: gg::glow::Context, win_ctx: WindowCtx) -> Self {
        let mut ctx = gg::GlContext::new(gl);
        unsafe {
            let (w, h): (u32, u32) = win_ctx.window().inner_size().into();
            let screen_pass = gg::RenderPass::new_screen(
                &mut ctx,
                w as usize,
                h as usize,
                Some((0.1, 0.2, 0.3, 1.0)),
            );
            Self {
                ctx,
                win_ctx,
                screen_pass,
            }
        }
    }

    /// Sets window mode from a WindowMode object.
    pub(crate) fn set_window_mode(&mut self, mode: WindowMode) -> GameResult {
        //use crate::conf::FullscreenType;
        // use glutin::dpi;
        let window = self.win_ctx.window();

        window.set_maximized(mode.maximized);

        // TODO: Min and max dimension constraints have gone away,
        // remove them from WindowMode

        //let monitors = window.available_monitors();
        // TODO: Okay, how we set fullscreen stuff has changed
        // and this needs to be totally revamped.
        /*
        match (mode.fullscreen_type, monitors.last()) {
            (FullscreenType::Windowed, _) => {
                window.set_fullscreen(None);
                window.set_decorations(!mode.borderless);
                window.set_inner_size(dpi::LogicalSize {
                    width: f64::from(mode.width),
                    height: f64::from(mode.height),
                });
                window.set_resizable(mode.resizable);
            }
            (FullscreenType::True, Some(monitor)) => {
                window.set_fullscreen(Some(monitor));
                window.set_inner_size(dpi::LogicalSize {
                    width: f64::from(mode.width),
                    height: f64::from(mode.height),
                });
            }
            (FullscreenType::Desktop, Some(monitor)) => {
                let position = monitor.get_position();
                let dimensions = monitor.get_dimensions();
                let hidpi_factor = window.get_hidpi_factor();
                window.set_fullscreen(None);
                window.set_decorations(false);
                window.set_inner_size(dimensions.to_logical(hidpi_factor));
                window.set_position(position.to_logical(hidpi_factor));
            }
            _ => panic!("Unable to detect monitor; if you are on Linux Wayland it may be this bug: https://github.com/rust-windowing/winit/issues/793"),
        }
        */
        Ok(())
    }
}

pub trait WindowTrait {
    fn request_redraw(&self);
    fn swap_buffers(&self);
    fn resize_viewport(&self);
}

/// Used for desktop
#[cfg(not(target_arch = "wasm32"))]
impl WindowTrait for glutin::WindowedContext<glutin::PossiblyCurrent> {
    fn request_redraw(&self) {
        self.window().request_redraw();
    }
    fn swap_buffers(&self) {
        self.swap_buffers().unwrap();
    }
    // TODO
    fn resize_viewport(&self) {}
}

/// Used for wasm
#[cfg(target_arch = "wasm32")]
impl WindowTrait for winit::window::Window {
    fn request_redraw(&self) {
        self.request_redraw();
    }
    fn swap_buffers(&self) {
        /*
        let msg = format!("swapped buffers");
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&msg));
        */
    }
    // TODO
    fn resize_viewport(&self) {}
}

/*
pub(crate) mod canvas;
pub(crate) mod context;
pub(crate) mod image;
pub(crate) mod shader;
pub(crate) mod text;
pub(crate) mod types;

pub use mint;

pub mod spritebatch;

pub use crate::graphics::canvas::*;
pub use crate::graphics::image::*;
pub use crate::graphics::shader::*;
pub use crate::graphics::text::*;
pub use crate::graphics::types::*;

// This isn't really particularly nice, but it's only used
// in a couple places and it's not very easy to change or configure.
// Since the next major project is "rewrite the graphics engine" I think
// we're fine just leaving it.
//
// It exists basically because gfx-rs is incomplete and we can't *always*
// specify texture formats and such entirely at runtime, which we need to
// do to make sRGB handling work properly.
pub(crate) type BuggoSurfaceFormat = gfx::format::Srgba8;
type ShaderResourceType = [f32; 4];

/// A trait providing methods for working with a particular backend, such as OpenGL,
/// with associated gfx-rs types for that backend.  As a user you probably
/// don't need to touch this unless you want to write a new graphics backend
/// for ggez.  (Trust me, you don't.)
pub trait BackendSpec: fmt::Debug {
    /// gfx resource type
    type Resources: gfx::Resources;
    /// gfx factory type
    type Factory: gfx::Factory<Self::Resources> + Clone;
    /// gfx command buffer type
    type CommandBuffer: gfx::CommandBuffer<Self::Resources>;
    /// gfx device type
    type Device: gfx::Device<Resources = Self::Resources, CommandBuffer = Self::CommandBuffer>;

    /// A helper function to take a RawShaderResourceView and turn it into a typed one based on
    /// the surface type defined in a `BackendSpec`.
    ///
    /// But right now we only allow surfaces that use [f32;4] colors, so we can freely
    /// hardcode this in the `ShaderResourceType` type.
    fn raw_to_typed_shader_resource(
        &self,
        texture_view: gfx::handle::RawShaderResourceView<Self::Resources>,
    ) -> gfx::handle::ShaderResourceView<<Self as BackendSpec>::Resources, ShaderResourceType> {
        // gfx::memory::Typed is UNDOCUMENTED, aiee!
        // However there doesn't seem to be an official way to turn a raw tex/view into a typed
        // one; this API oversight would probably get fixed, except gfx is moving to a new
        // API model.  So, that also fortunately means that undocumented features like this
        // probably won't go away on pre-ll gfx...
        let typed_view: gfx::handle::ShaderResourceView<_, ShaderResourceType> =
            gfx::memory::Typed::new(texture_view);
        typed_view
    }

    /// Helper function that turns a raw to typed texture.
    /// A bit hacky since we can't really specify surface formats as part
    /// of this that well, alas.  There's some functions, like
    /// `gfx::Encoder::update_texture()`, that don't seem to have a `_raw()`
    /// counterpart, so we need this, so we need `BuggoSurfaceFormat` to
    /// keep fixed at compile time what texture format we're actually using.
    /// Oh well!
    fn raw_to_typed_texture(
        &self,
        texture_view: gfx::handle::RawTexture<Self::Resources>,
    ) -> gfx::handle::Texture<
        <Self as BackendSpec>::Resources,
        <BuggoSurfaceFormat as gfx::format::Formatted>::Surface,
    > {
        let typed_view: gfx::handle::Texture<
            _,
            <BuggoSurfaceFormat as gfx::format::Formatted>::Surface,
        > = gfx::memory::Typed::new(texture_view);
        typed_view
    }

    /// Returns the version of the backend, `(major, minor)`.
    ///
    /// So for instance if the backend is using OpenGL version 3.2,
    /// it would return `(3, 2)`.
    fn version_tuple(&self) -> (u8, u8);

    /// Returns the glutin `Api` enum for this backend.
    fn api(&self) -> glutin::Api;

    /// Returns the text of the vertex and fragment shader files
    /// to create default shaders for this backend.
    fn shaders(&self) -> (&'static [u8], &'static [u8]);

    /// Returns a string containing some backend-dependent info.
    fn info(&self, device: &Self::Device) -> String;

    /// Creates the window.
    fn init<'a>(
        &self,
        window_builder: glutin::WindowBuilder,
        gl_builder: glutin::ContextBuilder<'a>,
        events_loop: &glutin::EventsLoop,
        color_format: gfx::format::Format,
        depth_format: gfx::format::Format,
    ) -> Result<
        (
            glutin::WindowedContext,
            Self::Device,
            Self::Factory,
            gfx::handle::RawRenderTargetView<Self::Resources>,
            gfx::handle::RawDepthStencilView<Self::Resources>,
        ),
        glutin::CreationError,
    >;

    /// Create an Encoder for the backend.
    fn encoder(factory: &mut Self::Factory) -> gfx::Encoder<Self::Resources, Self::CommandBuffer>;

    /// Resizes the viewport for the backend. (right now assumes a Glutin window...)
    fn resize_viewport(
        &self,
        color_view: &gfx::handle::RawRenderTargetView<Self::Resources>,
        depth_view: &gfx::handle::RawDepthStencilView<Self::Resources>,
        color_format: gfx::format::Format,
        depth_format: gfx::format::Format,
        window: &glutin::WindowedContext,
    ) -> Option<(
        gfx::handle::RawRenderTargetView<Self::Resources>,
        gfx::handle::RawDepthStencilView<Self::Resources>,
    )>;
}

/// A backend specification for OpenGL.
/// This is different from [`Backend`](../conf/enum.Backend.html)
/// because this needs to be its own struct to implement traits
/// upon, and because there may need to be a layer of translation
/// between what the user asks for in the config, and what the
/// graphics backend code actually gets from the driver.
///
/// You shouldn't normally need to fiddle with this directly
/// but it has to be public because generic types like
/// [`Shader`](type.Shader.html) depend on it.
#[derive(Debug, Copy, Clone, PartialEq, Eq, SmartDefault)]
pub struct GlBackendSpec {
    #[default = 3]
    major: u8,
    #[default = 2]
    minor: u8,
    #[default(glutin::Api::OpenGl)]
    api: glutin::Api,
}

impl From<conf::Backend> for GlBackendSpec {
    fn from(c: conf::Backend) -> Self {
        match c {
            conf::Backend::OpenGL { major, minor } => Self {
                major,
                minor,
                api: glutin::Api::OpenGl,
            },
            conf::Backend::OpenGLES { major, minor } => Self {
                major,
                minor,
                api: glutin::Api::OpenGlEs,
            },
        }
    }
}

impl BackendSpec for GlBackendSpec {
    type Resources = gfx_device_gl::Resources;
    type Factory = gfx_device_gl::Factory;
    type CommandBuffer = gfx_device_gl::CommandBuffer;
    type Device = gfx_device_gl::Device;

    fn version_tuple(&self) -> (u8, u8) {
        (self.major, self.minor)
    }

    fn api(&self) -> glutin::Api {
        self.api
    }

    fn shaders(&self) -> (&'static [u8], &'static [u8]) {
        match self.api {
            glutin::Api::OpenGl => (
                include_bytes!("shader/basic_150.glslv"),
                include_bytes!("shader/basic_150.glslf"),
            ),
            glutin::Api::OpenGlEs => (
                include_bytes!("shader/basic_es300.glslv"),
                include_bytes!("shader/basic_es300.glslf"),
            ),
            a => panic!("Unsupported API: {:?}, should never happen", a),
        }
    }

    fn init<'a>(
        &self,
        window_builder: glutin::WindowBuilder,
        gl_builder: glutin::ContextBuilder<'a>,
        events_loop: &glutin::EventsLoop,
        color_format: gfx::format::Format,
        depth_format: gfx::format::Format,
    ) -> Result<
        (
            glutin::WindowedContext,
            Self::Device,
            Self::Factory,
            gfx::handle::RawRenderTargetView<Self::Resources>,
            gfx::handle::RawDepthStencilView<Self::Resources>,
        ),
        glutin::CreationError,
    > {
        gfx_window_glutin::init_raw(
            window_builder,
            gl_builder,
            events_loop,
            color_format,
            depth_format,
        )
    }

    fn info(&self, device: &Self::Device) -> String {
        let info = device.get_info();
        format!(
            "Driver vendor: {}, renderer {}, version {:?}, shading language {:?}",
            info.platform_name.vendor,
            info.platform_name.renderer,
            info.version,
            info.shading_language
        )
    }

    fn encoder(factory: &mut Self::Factory) -> gfx::Encoder<Self::Resources, Self::CommandBuffer> {
        factory.create_command_buffer().into()
    }

    fn resize_viewport(
        &self,
        color_view: &gfx::handle::RawRenderTargetView<Self::Resources>,
        depth_view: &gfx::handle::RawDepthStencilView<Self::Resources>,
        color_format: gfx::format::Format,
        depth_format: gfx::format::Format,
        window: &glutin::WindowedContext,
    ) -> Option<(
        gfx::handle::RawRenderTargetView<Self::Resources>,
        gfx::handle::RawDepthStencilView<Self::Resources>,
    )> {
        // Basically taken from the definition of
        // gfx_window_glutin::update_views()
        let dim = color_view.get_dimensions();
        assert_eq!(dim, depth_view.get_dimensions());
        if let Some((cv, dv)) =
            gfx_window_glutin::update_views_raw(window, dim, color_format, depth_format)
        {
            Some((cv, dv))
        } else {
            None
        }
    }
}

const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        pos: [0.0, 0.0],
        uv: [0.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 0.0],
        uv: [1.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0],
        uv: [0.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

gfx_defines! {
    /// Structure containing fundamental vertex data.
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32;4] = "a_VertColor",
    }

    /// Internal structure containing values that are different for each
    /// drawable object.  This is the per-object data that
    /// gets fed into the shaders.
    vertex InstanceProperties {
        // the columns here are for the transform matrix;
        // you can't shove a full matrix into an instance
        // buffer, annoyingly.
        col1: [f32; 4] = "a_TCol1",
        col2: [f32; 4] = "a_TCol2",
        col3: [f32; 4] = "a_TCol3",
        col4: [f32; 4] = "a_TCol4",
        src: [f32; 4] = "a_Src",
        color: [f32; 4] = "a_Color",
    }

    /// Internal structure containing global shader state.
    constant Globals {
        mvp_matrix: [[f32; 4]; 4] = "u_MVP",
    }

    // Internal structure containing graphics pipeline state.
    // This can't be a doc comment though because it somehow
    // breaks the gfx_defines! macro though.  :-(
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        globals: gfx::ConstantBuffer<Globals> = "Globals",
        rect_instance_properties: gfx::InstanceBuffer<InstanceProperties> = (),
        // The default values here are overwritten by the
        // pipeline init values in `shader::create_shader()`.
        out: gfx::RawRenderTarget =
          ("Target0",
           gfx::format::Format(gfx::format::SurfaceType::R8_G8_B8_A8, gfx::format::ChannelType::Srgb),
           gfx::state::ColorMask::all(), Some(gfx::preset::blend::ALPHA)
          ),
    }
}

impl fmt::Display for InstanceProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let matrix = eu::Transform3D::from_column_slice(&matrix_vec);
        let matrix: eu::Transform3D<f32, (), ()> = eu::Transform3D::column_major(
            self.col1[0],
            self.col1[1],
            self.col1[2],
            self.col1[3],
            self.col2[0],
            self.col2[1],
            self.col2[2],
            self.col2[3],
            self.col3[0],
            self.col3[1],
            self.col3[2],
            self.col3[3],
            self.col4[0],
            self.col4[1],
            self.col4[2],
            self.col4[3],
        );
        writeln!(
            f,
            "Src: ({},{}+{},{})",
            self.src[0], self.src[1], self.src[2], self.src[3]
        )?;
        writeln!(f, "Color: {:?}", self.color)?;
        write!(f, "Matrix: {:?}", matrix)
    }
}

impl Default for InstanceProperties {
    fn default() -> Self {
        InstanceProperties {
            col1: [1.0, 0.0, 0.0, 0.0],
            col2: [0.0, 1.0, 0.0, 0.0],
            col3: [1.0, 0.0, 1.0, 0.0],
            col4: [1.0, 0.0, 0.0, 1.0],
            src: [0.0, 0.0, 1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}
/// A structure for conveniently storing `Sampler`'s, based off
/// their `SamplerInfo`.
pub(crate) struct SamplerCache<B>
where
    B: BackendSpec,
{
    samplers: HashMap<texture::SamplerInfo, gfx::handle::Sampler<B::Resources>>,
}

impl<B> SamplerCache<B>
where
    B: BackendSpec,
{
    fn new() -> Self {
        SamplerCache {
            samplers: HashMap::new(),
        }
    }

    fn get_or_insert(
        &mut self,
        info: texture::SamplerInfo,
        factory: &mut B::Factory,
    ) -> gfx::handle::Sampler<B::Resources> {
        let sampler = self
            .samplers
            .entry(info)
            .or_insert_with(|| factory.create_sampler(info));
        sampler.clone()
    }
}

impl From<gfx::buffer::CreationError> for GameError {
    fn from(e: gfx::buffer::CreationError) -> Self {
        use gfx::buffer::CreationError;
        match e {
            CreationError::UnsupportedBind(b) => GameError::RenderError(format!(
                "Could not create buffer: Unsupported Bind ({:?})",
                b
            )),
            CreationError::UnsupportedUsage(u) => GameError::RenderError(format!(
                "Could not create buffer: Unsupported Usage ({:?})",
                u
            )),
            CreationError::Other => {
                GameError::RenderError("Could not create buffer: Unknown error".to_owned())
            }
        }
    }
}
*/
// **********************************************************************
// DRAWING
// **********************************************************************

#[derive(Debug)]
pub struct ScreenRenderPass<'a> {
    pub(crate) ctx: &'a mut GraphicsContext,
}

impl<'a> ScreenRenderPass<'a> {
    /// Calls the given thunk with a pipeline
    pub fn quad_pipeline<F>(&mut self, shader: gg::Shader, mut f: F)
    where
        F: FnMut(&mut dyn gg::Pipeline),
    {
        unsafe {
            let mut pipeline = ggraphics::QuadPipeline::new(&self.ctx.ctx, shader);
            /*
            let dc = pipeline.new_drawcall(gl, particle_texture, ggraphics::SamplerSpec::default());
            dc.add(ggraphics::QuadData::empty());
            */
            f(&mut pipeline);
            self.ctx.screen_pass.add_pipeline(pipeline);
        }
    }

    /// Creates and returns a new quad pipeline
    pub fn add_quad_pipeline(&mut self, shader: gg::Shader) -> &mut dyn gg::Pipeline {
        unsafe {
            let pipeline = ggraphics::QuadPipeline::new(&self.ctx.ctx, shader);
            self.ctx.screen_pass.add_pipeline(pipeline);
            &mut **self
                .ctx
                .screen_pass
                .pipelines
                .last_mut()
                .expect("Should never happen")
        }
    }
}

/// Returns the final render pass that will draw to the screen.
pub fn screen_render_pass(ctx: &mut Context) -> ScreenRenderPass {
    let s = ScreenRenderPass {
        ctx: &mut ctx.gfx_context,
    };
    s
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your [`EventHandler`](../event/trait.EventHandler.html)'s
/// [`draw()`](../event/trait.EventHandler.html#tymethod.draw) method.
///
/// Unsets any active canvas.
pub fn present(ctx: &mut Context) -> GameResult<()> {
    ctx.gfx_context.ctx.draw();
    ctx.gfx_context.win_ctx.swap_buffers()?;
    Ok(())
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Returns a string that tells a little about the obtained rendering mode.
/// It is supposed to be human-readable and will change; do not try to parse
/// information out of it!
pub fn renderer_info(ctx: &Context) -> GameResult<String> {
    let (vend, rend, vers, shader_vers) = ctx.gfx_context.ctx.get_info();
    Ok(format!(
        "GL context created info:
  Vendor: {}
  Renderer: {}
  Version: {}
  Shader version: {}",
        vend, rend, vers, shader_vers
    ))
}

/// Sets the window mode, such as the size and other properties.
///
/// Setting the window mode may have side effects, such as clearing
/// the screen or setting the screen coordinates viewport to some
/// undefined value (for example, the window was resized).  It is
/// recommended to call
/// [`set_screen_coordinates()`](fn.set_screen_coordinates.html) after
/// changing the window size to make sure everything is what you want
/// it to be.
pub fn set_mode(context: &mut Context, mode: WindowMode) -> GameResult {
    let gfx = &mut context.gfx_context;
    gfx.set_window_mode(mode)?;
    // Save updated mode.
    context.conf.window_mode = mode;
    Ok(())
}

// TODO
// /// Sets the window icon.
// pub fn set_window_icon<P: AsRef<Path>>(context: &mut Context, path: Option<P>) -> GameResult<()> {
//     let icon = match path {
//         Some(p) => {
//             let p: &Path = p.as_ref();
//             Some(context::load_icon(p, &mut context.filesystem)?)
//         }
//         None => None,
//     };
//     context.gfx_context.window.set_window_icon(icon);
//     Ok(())
// }

/// Sets the window title.
pub fn set_window_title(context: &Context, title: &str) {
    context.gfx_context.win_ctx.window().set_title(title);
}

/// Returns the size of the window in pixels as (width, height),
/// including borders, titlebar, etc.
/// Returns zeros if the window doesn't exist.
pub fn size(context: &Context) -> (f32, f32) {
    let gfx = &context.gfx_context;
    let size = gfx.win_ctx.window().outer_size();
    (size.width as f32, size.height as f32)
}

/// Returns the size of the window's underlying drawable in pixels as (width, height).
/// Returns zeros if window doesn't exist.
pub fn drawable_size(context: &Context) -> (f32, f32) {
    let gfx = &context.gfx_context;
    let size = gfx.win_ctx.window().inner_size();
    (size.width as f32, size.height as f32)
}

/// Sets the window to fullscreen or back.
pub fn set_fullscreen(context: &mut Context, fullscreen: conf::FullscreenType) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.fullscreen_type = fullscreen;
    set_mode(context, window_mode)
}

/// Sets the window size/resolution to the specified width and height.
pub fn set_drawable_size(context: &mut Context, width: f32, height: f32) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.width = width;
    window_mode.height = height;
    set_mode(context, window_mode)
}

/// Sets whether or not the window is resizable.
pub fn set_resizable(context: &mut Context, resizable: bool) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.resizable = resizable;
    set_mode(context, window_mode)
}

/// Returns a reference to the Glutin window.
/// Ideally you should not need to use this because ggez
/// would provide all the functions you need without having
/// to dip into Glutin itself.  But life isn't always ideal.
pub fn window(context: &Context) -> &glutin::window::Window {
    let gfx = &context.gfx_context;
    &gfx.win_ctx.window()
}

/// TODO: This is roughb ut works
pub fn gl_context(context: &Context) -> &gg::GlContext {
    &context.gfx_context.ctx
}

/// TODO: This is roughb ut works
pub fn gl_context_mut(context: &mut Context) -> &mut gg::GlContext {
    &mut context.gfx_context.ctx
}

/*


/// Take a screenshot by outputting the current render surface
/// (screen or selected canvas) to an `Image`.
pub fn screenshot(ctx: &mut Context) -> GameResult<Image> {
    // TODO LATER: This makes the screenshot upside-down form some reason...
    // Probably because all our images are upside down, for coordinate reasons!
    // How can we fix it?
    use gfx::memory::Bind;
    let debug_id = DebugId::get(ctx);

    let gfx = &mut ctx.gfx_context;
    let (w, h, _depth, aa) = gfx.data.out.get_dimensions();
    let surface_format = gfx.color_format();
    let gfx::format::Format(surface_type, channel_type) = surface_format;

    let texture_kind = gfx::texture::Kind::D2(w, h, aa);
    let texture_info = gfx::texture::Info {
        kind: texture_kind,
        levels: 1,
        format: surface_type,
        bind: Bind::TRANSFER_SRC | Bind::TRANSFER_DST | Bind::SHADER_RESOURCE,
        usage: gfx::memory::Usage::Data,
    };
    let target_texture = gfx
        .factory
        .create_texture_raw(texture_info, Some(channel_type), None)?;
    let image_info = gfx::texture::ImageInfoCommon {
        xoffset: 0,
        yoffset: 0,
        zoffset: 0,
        width: w,
        height: h,
        depth: 0,
        format: surface_format,
        mipmap: 0,
    };

    let mut local_encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> =
        gfx.factory.create_command_buffer().into();

    local_encoder.copy_texture_to_texture_raw(
        gfx.data.out.get_texture(),
        None,
        image_info,
        &target_texture,
        None,
        image_info,
    )?;

    local_encoder.flush(&mut *gfx.device);

    let resource_desc = gfx::texture::ResourceDesc {
        channel: channel_type,
        layer: None,
        min: 0,
        max: 0,
        swizzle: gfx::format::Swizzle::new(),
    };
    let shader_resource = gfx
        .factory
        .view_texture_as_shader_resource_raw(&target_texture, resource_desc)?;
    let image = Image {
        texture: shader_resource,
        texture_handle: target_texture,
        sampler_info: gfx.default_sampler_info,
        blend_mode: None,
        width: w,
        height: h,
        debug_id,
    };

    Ok(image)
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: left, y: top, w: width, h: height }`
///
/// If the Y axis increases downwards, the `height` of the `Rect`
/// will be negative.
pub fn screen_coordinates(ctx: &Context) -> Rect {
    ctx.gfx_context.screen_rect
}

/// Sets the bounds of the screen viewport.
///
/// The default coordinate system has (0,0) at the top-left corner
/// with X increasing to the right and Y increasing down, with the
/// viewport scaled such that one coordinate unit is one pixel on the
/// screen.  This function lets you change this coordinate system to
/// be whatever you prefer.
///
/// The `Rect`'s x and y will define the top-left corner of the screen,
/// and that plus its w and h will define the bottom-right corner.
pub fn set_screen_coordinates(context: &mut Context, rect: Rect) -> GameResult {
    let gfx = &mut context.gfx_context;
    gfx.set_projection_rect(rect);
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}



/// Applies `DrawParam` to `Rect`.
pub fn transform_rect(rect: Rect, param: DrawParam) -> Rect {
    let w = param.src.w * param.scale.x * rect.w;
    let h = param.src.h * param.scale.y * rect.h;
    let offset_x = w * param.offset.x;
    let offset_y = h * param.offset.y;
    let dest_x = param.dest.x - offset_x;
    let dest_y = param.dest.y - offset_y;
    let mut r = Rect {
        w,
        h,
        x: dest_x + rect.x * param.scale.x,
        y: dest_y + rect.y * param.scale.y,
    };
    r.rotate(param.rotation);
    r
}

#[cfg(test)]
mod tests {
    use crate::graphics::{transform_rect, DrawParam, Rect};
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn headless_test_transform_rect() {
        {
            let r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new();
            let real = transform_rect(r, param);
            let expected = r;
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new().scale([0.5, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -0.5,
                y: -0.5,
                w: 1.0,
                h: 0.5,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new().offset([0.5, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -1.5,
                y: -1.5,
                w: 1.0,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: 1.0,
                y: 0.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new().rotation(PI * 0.5);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -1.0,
                y: 1.0,
                w: 1.0,
                h: 2.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .scale([0.5, 0.5])
                .offset([0.0, 1.0])
                .rotation(PI * 0.5);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: 0.5,
                y: -0.5,
                w: 0.5,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
    }
}
*/
