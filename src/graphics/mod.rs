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

#![allow(unsafe_code)]
use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::path::Path;
use std::u16;

use gfx::texture;
use gfx::Device;
use gfx::Factory;
use old_school_gfx_glutin_ext::*;

use crate::conf;
use crate::conf::WindowMode;
use crate::context::Context;
use crate::GameError;
use crate::GameResult;

pub(crate) mod canvas;
pub(crate) mod context;
pub(crate) mod drawparam;
pub(crate) mod image;
pub(crate) mod mesh;
pub(crate) mod shader;
pub(crate) mod text;
pub(crate) mod types;

pub use mint;

pub mod spritebatch;

pub use crate::graphics::canvas::*;
pub use crate::graphics::drawparam::*;
pub use crate::graphics::image::*;
pub use crate::graphics::mesh::*;
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

type BackendSpecInitResult<Device, Factory, Resources> = Result<
    (
        glutin::WindowedContext<glutin::PossiblyCurrent>,
        Device,
        Factory,
        gfx::handle::RawRenderTargetView<Resources>,
        gfx::handle::RawDepthStencilView<Resources>,
    ),
    glutin::CreationError,
>;

type MainTargetView<Resources> = Option<(
    gfx::handle::RawRenderTargetView<Resources>,
    gfx::handle::RawDepthStencilView<Resources>,
)>;

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
    /// But right now we only allow surfaces that use \[f32;4\] colors, so we can freely
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
    fn shaders(&self) -> (&'static [u8], &'static [u8], &'static [u8]);

    /// Returns a string containing some backend-dependent info.
    fn info(&self, device: &Self::Device) -> String;

    /// Creates the window.
    fn init<'a>(
        &self,
        window_builder: glutin::window::WindowBuilder,
        gl_builder: glutin::ContextBuilder<'a, glutin::NotCurrent>,
        events_loop: &glutin::event_loop::EventLoop<()>,
        color_format: gfx::format::Format,
        depth_format: gfx::format::Format,
    ) -> BackendSpecInitResult<Self::Device, Self::Factory, Self::Resources>;
    /// Create an Encoder for the backend.
    fn encoder(factory: &mut Self::Factory) -> gfx::Encoder<Self::Resources, Self::CommandBuffer>;

    /// Resizes the viewport for the backend. (right now assumes a Glutin window...)
    fn resize_viewport(
        &self,
        color_view: &gfx::handle::RawRenderTargetView<Self::Resources>,
        depth_view: &gfx::handle::RawDepthStencilView<Self::Resources>,
        color_format: gfx::format::Format,
        depth_format: gfx::format::Format,
        window: &glutin::WindowedContext<glutin::PossiblyCurrent>,
    ) -> MainTargetView<Self::Resources>;
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

    fn shaders(&self) -> (&'static [u8], &'static [u8], &'static [u8]) {
        match self.api {
            glutin::Api::OpenGl => (
                include_bytes!("shader/basic_150.glslv"),
                include_bytes!("shader/basic_150.glslf"),
                include_bytes!("shader/resolve_150.glslf"),
            ),
            glutin::Api::OpenGlEs => (
                include_bytes!("shader/basic_es300.glslv"),
                include_bytes!("shader/basic_es300.glslf"),
                include_bytes!("shader/resolve_es300.glslf"),
            ),
            a => panic!("Unsupported API: {:?}, should never happen", a),
        }
    }

    fn init<'a>(
        &self,
        window_builder: glutin::window::WindowBuilder,
        gl_builder: glutin::ContextBuilder<'a, glutin::NotCurrent>,
        events_loop: &glutin::event_loop::EventLoop<()>,
        color_format: gfx::format::Format,
        depth_format: gfx::format::Format,
    ) -> BackendSpecInitResult<Self::Device, Self::Factory, Self::Resources> {
        gl_builder
            .with_gfx_color_raw(color_format)
            .with_gfx_depth_raw(depth_format)
            .build_windowed(window_builder, events_loop)
            .map(|i| i.init_gfx_raw(color_format, depth_format))
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
        window: &glutin::WindowedContext<glutin::PossiblyCurrent>,
    ) -> MainTargetView<Self::Resources> {
        // Basically taken from the definition of
        // gfx_window_glutin::update_views()
        let dim = color_view.get_dimensions();
        assert_eq!(dim, depth_view.get_dimensions());
        window.updated_views_raw(dim, color_format, depth_format)
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
}

pub(crate) mod pipe {
    use super::{Globals, InstanceProperties, Vertex};

    gfx_pipeline_inner! {
        vbuf: gfx::VertexBuffer<Vertex>,
        tex: gfx::TextureSampler<[f32; 4]>,
        globals: gfx::ConstantBuffer<Globals>,
        rect_instance_properties: gfx::InstanceBuffer<InstanceProperties>,
        out: gfx::RawRenderTarget,
    }

    pub fn new() -> Init<'static> {
        Init {
            vbuf: (),
            tex: "t_Texture",
            globals: "Globals",
            rect_instance_properties: (),
            out: (
                "Target0",
                gfx::format::Format(
                    gfx::format::SurfaceType::R8_G8_B8_A8,
                    gfx::format::ChannelType::Srgb,
                ),
                gfx::state::ColorMask::all(),
                Some(gfx::preset::blend::ALPHA),
            ),
        }
    }
}

impl fmt::Display for InstanceProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let matrix = Matrix4::from_cols_array_2d(&[self.col1, self.col2, self.col3, self.col4]);
        writeln!(
            f,
            "Src: ({},{}+{},{})",
            self.src[0], self.src[1], self.src[2], self.src[3]
        )?;
        writeln!(f, "Color: {:?}", self.color)?;
        write!(f, "Matrix: {}", matrix)
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

// **********************************************************************
// DRAWING
// **********************************************************************

/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context, color: Color) {
    let gfx = &mut ctx.gfx_context;
    let linear_color: types::LinearColor = color.into();
    let c: [f32; 4] = linear_color.into();
    gfx.encoder.clear_raw(&gfx.data.out, c.into());
}

/// Draws the given `Drawable` object to the screen by calling its
/// [`draw()`](trait.Drawable.html#tymethod.draw) method.
pub fn draw<D, T>(ctx: &mut Context, drawable: &D, params: T) -> GameResult
where
    D: Drawable,
    T: Into<DrawParam>,
{
    let params = params.into();
    drawable.draw(ctx, params)
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your [`EventHandler`](../event/trait.EventHandler.html)'s
/// [`draw()`](../event/trait.EventHandler.html#tymethod.draw) method.
///
/// Unsets any active canvas.
pub fn present(ctx: &mut Context) -> GameResult<()> {
    let gfx = &mut ctx.gfx_context;
    gfx.data.out = gfx.screen_render_target.clone();
    // We might want to give the user more control over when the
    // encoder gets flushed eventually, if we want them to be able
    // to do their own gfx drawing.  HOWEVER, the whole pipeline type
    // thing is a bigger hurdle, so this is fine for now.
    gfx.encoder.flush(&mut *gfx.device);
    gfx.window.swap_buffers()?;
    gfx.device.cleanup();
    Ok(())
}

/// Take a screenshot by outputting the current render surface
/// (screen or selected canvas) to an `Image`.
pub fn screenshot(ctx: &mut Context) -> GameResult<Image> {
    use gfx::memory::Typed;
    use gfx::traits::FactoryExt;

    let gfx = &mut ctx.gfx_context;
    let (w, h, _depth, aa) = gfx.data.out.get_dimensions();
    if aa != gfx_core::texture::AaMode::Single {
        // Details see https://github.com/ggez/ggez/issues/751
        return Err(GameError::RenderError("Can't take screenshots of anti aliased textures.\n(since neither copying or resolving them is supported right now)".to_string()));
    }

    let surface_format = gfx.color_format();

    let dl_buffer = &mut gfx.to_rgba8_buffer;
    // check if it's big enough and recreate it if not
    let size_needed = usize::from(w) * usize::from(h) * 4;
    if dl_buffer.len() != size_needed {
        *dl_buffer = gfx.factory.create_download_buffer::<u8>(size_needed)?;
    }

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

    let encoder = &mut gfx.encoder;

    encoder.copy_texture_to_buffer_raw(
        gfx.data.out.get_texture(),
        None,
        image_info,
        dl_buffer.raw(),
        0,
    )?;

    encoder.flush(&mut *gfx.device);

    let mut data = gfx.factory.read_mapping(dl_buffer)?.to_vec();
    flip_pixel_data(&mut data, w as usize, h as usize);
    let image = Image::from_rgba8(ctx, w, h, &data)?;

    Ok(image)
}

/// Fast non-allocating function for flipping pixel data in an image vertically
fn flip_pixel_data(rgba: &mut Vec<u8>, width: usize, height: usize) {
    // cast the buffer into u32 so we can easily access the pixels themselves
    // splits the pixel buffer into an upper (first) and a lower (second) half
    let pixels: (&mut [u32], &mut [u32]) =
        bytemuck::cast_slice_mut(rgba.as_mut_slice()).split_at_mut(width * height / 2);

    // When the image has an uneven height, it will split the buffer in the middle of a row.
    // This will decrease pixel count so that the x,y in the loop will never enter the split row since
    // for uneven height images the middle row will stay the same anyway
    let pixel_count = if height % 2 == 0 {
        width * height / 2
    } else {
        width * height / 2 - width / 2
    };
    // Even though we removed uwidth / 2 from pixel_count,
    // the second half of the buffer's size will still contain that data so
    // we need to offset the index on that by the size of said data
    let second_set_offset = if height % 2 == 0 {
        // even height (evenness on width doesn't matter)
        0
    } else if width % 2 == 0 {
        // uneven height but even width
        width / 2
    } else {
        // uneven height and uneven width
        width / 2 + 1
    };
    for i in 0..pixel_count {
        let x = i % width;
        let y = i / width;
        let reverse_y = height / 2 - y - 1;

        let idx = (y * width) + x;
        let second_idx = (reverse_y * width) + x + second_set_offset;

        std::mem::swap(&mut pixels.0[idx], &mut pixels.1[second_idx]);
    }
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Get the default filter mode for new images.
pub fn default_filter(ctx: &Context) -> FilterMode {
    let gfx = &ctx.gfx_context;
    gfx.default_sampler_info.filter.into()
}

/// Returns a string that tells a little about the obtained rendering mode.
/// It is supposed to be human-readable and will change; do not try to parse
/// information out of it!
pub fn renderer_info(ctx: &Context) -> GameResult<String> {
    let backend_info = ctx.gfx_context.backend_spec.info(&*ctx.gfx_context.device);
    Ok(format!(
        "Requested {:?} {}.{} Core profile, actually got {}.",
        ctx.gfx_context.backend_spec.api,
        ctx.gfx_context.backend_spec.major,
        ctx.gfx_context.backend_spec.minor,
        backend_info
    ))
}

/// Returns the screen color format used by the context
pub fn get_window_color_format(ctx: &Context) -> gfx::format::Format {
    ctx.gfx_context.color_format()
}

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: left, y: top, w: width, h: height }`
///
/// If the Y axis increases downwards, the `height` of the `Rect`
/// will be negative.
pub fn screen_coordinates(ctx: &Context) -> Rect {
    ctx.gfx_context.screen_rect
}

/// Sets the default filter mode used to scale images.
///
/// This does not apply retroactively to already created images.
pub fn set_default_filter(ctx: &mut Context, mode: FilterMode) {
    let gfx = &mut ctx.gfx_context;
    let new_mode = mode.into();
    let sampler_info = texture::SamplerInfo::new(new_mode, texture::WrapMode::Clamp);
    // We create the sampler now so we don't end up creating it at some
    // random-ass time while we're trying to draw stuff.
    let _sampler = gfx.samplers.get_or_insert(sampler_info, &mut *gfx.factory);
    gfx.default_sampler_info = sampler_info;
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
    gfx.set_global_mvp(Matrix4::IDENTITY)
}

/// Premultiplies the given transformation matrix with the current projection matrix
///
/// You must call [`apply_transformations(ctx)`](fn.apply_transformations.html)
/// after calling this to apply these changes and recalculate the
/// underlying MVP matrix.
pub fn mul_projection<M>(context: &mut Context, transform: M)
where
    M: Into<mint::ColumnMatrix4<f32>>,
{
    let transform = Matrix4::from(transform.into());
    let gfx = &mut context.gfx_context;
    let curr = gfx.projection();
    gfx.set_projection(transform * curr);
}

/// Sets the raw projection matrix to the given homogeneous
/// transformation matrix.  For an introduction to graphics matrices,
/// a good source is this: <http://ncase.me/matrix/>
///
/// You must call [`apply_transformations(ctx)`](fn.apply_transformations.html)
/// after calling this to apply these changes and recalculate the
/// underlying MVP matrix.
pub fn set_projection<M>(context: &mut Context, proj: M)
where
    M: Into<mint::ColumnMatrix4<f32>>,
{
    let proj = Matrix4::from(proj.into());
    let gfx = &mut context.gfx_context;
    gfx.set_projection(proj);
}

/// Gets a copy of the context's raw projection matrix
pub fn projection(context: &Context) -> mint::ColumnMatrix4<f32> {
    let gfx = &context.gfx_context;
    gfx.projection().into()
}

/// Sets the blend mode of the currently active shader program
pub fn set_blend_mode(ctx: &mut Context, mode: BlendMode) -> GameResult {
    ctx.gfx_context.set_blend_mode(mode)
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
pub fn set_mode(context: &mut Context, mut mode: WindowMode) -> GameResult {
    let gfx = &mut context.gfx_context;
    let result = gfx.set_window_mode(mode);
    if let Err(GameError::WindowError(_)) = result {
        // true fullscreen could not be set because the video mode matching the resolution is missing
        // so keep the old one
        mode.fullscreen_type = context.conf.window_mode.fullscreen_type;
    }
    // Save updated mode.
    context.conf.window_mode = mode;
    result
}

/// Sets the window to fullscreen or back.
pub fn set_fullscreen(context: &mut Context, fullscreen: conf::FullscreenType) -> GameResult {
    let window_mode = context.conf.window_mode.fullscreen_type(fullscreen);
    set_mode(context, window_mode)
}

/// Sets the window size (in physical pixels) / resolution to the specified width and height.
///
/// Note:   These dimensions are only interpreted as resolutions in true fullscreen mode.
///         If the selected resolution is not supported this function will return an Error.
pub fn set_drawable_size(context: &mut Context, width: f32, height: f32) -> GameResult {
    let window_mode = context.conf.window_mode.dimensions(width, height);
    set_mode(context, window_mode)
}

/// Sets whether or not the window is resizable.
pub fn set_resizable(context: &mut Context, resizable: bool) -> GameResult {
    let window_mode = context.conf.window_mode.resizable(resizable);
    set_mode(context, window_mode)
}

/// Sets the window icon.
pub fn set_window_icon<P: AsRef<Path>>(context: &mut Context, path: Option<P>) -> GameResult<()> {
    let icon = match path {
        Some(p) => {
            let p: &Path = p.as_ref();
            Some(context::load_icon(p, &mut context.filesystem)?)
        }
        None => None,
    };
    context.gfx_context.window.window().set_window_icon(icon);
    Ok(())
}

/// Sets the window title.
pub fn set_window_title(context: &Context, title: &str) {
    context.gfx_context.window.window().set_title(title);
}

/// Sets the window position.
pub fn set_window_position<P: Into<winit::dpi::Position>>(
    context: &Context,
    position: P,
) -> GameResult<()> {
    context
        .gfx_context
        .window
        .window()
        .set_outer_position(position);
    Ok(())
}

/// Gets the window position.
pub fn get_window_position(context: &Context) -> GameResult<winit::dpi::PhysicalPosition<i32>> {
    match context.gfx_context.window.window().outer_position() {
        Ok(position) => Ok(position),
        Err(e) => Err(GameError::WindowError(e.to_string())),
    }
}

/// Returns a reference to the Glutin window.
/// Ideally you should not need to use this because ggez
/// would provide all the functions you need without having
/// to dip into Glutin itself.  But life isn't always ideal.
pub fn window(context: &Context) -> &glutin::window::Window {
    let gfx = &context.gfx_context;
    gfx.window.window()
}

/// Returns an iterator providing all resolutions supported by the current monitor.
pub fn supported_resolutions(
    ctx: &crate::Context,
) -> impl Iterator<Item = winit::dpi::PhysicalSize<u32>> {
    let gfx = &ctx.gfx_context;
    let window = gfx.window.window();
    let monitor = window.current_monitor().unwrap();
    monitor.video_modes().map(|v_mode| v_mode.size())
}

/// Returns the size of the window in pixels as (width, height),
/// including borders, titlebar, etc.
/// Returns zeros if the window doesn't exist.
pub fn size(context: &Context) -> (f32, f32) {
    let gfx = &context.gfx_context;
    let window = gfx.window.window();
    let physical_size = window.outer_size();
    (physical_size.width as f32, physical_size.height as f32)
}

/// Returns the size of the window's underlying drawable in physical pixels as (width, height).
/// Returns zeros if window doesn't exist.
pub fn drawable_size(context: &Context) -> (f32, f32) {
    let gfx = &context.gfx_context;
    let window = gfx.window.window();
    let physical_size = window.inner_size();
    (physical_size.width as f32, physical_size.height as f32)
}

/// Return raw window context
pub fn window_raw(context: &mut Context) -> &mut glutin::WindowedContext<glutin::PossiblyCurrent> {
    &mut context.gfx_context.window
}

/// Deletes all cached font data.
///
/// Suggest this only gets used if you're sure you actually need it.
pub fn clear_font_cache(ctx: &mut Context) {
    use glyph_brush::GlyphBrushBuilder;
    use std::cell::RefCell;
    use std::rc::Rc;
    let font_vec =
        glyph_brush::ab_glyph::FontArc::try_from_slice(Font::default_font_bytes()).unwrap();
    let glyph_brush = GlyphBrushBuilder::using_font(font_vec).build();
    let (glyph_cache_width, glyph_cache_height) = glyph_brush.texture_dimensions();
    let initial_contents = vec![255; 4 * glyph_cache_width as usize * glyph_cache_height as usize];
    let glyph_cache = Image::from_rgba8(
        ctx,
        glyph_cache_width.try_into().unwrap(),
        glyph_cache_height.try_into().unwrap(),
        &initial_contents,
    )
    .unwrap();
    let glyph_state = Rc::new(RefCell::new(spritebatch::SpriteBatch::new(
        glyph_cache.clone(),
    )));
    ctx.gfx_context.glyph_brush = Rc::new(RefCell::new(glyph_brush));
    ctx.gfx_context.glyph_cache = glyph_cache;
    ctx.gfx_context.glyph_state = glyph_state;
}

#[allow(clippy::type_complexity)]
/// Returns raw `gfx-rs` state objects, if you want to use `gfx-rs` to write
/// your own graphics pipeline then this gets you the interfaces you need
/// to do so.
///
/// Returns all the relevant objects at once;
/// getting them one by one is awkward 'cause it tends to create double-borrows
/// on the Context object.
pub fn gfx_objects(
    context: &mut Context,
) -> (
    &mut <GlBackendSpec as BackendSpec>::Factory,
    &mut <GlBackendSpec as BackendSpec>::Device,
    &mut gfx::Encoder<
        <GlBackendSpec as BackendSpec>::Resources,
        <GlBackendSpec as BackendSpec>::CommandBuffer,
    >,
    gfx::handle::RawDepthStencilView<<GlBackendSpec as BackendSpec>::Resources>,
    gfx::handle::RawRenderTargetView<<GlBackendSpec as BackendSpec>::Resources>,
) {
    let gfx = &mut context.gfx_context;
    let f = &mut gfx.factory;
    let d = gfx.device.as_mut();
    let e = &mut gfx.encoder;
    let dv = gfx.depth_view.clone();
    let cv = gfx.data.out.clone();
    (f, d, e, dv, cv)
}

/// All types that can be drawn on the screen implement the `Drawable` trait.
pub trait Drawable {
    /// Draws the drawable onto the rendering target.
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult;

    /// Returns a bounding box in the form of a `Rect`.
    ///
    /// It returns `Option` because some `Drawable`s may have no bounding box
    /// (an empty `SpriteBatch` for example).
    fn dimensions(&self, ctx: &mut Context) -> Option<Rect>;

    /// Sets the blend mode to be used when drawing this drawable.
    /// This overrides the general [`graphics::set_blend_mode()`](fn.set_blend_mode.html).
    /// If `None` is set, defers to the blend mode set by
    /// `graphics::set_blend_mode()`.
    fn set_blend_mode(&mut self, mode: Option<BlendMode>);

    /// Gets the blend mode to be used when drawing this drawable.
    fn blend_mode(&self) -> Option<BlendMode>;
}

/// Applies `DrawParam` to `Rect`.
pub fn transform_rect(rect: Rect, param: DrawParam) -> Rect {
    match param.trans {
        Transform::Values {
            scale,
            offset,
            dest,
            rotation,
        } => {
            // first apply the offset
            let mut r = Rect {
                w: rect.w,
                h: rect.h,
                x: rect.x - offset.x * rect.w,
                y: rect.y - offset.y * rect.h,
            };
            // apply the scale
            let real_scale = (param.src.w * scale.x, param.src.h * scale.y);
            r.w = real_scale.0 * rect.w;
            r.h = real_scale.1 * rect.h;
            r.x *= real_scale.0;
            r.y *= real_scale.1;
            // apply the rotation
            r.rotate(rotation);
            // apply the destination translation
            r.x += dest.x;
            r.y += dest.y;

            r
        }
        Transform::Matrix(_m) => todo!("Fix me"),
    }
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
            let param = DrawParam::default();
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
                .rotation(PI * 0.5)
                .dest([1.0, 0.0]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: 1.5,
                y: -0.5,
                w: 0.5,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .offset([0.5, 0.5])
                .rotation(PI * 1.5)
                .dest([1.0, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: 0.5,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .offset([0.5, 0.5])
                .rotation(PI * 0.25)
                .scale([2.0, 1.0])
                .dest([1.0, 2.0]);
            let real = transform_rect(r, param);
            let sqrt = (2f32).sqrt() / 2.;
            let unit = sqrt + sqrt / 2.;
            let expected = Rect {
                x: -unit + 1.,
                y: -unit + 2.,
                w: 2. * unit,
                h: 2. * unit,
            };
            assert_relative_eq!(real, expected);
        }
    }
}
