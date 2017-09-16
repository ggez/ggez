//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text.
//!
//! This module also manages graphics state, coordinate systems, etc.
//! The default coordinate system has the origin in the upper-left
//! corner of the screen.

use std::fmt;
use std::path;
use std::convert::From;
use std::collections::HashMap;
use std::io::Read;
use std::u16;

use sdl2;
use image;
use gfx;
use gfx::texture;
use gfx::traits::Device;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_sdl;
use gfx::Factory;

use nalgebra as na;


use context::Context;
use GameError;
use GameResult;

mod text;
mod types;
pub mod spritebatch;

pub use self::text::*;
pub use self::types::*;

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;



const QUAD_VERTS: [Vertex; 4] = [Vertex {
                                     pos: [-0.5, -0.5],
                                     uv: [0.0, 0.0],
                                 },
                                 Vertex {
                                     pos: [0.5, -0.5],
                                     uv: [1.0, 0.0],
                                 },
                                 Vertex {
                                     pos: [0.5, 0.5],
                                     uv: [1.0, 1.0],
                                 },
                                 Vertex {
                                     pos: [-0.5, 0.5],
                                     uv: [0.0, 1.0],
                                 }];

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

    /// Internal structure containing values that are different for each rect.
    vertex InstanceProperties {
        src: [f32; 4] = "a_Src",
        dest: [f32; 2] = "a_Dest",
        scale: [f32; 2] = "a_Scale",
        offset: [f32; 2] = "a_Offset",
        shear: [f32; 2] = "a_Shear",
        rotation: f32 = "a_Rotation",
    }

    /// Internal structure containing global shader state.
    constant Globals {
        mvp_matrix: [[f32; 4]; 4] = "u_MVP",
        color: [f32; 4] = "u_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        globals: gfx::ConstantBuffer<Globals> = "Globals",
        rect_instance_properties: gfx::InstanceBuffer<InstanceProperties> = (),
        out: gfx::BlendTarget<ColorFormat> =
          ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

impl Default for InstanceProperties {
    fn default() -> Self {
        InstanceProperties {
            src: [0.0, 0.0, 1.0, 1.0],
            dest: [0.0, 0.0],
            scale: [1.0, 1.0],
            offset: [0.0, 0.0],
            shear: [0.0, 0.0],
            rotation: 0.0,
        }
    }
}

impl From<DrawParam> for InstanceProperties {
    fn from(p: DrawParam) -> Self {
        InstanceProperties {
            src: p.src.into(),
            dest: types::pt2arr(p.dest),
            scale: types::pt2arr(p.scale),
            offset: types::pt2arr(p.offset),
            shear: types::pt2arr(p.shear),
            rotation: p.rotation,
        }
    }
}

/// A structure for conveniently storing Sampler's, based off
/// their `SamplerInfo`.
///
/// Making this generic is tricky 'cause it has methods that depend
/// on the generic Factory trait, it seems, so for now we just kind
/// of hack it.
struct SamplerCache<R>
    where R: gfx::Resources
{
    samplers: HashMap<texture::SamplerInfo, gfx::handle::Sampler<R>>,
}

impl<R> SamplerCache<R>
    where R: gfx::Resources
{
    fn new() -> Self {
        SamplerCache { samplers: HashMap::new() }
    }
    fn get_or_insert<F>(&mut self,
                        info: texture::SamplerInfo,
                        factory: &mut F)
                        -> gfx::handle::Sampler<R>
        where F: gfx::Factory<R>
    {
        let sampler = self.samplers
            .entry(info)
            .or_insert_with(|| factory.create_sampler(info));
        sampler.clone()
    }
}

/// A structure that contains graphics state.
/// For instance, background and foreground colors,
/// window info, DPI, rendering pipeline state, etc.
///
/// As an end-user you shouldn't ever have to touch this, but it goes
/// into part of the `Context` and so has to be public, at least
/// until the `pub(restricted)` feature is stable.
pub struct GraphicsContextGeneric<R, F, C, D>
    where R: gfx::Resources,
          F: gfx::Factory<R>,
          C: gfx::CommandBuffer<R>,
          D: gfx::Device<Resources = R, CommandBuffer = C>
{
    background_color: Color,
    shader_globals: Globals,
    projection: Matrix4,
    transform_stack: Vec<Matrix4>,
    view_stack: Vec<Matrix4>,
    white_image: Image,
    line_width: f32,
    point_size: f32,
    screen_rect: Rect,
    dpi: (f32, f32, f32),

    window: sdl2::video::Window,
    #[allow(dead_code)]
    gl_context: sdl2::video::GLContext,
    device: Box<D>,
    factory: Box<F>,
    encoder: gfx::Encoder<R, C>,
    // color_view: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
    #[allow(dead_code)]
    depth_view: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,

    pso: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
    quad_slice: gfx::Slice<R>,
    quad_vertex_buffer: gfx::handle::Buffer<R, Vertex>,
    default_sampler_info: texture::SamplerInfo,
    samplers: SamplerCache<R>,
}

impl<R, F, C, D> fmt::Debug for GraphicsContextGeneric<R, F, C, D>
    where R: gfx::Resources,
          F: gfx::Factory<R>,
          C: gfx::CommandBuffer<R>,
          D: gfx::Device<Resources = R, CommandBuffer = C>
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<GraphicsContext: {:p}>", self)
    }
}

/// A concrete graphics context for GL rendering.
pub type GraphicsContext = GraphicsContextGeneric<gfx_device_gl::Resources,
                                                  gfx_device_gl::Factory,
                                                  gfx_device_gl::CommandBuffer,
                                                  gfx_device_gl::Device>;

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
            let result = gfx_window_sdl::init::<ColorFormat, DepthFormat>(window_builder);
            match result {
                Ok(_) => {
                    println!("Ok, got GL {}.{}.",
                             gl.context_major_version(),
                             gl.context_minor_version())
                }
                Err(res) => println!("Request failed: {:?}", res),
            }
        }
    }
}

impl From<gfx::buffer::CreationError> for GameError {
    fn from(e: gfx::buffer::CreationError) -> Self {
        use gfx::buffer::CreationError;
        match e {
            CreationError::UnsupportedBind(b) => {
                GameError::RenderError(format!("Could not create buffer: Unsupported Bind ({:?})",
                                               b))
            }
            CreationError::UnsupportedUsage(u) => {
                GameError::RenderError(format!("Could not create buffer: Unsupported Usage ({:?})",
                                               u))
            }
            CreationError::Other => GameError::RenderError(format!("Could not create buffer: Unknown error")),
        }
    }
}

impl GraphicsContext {
    pub fn new(video: sdl2::VideoSubsystem,
               window_title: &str,
               screen_width: u32,
               screen_height: u32,
               vsync: bool,
               resize: bool)
               -> GameResult<GraphicsContext> {
        // WINDOW SETUP
        let gl = video.gl_attr();
        gl.set_context_version(GL_MAJOR_VERSION, GL_MINOR_VERSION);
        gl.set_context_profile(sdl2::video::GLProfile::Core);
        gl.set_red_size(5);
        gl.set_green_size(5);
        gl.set_blue_size(5);
        gl.set_alpha_size(8);
        let mut window_builder = video.window(window_title, screen_width, screen_height);
        if resize {
            window_builder.resizable();
        }
        let (window, gl_context, device, mut factory, color_view, depth_view) =
            gfx_window_sdl::init(window_builder)?;

        // println!("Vsync enabled: {}", vsync);
        let vsync_int = if vsync { 1 } else { 0 };
        video.gl_set_swap_interval(vsync_int);

        let display_index = window.display_index()?;
        let dpi = window.subsystem().display_dpi(display_index)?;

        // GFX SETUP
        let encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> =
            factory.create_command_buffer().into();

        let pso = factory
            .create_pipeline_simple(include_bytes!("shader/basic_150.glslv"),
                                    include_bytes!("shader/basic_150.glslf"),
                                    pipe::new())?;

        let rect_inst_props = factory
            .create_buffer(1,
                           gfx::buffer::Role::Vertex,
                           gfx::memory::Usage::Dynamic,
                           gfx::SHADER_RESOURCE)?;

        let (quad_vertex_buffer, mut quad_slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTS, &QUAD_INDICES[..]);

        quad_slice.instances = Some((1, 0));

        let globals_buffer = factory.create_constant_buffer(1);
        let mut samplers: SamplerCache<gfx_device_gl::Resources> = SamplerCache::new();
        let sampler_info = texture::SamplerInfo::new(texture::FilterMethod::Bilinear,
                                                     texture::WrapMode::Clamp);
        let sampler = samplers.get_or_insert(sampler_info, &mut factory);
        let white_image =
            Image::make_raw(&mut factory, &sampler_info, 1, 1, &[255, 255, 255, 255])?;
        let texture = white_image.texture.clone();

        let data = pipe::Data {
            vbuf: quad_vertex_buffer.clone(),
            tex: (texture, sampler),
            rect_instance_properties: rect_inst_props,
            globals: globals_buffer,
            out: color_view,
        };

        // Set initial uniform values
        let left = 0.0;
        let right = screen_width as f32;
        let top = 0.0;
        let bottom = screen_height as f32;
        let initial_projection = Matrix4::identity(); // not the actual initial projection matrix, just placeholder
        let initial_view = Matrix4::identity();
        let initial_transform = Matrix4::identity();
        let globals = Globals {
            mvp_matrix: initial_projection.into(),
            color: types::WHITE.into(),
        };


        let mut gfx = GraphicsContext {
            background_color: Color::new(0.1, 0.2, 0.3, 1.0),
            shader_globals: globals,
            projection: initial_projection,
            transform_stack: vec![initial_transform],
            view_stack: vec![initial_view],
            line_width: 1.0,
            point_size: 1.0,
            white_image: white_image,
            screen_rect: Rect::new(left, bottom, (right - left), (top - bottom)),
            dpi: dpi,

            window: window,
            gl_context: gl_context,
            device: Box::new(device),
            factory: Box::new(factory),
            encoder: encoder,
            depth_view: depth_view,

            pso: pso,
            data: data,
            quad_slice: quad_slice,
            quad_vertex_buffer: quad_vertex_buffer,
            default_sampler_info: sampler_info,
            samplers: samplers,
        };

        // Calculate and apply the actual initial projection matrix
        let w = screen_width as f32;
        let h = screen_height as f32;
        let rect = Rect {
            x: (w / 2.0),
            y: (h / 2.0),
            w,
            h: -h,
        };
        gfx.set_projection_rect(rect);
        gfx.calculate_transform_matrix();
        gfx.update_globals()?;
        Ok(gfx)
    }

    /// Sends the current value of the graphics context's shader globals
    /// to the graphics card.
    fn update_globals(&mut self) -> GameResult<()> {
        self.encoder
            .update_buffer(&self.data.globals, &[self.shader_globals], 0)?;
        Ok(())
    }

    /// Recalculates the context's Model-View-Projection matrix based on
    /// the matrices on the top of the respective stacks and the projection
    /// matrix.
    fn calculate_transform_matrix(&mut self) {
        let model = self.transform_stack[self.transform_stack.len() - 1];
        let view = self.view_stack[self.view_stack.len() - 1];
        let mvp = self.projection * view * model;
        self.shader_globals.mvp_matrix = mvp.into();
    }

    /// Pushes a homogeneous transform matrix to the top of the transform 
    /// (model) matrix stack.
    fn push_transform(&mut self, t: Matrix4) {
        self.transform_stack.push(t);
    }

    /// Pops the current transform matrix off the top of the transform 
    /// (model) matrix stack.
    fn pop_transform(&mut self) {
        if self.transform_stack.len() > 1 {
            self.transform_stack.pop();
        }
    }

    /// Sets the current transform matrix.
    fn set_transform(&mut self, t: Matrix4) {
        let idx = self.transform_stack.len() - 1;
        self.transform_stack[idx] = t;
    }

    /// Pushes a homogeneous transform matrix to the top of the view 
    /// matrix stack.
    fn push_view(&mut self, v: Matrix4) {
        self.view_stack.push(v);
    }

    /// Pops the current transform matrix off the top of the view 
    /// matrix stack.
    fn pop_view(&mut self) {
        if self.view_stack.len() > 1 {
            self.view_stack.pop();
        }
    }

    /// Sets the current transform matrix.
    fn set_view(&mut self, t: Matrix4) {
        let idx = self.view_stack.len() - 1;
        self.view_stack[idx] = t;
    }

    /// Converts the given `DrawParam` into an `InstanceProperties` object and
    /// sends it to the graphics card at the front of the instance buffer.
    fn update_instance_properties(&mut self, draw_params: DrawParam) -> GameResult<()> {
        let properties = draw_params.into();
        self.encoder
            .update_buffer(&self.data.rect_instance_properties, &[properties], 0)?;
        Ok(())
    }

    /// Returns a reference to the SDL window.
    /// Ideally you should not need to use this because ggez
    /// would provide all the functions you need without having
    /// to dip into SDL itself.  But life isn't always ideal.
    pub fn get_window(&self) -> &sdl2::video::Window {
        &self.window
    }

    /// Returns a mutable reference to the SDL window.
    pub fn get_window_mut(&mut self) -> &mut sdl2::video::Window {
        &mut self.window
    }


    /// Returns the size of the window in pixels as (height, width).
    pub fn get_size(&self) -> (u32, u32) {
        self.window.size()
    }

    /// Returns the size of the window's underlaying drawable in pixels as (height, width).
    /// This may return a different value than `get_size()` when run on a platform with high-DPI support
    pub fn get_drawable_size(&self) -> (u32, u32) {
        self.window.drawable_size()
    }

    /// EXPERIMENTAL function to get the gfx-rs `Factory` object.
    pub fn get_factory(&mut self) -> &mut gfx_device_gl::Factory {
        &mut self.factory
    }

    /// EXPERIMENTAL function to get the gfx-rs `Device` object.
    pub fn get_device(&mut self) -> &mut gfx_device_gl::Device {
        self.device.as_mut()
    }

    /// EXPERIMENTAL function to get the gfx-rs `Encoder` object.
    pub fn get_encoder
        (&mut self)
         -> &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> {
        &mut self.encoder
    }

    /// EXPERIMENTAL function to get the gfx-rs depth view
    pub fn get_depth_view
        (&self)
         -> gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil> {
        self.depth_view.clone()
    }

    /// EXPERIMENTAL function to get the gfx-rs color view
    pub fn get_color_view(&self)
                          -> gfx::handle::RenderTargetView<gfx_device_gl::Resources,
                                                           (gfx::format::R8_G8_B8_A8,
                                                            gfx::format::Srgb)> {
        self.data.out.clone()
    }

    /// Shortcut function to set the screen rect ortho mode
    /// to a given `Rect`.
    ///
    /// Call `update_globals()` to apply them after calling this.
    fn set_projection_rect(&mut self, rect: Rect) {
        type Vec3 = na::Vector3<f32>;
        self.screen_rect = rect;
        let half_width = rect.w / 2.0;
        let half_height = rect.h / 2.0;
        self.projection = Matrix4::new_orthographic(
            rect.x - half_width,
            rect.x + half_width,
            rect.y + half_height,
            rect.y - half_height,
            -1.0,
            1.0)
            .append_nonuniform_scaling(&Vec3::new(1.0, -1.0, 1.0));
    }
}

// **********************************************************************
// DRAWING
// **********************************************************************

/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    gfx.encoder
        .clear(&gfx.data.out, gfx.background_color.into());
}

/// Draws the given `Drawable` object to the screen by calling its
/// `draw()` method.
pub fn draw(ctx: &mut Context, drawable: &Drawable, dest: Point, rotation: f32) -> GameResult<()> {
    drawable.draw(ctx, dest, rotation)
}


/// Draws the given `Drawable` object to the screen by calling its `draw_ex()` method.
pub fn draw_ex(ctx: &mut Context, drawable: &Drawable, params: DrawParam) -> GameResult<()> {
    drawable.draw_ex(ctx, params)
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your `EventHandler`'s `draw()` method.
pub fn present(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    // We might want to give the user more control over when the
    // encoder gets flushed eventually, if we want them to be able
    // to do their own gfx drawing.  HOWEVER, the whole pipeline type
    // thing is a bigger hurdle, so this is fine for now.
    gfx.encoder.flush(&mut *gfx.device);
    gfx.window.gl_swap_window();
    gfx.device.cleanup();
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
pub fn circle(ctx: &mut Context,
              mode: DrawMode,
              point: Point,
              radius: f32,
              tolerance: f32)
              -> GameResult<()> {
    let m = Mesh::new_circle(ctx, mode, point, radius, tolerance)?;
    m.draw(ctx, Point::origin(), 0.0)
}

/// Draw an ellipse.
pub fn ellipse(ctx: &mut Context,
               mode: DrawMode,
               point: Point,
               radius1: f32,
               radius2: f32,
               tolerance: f32)
               -> GameResult<()> {
    let m = Mesh::new_ellipse(ctx, mode, point, radius1, radius2, tolerance)?;
    m.draw(ctx, Point::origin(), 0.0)
}

/// Draws a line of one or more connected segments.
pub fn line(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    let w = ctx.gfx_context.line_width;
    let m = Mesh::new_line(ctx, points, w)?;
    m.draw(ctx, Point::origin(), 0.0)
}

/// Draws points.
pub fn points(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    let size = ctx.gfx_context.point_size;
    for p in points {
        let r = Rect::new(p.x, p.y, size, size);
        rectangle(ctx, DrawMode::Fill, r)?;
    }
    Ok(())
}

/// Draws a closed polygon
pub fn polygon(ctx: &mut Context, mode: DrawMode, vertices: &[Point]) -> GameResult<()> {
    let w = ctx.gfx_context.line_width;
    let m = Mesh::new_polygon(ctx, mode, vertices, w)?;
    m.draw(ctx, Point::origin(), 0.0)
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
pub fn rectangle(ctx: &mut Context, mode: DrawMode, rect: Rect) -> GameResult<()> {
    let x = rect.x;
    let y = rect.y;
    let w = rect.w;
    let h = rect.h;
    let x1 = x - (w / 2.0);
    let x2 = x + (w / 2.0);
    let y1 = y - (h / 2.0);
    let y2 = y + (h / 2.0);
    let pts = [Point::new(x1, y1),
               Point::new(x2, y1),
               Point::new(x2, y2),
               Point::new(x1, y2)];
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
    ctx.gfx_context.shader_globals.color.into()
}

/// Get the default filter mode for new images.
pub fn get_default_filter(ctx: &Context) -> FilterMode {
    let gfx = &ctx.gfx_context;
    gfx.default_sampler_info.filter.into()
}


/// Get the current width for drawing lines and stroked polygons.
pub fn get_line_width(ctx: &Context) -> f32 {
    ctx.gfx_context.line_width
}


/// Get the current size for drawing points.
pub fn get_point_size(ctx: &Context) -> f32 {
    ctx.gfx_context.point_size
}

/// Returns a string that tells a little about the obtained rendering mode.
/// It is supposed to be human-readable and will change; do not try to parse
/// information out of it!
pub fn get_renderer_info(ctx: &Context) -> GameResult<String> {
    let video = ctx.sdl_context.video()?;

    let gl = video.gl_attr();

    Ok(format!("Requested GL {}.{} Core profile, actually got GL {}.{} {:?} profile.",
               GL_MAJOR_VERSION,
               GL_MINOR_VERSION,
               gl.context_major_version(),
               gl.context_minor_version(),
               gl.context_profile()))
}

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: center_x, y: cenyer_y, w: width, h: height }`
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
    gfx.shader_globals.color = color.into();
    gfx.update_globals()
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
    let _sampler = gfx.samplers
        .get_or_insert(sampler_info, &mut *gfx.factory);
    gfx.default_sampler_info = sampler_info;
}

/// Set the current width for drawing lines and stroked polygons.
pub fn set_line_width(ctx: &mut Context, width: f32) {
    ctx.gfx_context.line_width = width;
}

/// Set the current size for drawing points.
pub fn set_point_size(ctx: &mut Context, size: f32) {
    ctx.gfx_context.point_size = size;
}

/// Sets the bounds of the screen viewport.
///
/// The default coordinate system has (0,0) at the top-left corner
/// with X increasing to the right and Y increasing down, with the
/// viewport scaled such that one coordinate unit is one pixel on the
/// screen.  This function lets you change this coordinate system to
/// be whatever you prefer.
///
/// Recall that a `Rect` currently the x and y coordinates at the center,
/// so if you wanted a coordinate system from (0,0) at the bottom-left
/// to (640, 480) at the top-right, you would call this function with
/// a `Rect{x: 320, y: 240, w: 640, h: 480}`
pub fn set_screen_coordinates(context: &mut Context, rect: Rect) -> GameResult<()> {
    let gfx = &mut context.gfx_context;
    gfx.set_projection_rect(rect);
    gfx.update_globals()
}

/// Pushes a homogeneous transform matrix to the top of the transform 
/// (model) matrix stack of the `Context`. If no matrix is given, then
/// pushes a copy of the current transform matrix to the top of the stack.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn push_transform(context: &mut Context, transform: Option<Matrix4>) {
    let gfx = &mut context.gfx_context;
    if let Some(t) = transform {
        gfx.push_transform(t);
    } else {
        let copy = gfx.transform_stack[gfx.transform_stack.len() - 1].clone();
        gfx.push_transform(copy);
    }
}

/// Pops the transform matrix off the top of the transform 
/// (model) matrix stack of the `Context`.
pub fn pop_transform(context: &mut Context) {
    let gfx = &mut context.gfx_context;
    gfx.pop_transform();
}

/// Sets the current model transformation to the given homogeneous
/// transformation matrix.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn set_transform(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    gfx.set_transform(transform);
}

/// Appends the given transform to the current model transform.
/// 
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn transform(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    let curr = gfx.transform_stack[gfx.transform_stack.len() - 1].clone();
    gfx.set_transform(transform * curr);
}

/// Sets the current model transform to the origin transform (no transformation)
pub fn origin(context: &mut Context) {
    let gfx = &mut context.gfx_context;
    gfx.set_transform(Matrix4::identity());
}

/// Pushes a homogeneous transform matrix to the top of the view 
/// matrix stack of the `Context`.
pub fn push_view(context: &mut Context, view: Matrix4) {
    let gfx = &mut context.gfx_context;
    gfx.push_view(view);
}

/// Pops the transform matrix off the top of the view 
/// matrix stack of the `Context`.
pub fn pop_view(context: &mut Context) {
    let gfx = &mut context.gfx_context;
    gfx.pop_view();
}

/// Sets the current view matrix to the given homogeneous
/// transformation matrix.
pub fn set_view(context: &mut Context, view: Matrix4) {
    let gfx = &mut context.gfx_context;
    gfx.set_view(view);
}

/// Appends the given transformation matrix to the current view transform matrix 
pub fn transform_view(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    let curr = gfx.view_stack[gfx.view_stack.len() - 1].clone();
    gfx.set_view(transform * curr);
}

/// Calculates the new total transformation (Model-View-Projection) matrix
/// based on the matrices at the top of the transform and view matrix stacks
/// and sends it to the graphics card.
pub fn apply_transformations(context: &mut Context) -> GameResult<()> {
    let gfx = &mut context.gfx_context;
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}

/// Sets the window mode, such as the size and other properties.
///
/// Setting the window mode may have side effects, such as clearing
/// the screen or setting the screen coordinates viewport to some undefined value.
/// It is recommended to call `set_screen_coordinates()` after changing the window
/// size to make sure everything is what you want it to be.
pub fn set_mode(context: &mut Context,
                width: u32,
                height: u32,
                mode: WindowMode)
                -> GameResult<()> {
    {
        let window = &mut context.gfx_context.get_window_mut();
        window.set_size(width, height)?;
        // SDL sets "bordered" but Love2D does "not bordered";
        // we use the Love2D convention.
        window.set_bordered(!mode.borderless);
        window.set_fullscreen(mode.fullscreen_type)?;
        let (min_w, min_h) = mode.min_dimensions;
        window.set_minimum_size(min_w, min_h)?;
        let (max_w, max_h) = mode.max_dimensions;
        window.set_maximum_size(max_w, max_h)?;
    }
    {
        let video = context.sdl_context.video()?;
        let vsync_int = if mode.vsync { 1 } else { 0 };
        video.gl_set_swap_interval(vsync_int);
    }
    Ok(())
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

// **********************************************************************
// TYPES
// **********************************************************************


/// A struct containing all the necessary info for drawing a Drawable.
///
/// * `src` - a portion of the drawable to clip, as a fraction of the whole image.
///    Defaults to the whole image (1.0) if omitted.
/// * `dest` - the position to draw the graphic expressed as a `Point`.
/// * `rotation` - orientation of the graphic in radians.
/// * `scale` - x/y scale factors expressed as a `Point`.
/// * `offset` - specifies an offset from the center for transform operations like scale/rotation.
/// * `shear` - x/y shear factors expressed as a `Point`.
///
/// This struct implements the `Default` trait, so you can just do:
///
/// `graphics::draw_ex(ctx, drawable, DrawParam{ dest: my_dest, .. Default::default()} )`
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawParam {
    pub src: Rect,
    pub dest: Point,
    pub rotation: f32,
    pub scale: Point,
    pub offset: Point,
    pub shear: Point,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            src: Rect::one(),
            dest: Point::origin(),
            rotation: 0.0,
            scale: Point::new(1.0, 1.0),
            offset: Point::new(0.0, 0.0),
            shear: Point::new(0.0, 0.0),
        }
    }
}

impl DrawParam {
    fn into_matrix(self) -> Matrix4 {
        type Vec3 = na::Vector3<f32>;
        let translate = Matrix4::new_translation(&Vec3::new(self.dest.x, self.dest.y, 0.0));
        let offset = Matrix4::new_translation(&Vec3::new(self.offset.x, self.offset.y, 0.0));
        let offset_inverse = Matrix4::new_translation(&Vec3::new(-self.offset.x, -self.offset.y, 0.0));
        let axang = Vec3::z() * self.rotation;
        let rotation = Matrix4::new_rotation(axang);
        let scale = Matrix4::new_nonuniform_scaling(&Vec3::new(self.scale.x, self.scale.y, 1.0));
        let shear = Matrix4::new(
            1.0, self.shear.x, 0.0, 0.0,
            self.shear.y, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        translate * offset * rotation * shear * scale * offset_inverse * Matrix4::identity()
    }
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
    /// It just is a shortcut that calls `draw_ex()` with some sane defaults.
    ///
    /// * `ctx` - The `Context` this graphic will be rendered to.
    /// * `dest` - the position to draw the graphic expressed as a `Point`.
    /// * `rotation` - orientation of the graphic in radians.
    ///
    fn draw(&self, ctx: &mut Context, dest: Point, rotation: f32) -> GameResult<()> {
        self.draw_ex(ctx,
                     DrawParam {
                         dest: dest,
                         rotation: rotation,
                         ..Default::default()
                     })
    }
}

/// Generic in-GPU-memory image data available to be drawn on the screen.
#[derive(Clone)]
pub struct ImageGeneric<R>
    where R: gfx::Resources
{
    texture: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    sampler_info: gfx::texture::SamplerInfo,
    width: u32,
    height: u32,
}

/// In-GPU-memory image data available to be drawn on the screen,
/// using the OpenGL backend.
pub type Image = ImageGeneric<gfx_device_gl::Resources>;

/// Copies an 2D (RGBA) buffer into one that is the next
/// power of two size up in both dimensions.  All data is
/// retained and kept closest to [0,0]; anything extra is
/// filled with 0
fn scale_rgba_up_to_power_of_2(width: u16, height: u16, rgba: &[u8]) -> (u16, u16, Vec<u8>) {
    let width = width as usize;
    let height = height as usize;
    let w2 = width.next_power_of_two();
    let h2 = height.next_power_of_two();
    // println!("Scaling from {}x{} to {}x{}", width, height, w2, h2);
    let num_vals = w2 * h2 * 4;
    let mut v: Vec<u8> = Vec::with_capacity(num_vals);
    // This is a little wasteful because we will be replacing
    // many if not most of these 0's with the actual image data.
    // But it's much simpler to resize the thing once than to blit
    // each row, resize it out to fill the rest of the row with zeroes,
    // etc.
    v.resize(num_vals, 0);
    // Blit each row of the old image into the new array.
    for i in 0..h2 {
        if i < height {
            let src_start = i * width * 4;
            let src_end = src_start + width * 4;
            let dest_start = i * w2 * 4;
            let dest_end = dest_start + width * 4;
            let slice = &mut v[dest_start..dest_end];
            slice.copy_from_slice(&rgba[src_start..src_end]);
        }
    }
    (w2 as u16, h2 as u16, v)
}

impl Image {
    /// Load a new image from the file at the given path.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Image> {
        let img = {
            let mut buf = Vec::new();
            let mut reader = context.filesystem.open(path)?;
            reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_rgba()
        };
        let (width, height) = img.dimensions();
        Image::from_rgba8(context, width as u16, height as u16, &img)
    }

    /// Creates a new `Image` from the given buffer of `u8` RGBA values.
    pub fn from_rgba8(context: &mut Context,
                      width: u16,
                      height: u16,
                      rgba: &[u8])
                      -> GameResult<Image> {
        Image::make_raw(&mut context.gfx_context.factory,
                        &context.gfx_context.default_sampler_info,
                        width,
                        height,
                        rgba)
    }
    /// A helper function that just takes a factory directly so we can make an image
    /// without needing the full context object, so we can create an Image while still
    /// creating the GraphicsContext.
    fn make_raw(factory: &mut gfx_device_gl::Factory,
                sampler_info: &texture::SamplerInfo,
                width: u16,
                height: u16,
                rgba: &[u8])
                -> GameResult<Image> {
        // Check if the texture is not power of 2, and if not, pad it out.
        let view = if false {
            // let view = if !(width.is_power_of_two() && height.is_power_of_two()) {
            let (width, height, rgba) = scale_rgba_up_to_power_of_2(width, height, rgba);
            let rgba = &rgba;
            assert_eq!((width as usize) * (height as usize) * 4, rgba.len());
            let kind = gfx::texture::Kind::D2(width, height, gfx::texture::AaMode::Single);
            // The slice containing rgba is NOT rows x columns, it is a slice of
            // MIPMAP LEVELS.  Augh!
            let (_, view) = factory
                .create_texture_immutable_u8::<gfx::format::Srgba8>(kind, &[rgba])?;
            view
        } else {
            if width == 0 || height == 0 {
                let msg = format!("Tried to create a texture of size {}x{}, each dimension must \
                     be >0",
                                  width,
                                  height);
                return Err(GameError::ResourceLoadError(msg));
            }
            let kind = gfx::texture::Kind::D2(width, height, gfx::texture::AaMode::Single);
            let (_, view) = factory
                .create_texture_immutable_u8::<gfx::format::Srgba8>(kind, &[rgba])?;
            view

        };
        Ok(Image {
               texture: view,
               sampler_info: *sampler_info,
               width: width as u32,
               height: height as u32,
           })
    }

    /// A little helper function that creates a new Image that is just
    /// a solid square of the given size and color.  Mainly useful for
    /// debugging.
    pub fn solid(context: &mut Context, size: u16, color: Color) -> GameResult<Image> {
        let pixel_array: [u8; 4] = color.into();
        let size_squared = size as usize * size as usize;
        let mut buffer = Vec::with_capacity(size_squared);
        for _i in 0..size_squared {
            buffer.extend(&pixel_array[..]);
        }
        Image::from_rgba8(context, size, size, &buffer)
    }

    /// Return the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Return the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the filter mode for the image.
    pub fn get_filter(&self) -> FilterMode {
        self.sampler_info.filter.into()
    }

    /// Set the filter mode for the image.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.sampler_info.filter = mode.into();
    }

    /// Returns the dimensions of the image.
    pub fn get_dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
    }

    /// Gets the `Image`'s `WrapMode` along the X and Y axes.
    pub fn get_wrap(&self) -> (WrapMode, WrapMode) {
        (self.sampler_info.wrap_mode.0, self.sampler_info.wrap_mode.1)
    }

    /// Sets the `Image`'s `WrapMode` along the X and Y axes.
    pub fn set_wrap(&mut self, wrap_x: WrapMode, wrap_y: WrapMode) {
        self.sampler_info.wrap_mode.0 = wrap_x;
        self.sampler_info.wrap_mode.1 = wrap_y;
    }
}


impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "<Image: {}x{}, {:p}, texture address {:p}, sampler: {:?}>",
               self.width(),
               self.height(),
               self,
               &self.texture,
               &self.sampler_info)
    }
}


impl Drawable for Image {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        let src_width = param.src.w;
        let src_height = param.src.h;
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        // We also invert the Y scale if our screen coordinates
        // are "upside down", because by default we present the
        // illusion that the screen is addressed in pixels.
        // BUGGO: Which I rather regret now.
        let invert_y = if gfx.screen_rect.h < 0.0 { 1.0 } else { -1.0 };
        let real_scale = Point::new(
            src_width * param.scale.x * self.width as f32,
            src_height * param.scale.y * self.height as f32 * invert_y,
        );
        let mut new_param = param;
        new_param.scale = real_scale;
        // Not entirely sure why the inversion is necessary, but oh well.
        new_param.offset.x *= -1.0 * param.scale.x;
        new_param.offset.y *= param.scale.y;
        gfx.update_instance_properties(new_param)?;
        let sampler = gfx.samplers
            .get_or_insert(self.sampler_info, gfx.factory.as_mut());
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        gfx.data.tex = (self.texture.clone(), sampler);
        gfx.encoder.draw(&gfx.quad_slice, &gfx.pso, &gfx.data);
        Ok(())
    }
}

/// 2D polygon mesh
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
}

use lyon::tessellation as t;

struct VertexBuilder;

impl t::VertexConstructor<t::FillVertex, Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::FillVertex) -> Vertex {
        Vertex {
            pos: [vertex.position.x, vertex.position.y],
            uv: [0.0, 0.0],
        }
    }
}

impl t::VertexConstructor<t::StrokeVertex, Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: t::StrokeVertex) -> Vertex {
        Vertex {
            pos: [vertex.position.x, vertex.position.y],
            uv: [0.0, 0.0],
        }
    }
}

impl Mesh {
    fn from_vbuf(ctx: &mut Context,
                 buffer: &t::geometry_builder::VertexBuffers<Vertex>)
                 -> GameResult<Mesh> {
        let (vbuf, slice) =
            ctx.gfx_context
                .factory
                .create_vertex_buffer_with_slice(&buffer.vertices[..], &buffer.indices[..]);

        Ok(Mesh {
               buffer: vbuf,
               slice: slice,
           })
    }


    /// Create a new mesh for a line of one or more connected segments.
    /// WIP, sorry
    pub fn new_line(ctx: &mut Context, points: &[Point], width: f32) -> GameResult<Mesh> {
        Mesh::new_polyline(ctx, DrawMode::Line, points, width)
    }

    /// Create a new mesh for a circle.
    /// Stroked circles are still WIP, sorry.
    pub fn new_circle(ctx: &mut Context,
                      mode: DrawMode,
                      point: Point,
                      radius: f32,
                      tolerance: f32)
                      -> GameResult<Mesh> {
        {
            let buffers: &mut t::geometry_builder::VertexBuffers<_> = &mut t::VertexBuffers::new();
            match mode {
                DrawMode::Fill => {
                    // These builders have to be in separate match arms 'cause they're actually
                    // different types; one is GeometryBuilder<StrokeVertex> and the other is
                    // GeometryBuilder<FillVertex>
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    t::basic_shapes::fill_circle(t::math::point(point.x, point.y),
                                                 radius,
                                                 tolerance,
                                                 builder);
                }
                DrawMode::Line => {
                    let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                    let options = t::StrokeOptions::default()
                        .with_line_width(ctx.gfx_context.line_width)
                        .with_tolerance(tolerance);
                    t::basic_shapes::stroke_circle(t::math::point(point.x, point.y),
                                                   radius,
                                                   &options,
                                                   builder);
                }
            };
            Mesh::from_vbuf(ctx, buffers)
        }

    }

    /// Create a new mesh for an ellipse.
    /// Stroked ellipses are still WIP, sorry.
    pub fn new_ellipse(ctx: &mut Context,
                       mode: DrawMode,
                       point: Point,
                       radius1: f32,
                       radius2: f32,
                       tolerance: f32)
                       -> GameResult<Mesh> {
        use euclid::Length;
        let buffers: &mut t::geometry_builder::VertexBuffers<_> = &mut t::VertexBuffers::new();
        match mode {
            DrawMode::Fill => {
                // These builders have to be in separate match arms 'cause they're actually
                // different types; one is GeometryBuilder<StrokeVertex> and the other is
                // GeometryBuilder<FillVertex>
                let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                t::basic_shapes::fill_ellipse(t::math::point(point.x, point.y),
                                              t::math::vec2(radius1, radius2),
                                              Length::new(0.0),
                                              tolerance,
                                              builder);
            }
            DrawMode::Line => {
                let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                let options = t::StrokeOptions::default()
                    .with_line_width(ctx.gfx_context.line_width)
                    .with_tolerance(tolerance);
                t::basic_shapes::stroke_ellipse(t::math::point(point.x, point.y),
                                                t::math::vec2(radius1, radius2),
                                                Length::new(0.0),
                                                &options,
                                                builder);
            }
        };
        Mesh::from_vbuf(ctx, buffers)
    }

    /// Create a new mesh for series of connected lines
    pub fn new_polyline(ctx: &mut Context,
                        mode: DrawMode,
                        points: &[Point],
                        width: f32)
                        -> GameResult<Mesh> {
        let buffers: &mut t::geometry_builder::VertexBuffers<_> = &mut t::VertexBuffers::new();
        let points = points
            .into_iter()
            .map(|ggezpoint| t::math::point(ggezpoint.x, ggezpoint.y));
        match mode {
            DrawMode::Fill => {
                // These builders have to be in separate match arms 'cause they're actually
                // different types; one is GeometryBuilder<StrokeVertex> and the other is
                // GeometryBuilder<FillVertex>
                let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                let tessellator = &mut t::FillTessellator::new();
                let options = t::FillOptions::default();
                t::basic_shapes::fill_polyline(points, tessellator, &options, builder).unwrap();
            }
            DrawMode::Line => {
                let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                let options = t::StrokeOptions::default().with_line_width(width);
                t::basic_shapes::stroke_polyline(points, false, &options, builder);
            }
        };
        Mesh::from_vbuf(ctx, buffers)
    }


    /// Create a new mesh for closed polygon
    pub fn new_polygon(ctx: &mut Context,
                       mode: DrawMode,
                       points: &[Point],
                       width: f32)
                       -> GameResult<Mesh> {
        let buffers: &mut t::geometry_builder::VertexBuffers<_> = &mut t::VertexBuffers::new();
        let points = points
            .into_iter()
            .map(|ggezpoint| t::math::point(ggezpoint.x, ggezpoint.y));
        match mode {
            DrawMode::Fill => {
                // These builders have to be in separate match arms 'cause they're actually
                // different types; one is GeometryBuilder<StrokeVertex> and the other is
                // GeometryBuilder<FillVertex>
                let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                let tessellator = &mut t::FillTessellator::new();
                let options = t::FillOptions::default();
                t::basic_shapes::fill_polyline(points, tessellator, &options, builder).unwrap();
            }
            DrawMode::Line => {
                let builder = &mut t::BuffersBuilder::new(buffers, VertexBuilder);
                let options = t::StrokeOptions::default().with_line_width(width);
                t::basic_shapes::stroke_polyline(points, true, &options, builder);
            }
        };
        Mesh::from_vbuf(ctx, buffers)
    }

    /// Create a new `Mesh` from a raw list of triangles.
    ///
    /// Currently does not support UV's or indices.
    pub fn from_triangles(ctx: &mut Context, triangles: &[Point]) -> GameResult<Mesh> {
        // This is kind of non-ideal but works for now.
        let points: Vec<Vertex> = triangles
            .into_iter()
            .map(|p| {
                     Vertex {
                         pos: types::pt2arr(*p),
                         uv: types::pt2arr(*p),
                     }
                 })
            .collect();
        let (vbuf, slice) = ctx.gfx_context
            .factory
            .create_vertex_buffer_with_slice(&points[..], ());

        Ok(Mesh {
               buffer: vbuf,
               slice: slice,
           })
    }
}

impl Drawable for Mesh {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        gfx.update_instance_properties(param)?;

        gfx.data.vbuf = self.buffer.clone();
        gfx.data.tex.0 = gfx.white_image.texture.clone();

        gfx.encoder.draw(&self.slice, &gfx.pso, &gfx.data);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_image_scaling_up() {
        let mut from: Vec<u8> = Vec::new();
        const WIDTH: u16 = 5;
        const HEIGHT: u16 = 11;
        for i in 0..HEIGHT {
            let v = vec![i as u8; WIDTH as usize * 4];
            from.extend(v.iter());
        }

        assert_eq!(from.len(), WIDTH as usize * HEIGHT as usize * 4);
        let (width, height, res) = scale_rgba_up_to_power_of_2(WIDTH, HEIGHT, &from);
        assert_eq!(width, WIDTH.next_power_of_two());
        assert_eq!(height, HEIGHT.next_power_of_two());

        for i in 0..HEIGHT.next_power_of_two() {
            for j in 0..WIDTH.next_power_of_two() {
                let offset_within_row = (j * 4) as usize;
                let src_row_offset = (i * WIDTH * 4) as usize;
                let dst_row_offset = (i * width * 4) as usize;
                println!("{} {}", i, j);
                if i < HEIGHT && j < WIDTH {
                    assert_eq!(res[dst_row_offset + offset_within_row],
                               from[src_row_offset + offset_within_row]);
                } else {
                    assert_eq!(res[dst_row_offset + offset_within_row], 0);
                }
            }
        }
    }
}
