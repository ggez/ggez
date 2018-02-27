//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text.
//!
//! This module also manages graphics state, coordinate systems, etc.
//! The default coordinate system has the origin in the upper-left
//! corner of the screen, with Y increasing downwards.

use std::fmt;
use std::convert::From;
use std::collections::HashMap;
use std::u16;

use sdl2;
use gfx;
use gfx_device_gl;
use gfx_window_sdl;
use gfx::texture;
use gfx::Device;
use gfx::Factory;

use conf;
use conf::WindowMode;
use context::Context;
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

/// A marker trait saying that something is a label for a particular backend,
/// with associated gfx-rs types for that backend.
pub trait BackendSpec: fmt::Debug {
    /// gfx resource type
    type Resources: gfx::Resources;
    /// gfx factory type
    type Factory: gfx::Factory<Self::Resources>;
    /// gfx command buffer type
    type CommandBuffer: gfx::CommandBuffer<Self::Resources>;
    /// gfx device type
    type Device: gfx::Device<Resources = Self::Resources, CommandBuffer = Self::CommandBuffer>;
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
    #[default = r#"3"#] major: u8,
    #[default = r#"2"#] minor: u8,
}

impl From<conf::Backend> for GlBackendSpec {
    fn from(c: conf::Backend) -> Self {
        match c {
            conf::Backend::OpenGL { major, minor } => Self {
                major: major,
                minor: minor,
            },
        }
    }
}

impl BackendSpec for GlBackendSpec {
    type Resources = gfx_device_gl::Resources;
    type Factory = gfx_device_gl::Factory;
    type CommandBuffer = gfx_device_gl::CommandBuffer;
    type Device = gfx_device_gl::Device;
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

type ColorFormat = gfx::format::Srgba8;
// I don't know why this gives a dead code warning
// since this type is definitely used... oh well.
#[allow(dead_code)]
type DepthFormat = gfx::format::DepthStencil;

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
        out: gfx::BlendTarget<ColorFormat> =
          ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
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

impl From<DrawParam> for InstanceProperties {
    fn from(p: DrawParam) -> Self {
        let mat: [[f32; 4]; 4] = p.into_matrix().into();
        let linear_color: types::LinearColor = p.color
            .expect("Converting DrawParam to InstanceProperties had None for a color; this should never happen!")
            .into();
        Self {
            src: p.src.into(),
            col1: mat[0],
            col2: mat[1],
            col3: mat[2],
            col4: mat[3],
            color: linear_color.into(),
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

/// This can probably be removed but might be
/// handy to keep around a bit longer.  Just in case something else
/// crazy happens.
#[allow(unused)]
fn test_opengl_versions(video: &sdl2::VideoSubsystem) {
    let mut major_versions = [4u8, 3u8, 2u8, 1u8];
    let minor_versions = [5u8, 4u8, 3u8, 2u8, 1u8, 0u8];
    major_versions.reverse();
    for major in &major_versions {
        for minor in &minor_versions {
            let gl = video.gl_attr();
            gl.set_context_version(*major, *minor);
            gl.set_context_profile(sdl2::video::GLProfile::Core);
            gl.set_red_size(5);
            gl.set_green_size(5);
            gl.set_blue_size(5);
            gl.set_alpha_size(8);

            print!("Requesting GL {}.{}... ", major, minor);
            let window_builder = video.window("so full of hate", 640, 480);
            let result = gfx_window_sdl::init::<ColorFormat, DepthFormat>(video, window_builder);
            match result {
                Ok(_) => println!(
                    "Ok, got GL {}.{}.",
                    gl.context_major_version(),
                    gl.context_minor_version()
                ),
                Err(res) => println!("Request failed: {:?}", res),
            }
        }
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
pub fn clear(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    let linear_color: types::LinearColor = gfx.background_color.into();
    gfx.encoder.clear(&gfx.data.out, linear_color.into());
}

/// Draws the given `Drawable` object to the screen by calling its
/// `draw()` method.
pub fn draw(ctx: &mut Context, drawable: &Drawable, dest: Point2, rotation: f32) -> GameResult<()> {
    drawable.draw(ctx, dest, rotation)
}

/// Draws the given `Drawable` object to the screen by calling its `draw_ex()` method.
pub fn draw_ex(ctx: &mut Context, drawable: &Drawable, params: DrawParam) -> GameResult<()> {
    drawable.draw_ex(ctx, params)
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your `EventHandler`'s `draw()` method.
///
/// Unsets any active canvas.
pub fn present(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    gfx.data.out = gfx.screen_render_target.clone();
    // We might want to give the user more control over when the
    // encoder gets flushed eventually, if we want them to be able
    // to do their own gfx drawing.  HOWEVER, the whole pipeline type
    // thing is a bigger hurdle, so this is fine for now.
    gfx.encoder.flush(&mut *gfx.device);
    gfx.window.gl_swap_window();
    gfx.device.cleanup();
}


/// Take a screenshot by outputting the current render surface
/// (screen or selected canvas) to a PNG file.
pub fn screenshot(ctx: &mut Context) -> GameResult<Image> {
    use gfx::memory::{Bind, Typed};
    use gfx::format::Formatted;

    let gfx = &mut ctx.gfx_context;
    let (w, h, _depth, aa) = gfx.data.out.get_dimensions();
    let surface_format = <ColorFormat as Formatted>::get_format();

    // TODO: The bind and data settings here might be worth
    // fiddling with...
    let texture_kind = 
        gfx::texture::Kind::D2(w, h, aa);
    // The format here is the same as is defined in ColorFormat
    let target_texture: gfx::handle::Texture<_, <ColorFormat as Formatted>::Surface> = gfx.factory.create_texture(
        texture_kind,
        1,
        Bind::TRANSFER_SRC | Bind::TRANSFER_DST | Bind::SHADER_RESOURCE,
        gfx::memory::Usage::Data,
        Some(gfx::format::ChannelType::Srgb)
    )?;
    
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

    let mut local_encoder: gfx::Encoder<
        gfx_device_gl::Resources,
        gfx_device_gl::CommandBuffer> = gfx.factory.create_command_buffer().into();
    
    local_encoder.copy_texture_to_texture_raw(
        gfx.data.out.raw().get_texture(),
        None,
        image_info,
        target_texture.raw(),
        None,
        image_info
    )?;

    local_encoder.flush(&mut *gfx.device);

    let shader_resource = gfx.factory.view_texture_as_shader_resource::<gfx::format::Srgba8>(
            &target_texture,
            (0, 0),
            gfx::format::Swizzle::new(),
        )?;
    let image = Image {
        texture: shader_resource,
        texture_handle: target_texture,
        sampler_info: gfx.default_sampler_info,
        blend_mode: None,
        width: w as u32,
        height: h as u32,
    };

    Ok(image)
}

/*
// Draw an arc.
// Punting on this until later.
pub fn arc(_ctx: &mut Context,
           _mode: DrawMode,
           _point: Point,
           _radius: f32,
           _angle1: f32,
           _angle2: f32,
           _segments: u32)
           -> GameResult<()> {
    unimplemented!();
}
*/

/// Draw a circle.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
///
/// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
pub fn circle(
    ctx: &mut Context,
    mode: DrawMode,
    point: Point2,
    radius: f32,
    tolerance: f32,
) -> GameResult<()> {
    let m = Mesh::new_circle(ctx, mode, point, radius, tolerance)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

/// Draw an ellipse.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
///
/// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
pub fn ellipse(
    ctx: &mut Context,
    mode: DrawMode,
    point: Point2,
    radius1: f32,
    radius2: f32,
    tolerance: f32,
) -> GameResult<()> {
    let m = Mesh::new_ellipse(ctx, mode, point, radius1, radius2, tolerance)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

/// Draws a line of one or more connected segments.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn line(ctx: &mut Context, points: &[Point2], width: f32) -> GameResult<()> {
    let m = Mesh::new_line(ctx, points, width)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

/// Draws points (as rectangles)
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn points(ctx: &mut Context, points: &[Point2], point_size: f32) -> GameResult<()> {
    for p in points {
        let r = Rect::new(p.x, p.y, point_size, point_size);
        rectangle(ctx, DrawMode::Fill, r)?;
    }
    Ok(())
}

/// Draws a closed polygon
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn polygon(ctx: &mut Context, mode: DrawMode, vertices: &[Point2]) -> GameResult<()> {
    let m = Mesh::new_polygon(ctx, mode, vertices)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

// Renders text with the default font.
// Not terribly efficient as it re-renders the text with each call,
// but good enough for debugging.
// Doesn't actually work, double-borrow on ctx.  Bah.
// pub fn print(ctx: &mut Context, dest: Point, text: &str) -> GameResult<()> {
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
pub fn rectangle(ctx: &mut Context, mode: DrawMode, rect: Rect) -> GameResult<()> {
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
    polygon(ctx, mode, &pts)
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Returns the current background color.
pub fn get_background_color(ctx: &Context) -> Color {
    ctx.gfx_context.background_color
}

/// Returns the current foreground color.
pub fn get_color(ctx: &Context) -> Color {
    ctx.gfx_context.foreground_color
}

/// Get the default filter mode for new images.
pub fn get_default_filter(ctx: &Context) -> FilterMode {
    let gfx = &ctx.gfx_context;
    gfx.default_sampler_info.filter.into()
}

/// Returns a string that tells a little about the obtained rendering mode.
/// It is supposed to be human-readable and will change; do not try to parse
/// information out of it!
pub fn get_renderer_info(ctx: &Context) -> GameResult<String> {
    let video = ctx.sdl_context.video()?;

    let gl = video.gl_attr();

    Ok(format!(
        "Requested GL {}.{} Core profile, actually got GL {}.{} {:?} profile.",
        ctx.gfx_context.backend_spec.major,
        ctx.gfx_context.backend_spec.minor,
        gl.context_major_version(),
        gl.context_minor_version(),
        gl.context_profile()
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

/// Sets the background color.  Default: blue.
pub fn set_background_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.background_color = color;
}

/// Sets the foreground color, which will be used for drawing
/// rectangles, lines, etc.  Default: white.
pub fn set_color(ctx: &mut Context, color: Color) -> GameResult<()> {
    let gfx = &mut ctx.gfx_context;
    gfx.foreground_color = color;
    Ok(())
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
pub fn set_screen_coordinates(context: &mut Context, rect: Rect) -> GameResult<()> {
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
pub fn apply_transformations(context: &mut Context) -> GameResult<()> {
    let gfx = &mut context.gfx_context;
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}

/// Sets the blend mode of the currently active shader program
pub fn set_blend_mode(ctx: &mut Context, mode: BlendMode) -> GameResult<()> {
    ctx.gfx_context.set_blend_mode(mode)
}

/// Sets the window mode, such as the size and other properties.
///
/// Setting the window mode may have side effects, such as clearing
/// the screen or setting the screen coordinates viewport to some undefined value.
/// It is recommended to call `set_screen_coordinates()` after changing the window
/// size to make sure everything is what you want it to be.
pub fn set_mode(context: &mut Context, mode: WindowMode) -> GameResult<()> {
    {
        let gfx = &mut context.gfx_context;
        gfx.set_window_mode(mode)?;
    }
    {
        let video = &mut context.sdl_context.video()?;
        GraphicsContext::set_vsync(video, mode.vsync);
    }
    Ok(())
}

/// Toggles the fullscreen state of the window subsystem
///
pub fn set_fullscreen(context: &mut Context, fullscreen: bool) -> GameResult<()> {
    let fs_type = if fullscreen {
        sdl2::video::FullscreenType::True
    } else {
        sdl2::video::FullscreenType::Off
    };
    let gfx = &mut context.gfx_context;
    gfx.window.set_fullscreen(fs_type)?;

    Ok(())
}

/// Queries the fullscreen state of the window subsystem.
/// If true, then the game is running in fullscreen mode.
///
pub fn is_fullscreen(context: &mut Context) -> bool {
    let gfx = &context.gfx_context;
    gfx.window.fullscreen_state() == sdl2::video::FullscreenType::True
}

/// Sets the window resolution based on the specified width and height
///
pub fn set_resolution(context: &mut Context, width: u32, height: u32) -> GameResult<()> {
    let mut window_mode = context.conf.window_mode;
    window_mode.width = width;
    window_mode.height = height;
    set_mode(context, window_mode)
}

/// Returns a `Vec` of `(width, height)` tuples describing what
/// fullscreen resolutions are available for the given display.
pub fn get_fullscreen_modes(context: &Context, display_idx: i32) -> GameResult<Vec<(u32, u32)>> {
    let video = context.sdl_context.video()?;
    let display_count = video.num_video_displays()?;
    assert!(display_idx < display_count);

    let num_modes = video.num_display_modes(display_idx)?;

    (0..num_modes)
        .map(|i| video.display_mode(display_idx, i))
        .map(|ires| ires.map_err(GameError::VideoError))
        .map(|gres| gres.map(|dispmode| (dispmode.w as u32, dispmode.h as u32)))
        .collect()
}

/// Returns the number of connected displays.
pub fn get_display_count(context: &Context) -> GameResult<i32> {
    let video = context.sdl_context.video()?;
    video.num_video_displays().map_err(GameError::VideoError)
}

/// Returns a reference to the SDL window.
/// Ideally you should not need to use this because ggez
/// would provide all the functions you need without having
/// to dip into SDL itself.  But life isn't always ideal.
pub fn get_window(context: &Context) -> &sdl2::video::Window {
    let gfx = &context.gfx_context;
    &gfx.window
}

/// Returns a mutable reference to the SDL window.
pub fn get_window_mut(context: &mut Context) -> &mut sdl2::video::Window {
    let gfx = &mut context.gfx_context;
    &mut gfx.window
}

/// Returns the size of the window in pixels as (height, width).
pub fn get_size(context: &Context) -> (u32, u32) {
    let gfx = &context.gfx_context;
    gfx.window.size()
}

/// Returns the size of the window's underlying drawable in pixels as (height, width).
/// This may return a different value than `get_size()` when run on a platform with high-DPI support
pub fn get_drawable_size(context: &Context) -> (u32, u32) {
    let gfx = &context.gfx_context;
    gfx.window.drawable_size()
}

/// EXPERIMENTAL function to get the gfx-rs `Factory` object.
pub fn get_factory(context: &mut Context) -> &mut gfx_device_gl::Factory {
    let gfx = &mut context.gfx_context;
    &mut gfx.factory
}

/// EXPERIMENTAL function to get the gfx-rs `Device` object.
pub fn get_device(context: &mut Context) -> &mut gfx_device_gl::Device {
    let gfx = &mut context.gfx_context;
    gfx.device.as_mut()
}

/// EXPERIMENTAL function to get the gfx-rs `Encoder` object.
pub fn get_encoder(
    context: &mut Context,
) -> &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> {
    let gfx = &mut context.gfx_context;
    &mut gfx.encoder
}

/// EXPERIMENTAL function to get the gfx-rs depth view
pub fn get_depth_view(
    context: &mut Context,
) -> gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil> {
    let gfx = &mut context.gfx_context;
    gfx.depth_view.clone()
}

/// EXPERIMENTAL function to get the gfx-rs color view
pub fn get_screen_render_target(
    context: &Context,
) -> gfx::handle::RenderTargetView<
    gfx_device_gl::Resources,
    (gfx::format::R8_G8_B8_A8, gfx::format::Srgb),
> {
    let gfx = &context.gfx_context;
    gfx.data.out.clone()
}

/// EXPERIMENTAL function to get gfx-rs objects.
/// Getting them one by one is awkward 'cause it tends to create double-borrows
/// on the Context object.
pub fn get_gfx_objects(
    context: &mut Context,
) -> (
    &mut <GlBackendSpec as BackendSpec>::Factory,
    &mut <GlBackendSpec as BackendSpec>::Device,
    &mut gfx::Encoder<<GlBackendSpec as BackendSpec>::Resources, <GlBackendSpec as BackendSpec>::CommandBuffer>,
    gfx::handle::DepthStencilView<<GlBackendSpec as BackendSpec>::Resources, gfx::format::DepthStencil>,
    gfx::handle::RenderTargetView<
        <GlBackendSpec as BackendSpec>::Resources,
        (gfx::format::R8_G8_B8_A8, gfx::format::Srgb),
    >,
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
    /// Actually draws the object to the screen.
    ///
    /// This is the most general version of the operation, which is all that
    /// is required for implementing this trait.
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()>;

    /// Draws the drawable onto the rendering target.
    ///
    /// It just is a shortcut that calls `draw_ex()` with a default `DrawParam`
    /// except for the destination and rotation.
    ///
    /// * `ctx` - The `Context` this graphic will be rendered to.
    /// * `dest` - the position to draw the graphic expressed as a `Point2`.
    /// * `rotation` - orientation of the graphic in radians.
    ///
    fn draw(&self, ctx: &mut Context, dest: Point2, rotation: f32) -> GameResult<()> {
        self.draw_ex(
            ctx,
            DrawParam {
                dest: dest,
                rotation: rotation,
                ..Default::default()
            },
        )
    }

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
