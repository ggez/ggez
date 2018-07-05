//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text.
//!
//! This module also manages graphics state, coordinate systems, etc.
//! The default coordinate system has the origin in the upper-left
//! corner of the screen, with Y increasing downwards.

use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::u16;

use gfx;
use gfx::texture;
use gfx::Device;
use gfx::Factory;
use gfx_device_gl;
use glutin::{self, GlContext};

use conf;
use conf::WindowMode;
use context::Context;
use context::DebugId;
use GameError;
use GameResult;

mod canvas;
mod context;
mod drawparam;
mod image;
mod mesh;
mod shader;
mod text;
mod types;
use mint;
use nalgebra as na;

pub mod spritebatch;

pub use self::canvas::*;
pub(crate) use self::context::*;
pub use self::drawparam::*;
pub use self::image::*;
pub use self::mesh::*;
pub use self::shader::*;
pub use self::text::*;
pub use self::types::*;

type BuggoSurfaceFormat = gfx::format::Srgba8;
type ShaderResourceType = [f32;4];

/// A marker trait saying that something is a label for a particular backend,
/// with associated gfx-rs types for that backend.
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
    fn raw_to_typed_shader_resource(&self,
        texture_view: gfx::handle::RawShaderResourceView<Self::Resources>,
    ) -> gfx::handle::ShaderResourceView<
        <Self as BackendSpec>::Resources,
        ShaderResourceType,
    > {
        let typed_view: gfx::handle::ShaderResourceView<
            _,
            ShaderResourceType,
        > = gfx::memory::Typed::new(texture_view);
        typed_view
    }


    /// Returns the version of the backend, `(major, minor)`.
    /// 
    /// So for instance if the backend is using OpenGL version 3.2,
    /// it would return `(3, 2)`.
    fn version_tuple(&self) -> (u8, u8);

    /// Returns a string containing some backend-dependent info.
    fn get_info(&self, device: &Self::Device) -> String;

    /// Creates the window.
    fn init<'a>(&self, window_builder: glutin::WindowBuilder, gl_builder: glutin::ContextBuilder<'a>, events_loop: &glutin::EventsLoop,
    color_format: gfx::format::Format, depth_format: gfx::format::Format) -> (
        glutin::GlWindow, 
        Self::Device, 
        Self::Factory, 
        gfx::handle::RawRenderTargetView<Self::Resources>, 
        gfx::handle::RawDepthStencilView<Self::Resources>
    );

    /// Create an Encoder for the backend.
    fn get_encoder(factory: &mut Self::Factory) -> gfx::Encoder<
            Self::Resources,
            Self::CommandBuffer,
        >;

    /// Resizes the viewport for the backend. (right now assumes a Glutin window...)
    fn resize_viewport(&self, color_view: &gfx::handle::RawRenderTargetView<Self::Resources>, depth_view: &gfx::handle::RawDepthStencilView<Self::Resources>,
    color_format: gfx::format::Format, depth_format: gfx::format::Format,
    window: &glutin::GlWindow)  ->
        Option<(gfx::handle::RawRenderTargetView<Self::Resources>, 
        gfx::handle::RawDepthStencilView<Self::Resources>)>;
}

/// A backend specification for OpenGL.
/// This is different from `conf::Backend` because
/// this needs to be its own struct to implement traits upon,
/// and because there may need to be a layer of translation
/// between what the user specifies in the config, and what the
/// graphics backend init code actually gets.
///
/// You shouldn't normally need to fiddle with this directly
/// but it has to be exported cause generic types like
/// `Shader` depend on it.
#[derive(Debug, Copy, Clone, PartialEq, Eq, SmartDefault, Hash)]
pub struct GlBackendSpec {
    #[default = r#"3"#]
    major: u8,
    #[default = r#"2"#]
    minor: u8,
}

impl From<conf::Backend> for GlBackendSpec {
    fn from(c: conf::Backend) -> Self {
        match c {
            conf::Backend::OpenGL { major, minor } => Self { 
                major, 
                minor,
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

    fn init<'a>(&self, window_builder: glutin::WindowBuilder, gl_builder: glutin::ContextBuilder<'a>, events_loop: &glutin::EventsLoop,
    color_format: gfx::format::Format, depth_format: gfx::format::Format) -> (
        glutin::GlWindow, 
        Self::Device, 
        Self::Factory, 
        gfx::handle::RawRenderTargetView<Self::Resources>, 
        gfx::handle::RawDepthStencilView<Self::Resources>
    ) {
        use gfx_window_glutin;
        let (window, device, factory, screen_render_target, depth_view) =
            gfx_window_glutin::init_raw(
                window_builder,
                gl_builder,
                events_loop,
                color_format,
                depth_format,
            );
        (window, device, factory, screen_render_target, depth_view)
    }

    fn get_info(&self, device: &Self::Device) -> String {
        let info = device.get_info();
        format!(
            "  Driver vendor: {}, renderer {}, version {:?}, shading language {:?}",
            info.platform_name.vendor,
            info.platform_name.renderer,
            info.version,
            info.shading_language
        )
    }

    fn get_encoder(factory: &mut Self::Factory) -> gfx::Encoder<
            Self::Resources,
            Self::CommandBuffer,
        >  {
       factory.create_command_buffer().into()
    }


    fn resize_viewport(&self, color_view: &gfx::handle::RawRenderTargetView<Self::Resources>, depth_view: &gfx::handle::RawDepthStencilView<Self::Resources>,
    color_format: gfx::format::Format, depth_format: gfx::format::Format,
    window: &glutin::GlWindow) ->
        Option<(gfx::handle::RawRenderTargetView<Self::Resources>, 
        gfx::handle::RawDepthStencilView<Self::Resources>)> {
        // Basically taken from the definition of
        // gfx_window_glutin::update_views()
        let dim = color_view.get_dimensions();
        assert_eq!(dim, depth_view.get_dimensions());
        use gfx_window_glutin;
        if let Some((cv, dv)) = gfx_window_glutin::update_views_raw(
            window,
            dim,
            color_format,
            depth_format,
        ) {
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
    },
    Vertex {
        pos: [1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0],
        uv: [0.0, 1.0],
    },
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

gfx_defines!{
    /// Internal structure containing vertex data.
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    /// Internal structure containing values that are different for each
    /// drawable object.
    vertex InstanceProperties {
        // the columns here are for the transform matrix;
        // you can't shove a full matrix into an instance
        // buffer, annoyingly.
        src: [f32; 4] = "a_Src",
        col1: [f32; 4] = "a_TCol1",
        col2: [f32; 4] = "a_TCol2",
        col3: [f32; 4] = "a_TCol3",
        col4: [f32; 4] = "a_TCol4",
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
        // out: gfx::RawRenderTarget = "Target0",
    }
}

impl Default for InstanceProperties {
    fn default() -> Self {
        InstanceProperties {
            src: [0.0, 0.0, 1.0, 1.0],
            col1: [1.0, 0.0, 0.0, 0.0],
            col2: [0.0, 1.0, 0.0, 0.0],
            col3: [1.0, 0.0, 1.0, 0.0],
            col4: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// A structure for conveniently storing Sampler's, based off
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
        let sampler = self.samplers
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
/// TODO: Into<Color> ?
pub fn clear(ctx: &mut Context, color: Color) {
    let gfx = &mut ctx.gfx_context;
    // SRGB BUGGO: Only convert when drawing on srgb surface
    let linear_color: types::LinearColor = color.into();
    // SRGB BUGGO: Need a clear_raw() method here, which I don't think
    // gfx-rs has.  So for now we wing it.
    type ColorFormat = BuggoSurfaceFormat;
    let typed_render_target: gfx::handle::RenderTargetView<_, ColorFormat> =
        gfx::memory::Typed::new(gfx.data.out.clone());
    gfx.encoder.clear(&typed_render_target, linear_color.into());
}

/// Draws the given `Drawable` object to the screen by calling its
/// `draw()` method.
pub fn draw<D, T>(ctx: &mut Context, drawable: &D, params: T) -> GameResult
where
    D: Drawable,
    T: Into<DrawTransform>,
{
    let params = params.into();
    drawable.draw(ctx, DrawTransform::from(params))
}


/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your `EventHandler`'s `draw()` method.
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
/// (screen or selected canvas) to a PNG file.
pub fn screenshot(ctx: &mut Context) -> GameResult<Image> {
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
    let target_texture = gfx.factory
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
    let shader_resource = gfx.factory
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

/* // TODO: consider implementing.
// Draw an arc.
// Punting on this until later.
pub fn arc(_ctx: &mut Context,
           _mode: DrawMode,
           _point: Point,
           _radius: f32,
           _angle1: f32,
           _angle2: f32,
           _segments: u32)
           -> GameResult {
    unimplemented!();
}
*/

// TODO: Make all of these take Into<Color>???

/// Draw a circle.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
///
/// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
pub fn circle<P>(
    ctx: &mut Context,
    color: Color,
    mode: DrawMode,
    point: P,
    radius: f32,
    tolerance: f32,
) -> GameResult
where
    P: Into<mint::Point2<f32>>,
{
    let m = Mesh::new_circle(ctx, mode, point, radius, tolerance)?;
    m.draw(ctx, DrawParam::new().color(color))
}

/// Draw an ellipse.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
///
/// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
pub fn ellipse<P>(
    ctx: &mut Context,
    color: Color,
    mode: DrawMode,
    point: P,
    radius1: f32,
    radius2: f32,
    tolerance: f32,
) -> GameResult
where
    P: Into<mint::Point2<f32>>,
{
    let m = Mesh::new_ellipse(ctx, mode, point, radius1, radius2, tolerance)?;
    m.draw(ctx, DrawParam::new().color(color))
}

/// Draws a line of one or more connected segments.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn line<P>(ctx: &mut Context, color: Color, points: &[P], width: f32) -> GameResult
where
    P: Into<mint::Point2<f32>> + Clone,
{
    let m = Mesh::new_line(ctx, points, width)?;
    m.draw(ctx, DrawParam::new().color(color))
}

/// Draws points (as rectangles)
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn points<P>(ctx: &mut Context, color: Color, points: &[P], point_size: f32) -> GameResult
where
    P: Into<mint::Point2<f32>> + Clone,
{
    let points = points.into_iter().cloned().map(P::into);
    for p in points {
        let r = Rect::new(p.x, p.y, point_size, point_size);
        rectangle(ctx, color, DrawMode::Fill, r)?;
    }
    Ok(())
}

/// Draws a closed polygon
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn polygon<P>(ctx: &mut Context, color: Color, mode: DrawMode, vertices: &[P]) -> GameResult
where
    P: Into<mint::Point2<f32>> + Clone,
{
    let m = Mesh::new_polygon(ctx, mode, vertices)?;
    m.draw(ctx, DrawParam::new().color(color))
}

// TODO: consider removing - it's commented out on devel.
// Renders text with the default font.
// Not terribly efficient as it re-renders the text with each call,
// but good enough for debugging.
// Doesn't actually work, double-borrow on ctx.  Bah.
// pub fn print(ctx: &mut Context, dest: Point, text: &str) -> GameResult {
//     let rendered_text = {
//         let font = &ctx.default_font;
//         text::Text::new(ctx, text, font)?
//     };
//     draw(ctx, &rendered_text, dest, 0.0)
// }

/// Draws a rectangle.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn rectangle(ctx: &mut Context, color: Color, mode: DrawMode, rect: Rect) -> GameResult {
    let x1 = rect.x;
    let x2 = rect.x + rect.w;
    let y1 = rect.y;
    let y2 = rect.y + rect.h;
    let pts = [
        Point2::new(x1, y1),
        Point2::new(x2, y1),
        Point2::new(x2, y2),
        Point2::new(x1, y2),
    ];
    polygon(ctx, color, mode, &pts)
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Get the default filter mode for new images.
pub fn get_default_filter(ctx: &Context) -> FilterMode {
    let gfx = &ctx.gfx_context;
    gfx.default_sampler_info.filter.into()
}

// TODO: consider putting more stuff here;
// actual GL version will likely require new glutin features.
// ALSO TODO: Query actual GL profile stuff
/// Returns a string that tells a little about the obtained rendering mode.
/// It is supposed to be human-readable and will change; do not try to parse
/// information out of it!
pub fn get_renderer_info(ctx: &Context) -> GameResult<String> {
    Ok(format!(
        "Requested GL {}.{} Core profile with sRGB {}, actually got GL ?.? ? profile with sRGB {:?}.",
        ctx.gfx_context.backend_spec.major, ctx.gfx_context.backend_spec.minor,
        ctx.gfx_context.is_srgb(), ctx.gfx_context.color_format()
    ))
}

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: left, y: top, w: width, h: height }`
///
/// If the Y axis increases downwards, the `height` of the Rect
/// will be negative.
pub fn get_screen_coordinates(ctx: &Context) -> Rect {
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
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}

/// Sets the raw projection matrix to the given homogeneous
/// transformation matrix.
///
/// You must call `apply_transformations(ctx)` after calling this to apply
/// these changes and recalculate the underlying MVP matrix.
pub fn set_projection(context: &mut Context, proj: Matrix4) {
    let gfx = &mut context.gfx_context;
    gfx.set_projection(proj);
}

/// Premultiplies the given transformation matrix with the current projection matrix
///
/// You must call `apply_transformations(ctx)` after calling this to apply
/// these changes and recalculate the underlying MVP matrix.
pub fn transform_projection(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    let curr = gfx.get_projection();
    gfx.set_projection(transform * curr);
}

/// Gets a copy of the context's raw projection matrix
pub fn get_projection(context: &Context) -> Matrix4 {
    let gfx = &context.gfx_context;
    gfx.get_projection()
}

/// Pushes a homogeneous transform matrix to the top of the transform
/// (model) matrix stack of the `Context`. If no matrix is given, then
/// pushes a copy of the current transform matrix to the top of the stack.
///
/// You must call `apply_transformations(ctx)` after calling this to apply
/// these changes and recalculate the underlying MVP matrix.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn push_transform(context: &mut Context, transform: Option<Matrix4>) {
    let gfx = &mut context.gfx_context;
    if let Some(t) = transform {
        gfx.push_transform(t);
    } else {
        let copy = *gfx.modelview_stack
            .last()
            .expect("Matrix stack empty, should never happen");
        gfx.push_transform(copy);
    }
}

/// Pops the transform matrix off the top of the transform
/// (model) matrix stack of the `Context`.
///
/// You must call `apply_transformations(ctx)` after calling this to apply
/// these changes and recalculate the underlying MVP matrix.
pub fn pop_transform(context: &mut Context) {
    let gfx = &mut context.gfx_context;
    gfx.pop_transform();
}

/// Sets the current model transformation to the given homogeneous
/// transformation matrix.
///
/// You must call `apply_transformations(ctx)` after calling this to apply
/// these changes and recalculate the underlying MVP matrix.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn set_transform(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    gfx.set_transform(transform);
}

/// Gets a copy of the context's current transform matrix
pub fn get_transform(context: &Context) -> Matrix4 {
    let gfx = &context.gfx_context;
    gfx.get_transform()
}

/// Premultiplies the given transform with the current model transform.
///
/// You must call `apply_transformations(ctx)` after calling this to apply
/// these changes and recalculate the underlying MVP matrix.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn transform(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    let curr = gfx.get_transform();
    gfx.set_transform(transform * curr);
}

/// Sets the current model transform to the origin transform (no transformation)
///
/// You must call `apply_transformations(ctx)` after calling this to apply
/// these changes and recalculate the underlying MVP matrix.
pub fn origin(context: &mut Context) {
    let gfx = &mut context.gfx_context;
    gfx.set_transform(Matrix4::identity());
}

/// Calculates the new total transformation (Model-View-Projection) matrix
/// based on the matrices at the top of the transform and view matrix stacks
/// and sends it to the graphics card.
pub fn apply_transformations(context: &mut Context) -> GameResult {
    let gfx = &mut context.gfx_context;
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}

/// Sets the blend mode of the currently active shader program
pub fn set_blend_mode(ctx: &mut Context, mode: BlendMode) -> GameResult {
    ctx.gfx_context.set_blend_mode(mode)
}

/// Sets the window mode, such as the size and other properties.
///
/// Setting the window mode may have side effects, such as clearing
/// the screen or setting the screen coordinates viewport to some undefined value.
/// It is recommended to call `set_screen_coordinates()` after changing the window
/// size to make sure everything is what you want it to be.
pub fn set_mode(context: &mut Context, mode: WindowMode) -> GameResult {
    let gfx = &mut context.gfx_context;
    gfx.set_window_mode(mode)
}

/// Sets the window to fullscreen or back.
pub fn set_fullscreen(context: &mut Context, fullscreen: conf::FullscreenType) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.fullscreen_type = fullscreen;
    set_mode(context, window_mode)
}

/// Sets the window resolution based on the specified width and height.
pub fn set_resolution(context: &mut Context, width: f32, height: f32) -> GameResult {
    let mut window_mode = context.conf.window_mode;
    window_mode.width = width;
    window_mode.height = height;
    set_mode(context, window_mode)
}

use std::path::Path;
use winit::Icon;
/// Sets the window icon.
pub fn set_window_icon<P: AsRef<Path>>(context: &Context, path: Option<P>) -> GameResult<()> {
    let icon = match path {
        Some(path) => Some(Icon::from_path(path)?),
        None => None,
    };
    context.gfx_context.window.set_window_icon(icon);
    Ok(())
}

/// Sets the window title.
pub fn set_window_title(context: &Context, title: &str) {
    context.gfx_context.window.set_title(title);
}

/// Returns a reference to the SDL window.
/// Ideally you should not need to use this because ggez
/// would provide all the functions you need without having
/// to dip into SDL itself.  But life isn't always ideal.
pub fn get_window(context: &Context) -> &glutin::Window {
    let gfx = &context.gfx_context;
    &gfx.window
}

/// Returns the size of the window in pixels as (width, height),
/// including borders, titlebar, etc.
/// Returns zeros if window doesn't exist.
/// TODO: Rename, since get_drawable_size is usually what we
/// actually want
pub fn get_size(context: &Context) -> (f64, f64) {
    let gfx = &context.gfx_context;
    gfx.window.get_outer_size()
        .map(|logical_size| (logical_size.width, logical_size.height))
        .unwrap_or((0.0, 0.0))
}

/// Returns the size of the window's underlying drawable in pixels as (width, height).
/// Returns zeros if window doesn't exist.
pub fn get_drawable_size(context: &Context) -> (f64, f64) {
    let gfx = &context.gfx_context;
    gfx.window.get_inner_size()
        .map(|logical_size| (logical_size.width, logical_size.height))
        .unwrap_or((0.0, 0.0))
}

/// Returns the gfx-rs `Factory` object for ggez's rendering context.
pub fn get_factory(context: &mut Context) -> &mut gfx_device_gl::Factory {
    let gfx = &mut context.gfx_context;
    &mut gfx.factory
}

/// Returns the gfx-rs `Device` object for ggez's rendering context.
pub fn get_device(context: &mut Context) -> &mut gfx_device_gl::Device {
    let gfx = &mut context.gfx_context;
    gfx.device.as_mut()
}

/// Returns the gfx-rs `Encoder` object for ggez's rendering context.
pub fn get_encoder(
    context: &mut Context,
) -> &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> {
    let gfx = &mut context.gfx_context;
    &mut gfx.encoder
}

/// Returns the gfx-rs depth target object for ggez's rendering context.
pub fn get_depth_view(
    context: &mut Context,
) -> gfx::handle::RawDepthStencilView<gfx_device_gl::Resources> {
    let gfx = &mut context.gfx_context;
    gfx.depth_view.clone()
}

/// Returns the gfx-rs color target object for ggez's rendering context.
pub fn get_screen_render_target(
    context: &Context,
) -> gfx::handle::RawRenderTargetView<gfx_device_gl::Resources> {
    let gfx = &context.gfx_context;
    gfx.data.out.clone()
}

/// Returns raw `gfx-rs` state objects, if you want to use `gfx-rs` to write
/// your own graphics pipeline then this gets you the interfaces you need
/// to do so.
/// Returns all the relevant objects at once;
/// getting them one by one is awkward 'cause it tends to create double-borrows
/// on the Context object.
pub fn get_gfx_objects(
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
    ///
    /// ALSO TODO: Expand docs
    fn draw<D>(&self, ctx: &mut Context, param: D) -> GameResult
    where
        D: Into<DrawTransform>;

    /// Sets the blend mode to be used when drawing this drawable.
    /// This overrides the general `graphics::set_blend_mode()`.
    /// If `None` is set, defers to the blend mode set by
    /// `graphics::set_blend_mode()`.
    fn set_blend_mode(&mut self, mode: Option<BlendMode>);

    /// Gets the blend mode to be used when drawing this drawable.
    fn get_blend_mode(&self) -> Option<BlendMode>;
}

#[cfg(test)]
mod tests {}
