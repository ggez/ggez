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
use std::io::Read;

use sdl2;
use image;
use gfx;
use gfx::texture;
use gfx::traits::Device;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_sdl;
use gfx::format::Rgba8;
use gfx::Factory;


use context::Context;
use GameError;
use GameResult;

mod tessellation;
mod text;
mod types;

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

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Globals {
        transform: [[f32; 4];4] = "u_Transform",
        color: [f32; 4] = "u_Color",
    }

    // Values that are different for each rect.
    constant RectProperties {
        src: [f32; 4] = "u_Src",
        dest: [f32; 2] = "u_Dest",
        scale: [f32;2] = "u_Scale",
        offset: [f32;2] = "u_Offset",
        shear: [f32;2] = "u_Shear",
        rotation: f32 = "u_Rotation",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        globals: gfx::ConstantBuffer<Globals> = "Globals",
        rect_properties: gfx::ConstantBuffer<RectProperties> = "RectProperties",
        out: gfx::BlendTarget<ColorFormat> =
          ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

impl Default for RectProperties {
    fn default() -> Self {
        RectProperties {
            src: [0.0, 0.0, 1.0, 1.0],
            dest: [0.0, 0.0],
            scale: [1.0, 1.0],
            offset: [0.0, 0.0],
            shear: [0.0, 0.0],
            rotation: 0.0,
        }
    }
}

impl From<DrawParam> for RectProperties {
    fn from(p: DrawParam) -> Self {
        RectProperties {
            src: p.src.into(),
            dest: p.dest.into(),
            scale: [p.scale.x, p.scale.y],
            offset: p.offset.into(),
            shear: p.shear.into(),
            rotation: p.rotation,
        }
    }
}


/// A structure that contains graphics state.
/// For instance, background and foreground colors.
///
/// It doesn't actually hold any of the SDL graphics stuff,
/// just info we need that SDL doesn't keep track of.
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
    white_image: Image,
    line_width: f32,
    point_size: f32,
    screen_rect: Rect,

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

// GL only
pub type GraphicsContext = GraphicsContextGeneric<gfx_device_gl::Resources,
                                                  gfx_device_gl::Factory,
                                                  gfx_device_gl::CommandBuffer,
                                                  gfx_device_gl::Device>;

/// TODO: This can probably be removed before release but might be
/// handy to keep around until then.  Just in case something else
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

impl GraphicsContext {
    pub fn new(video: sdl2::VideoSubsystem,
               window_title: &str,
               screen_width: u32,
               screen_height: u32)
               -> GameResult<GraphicsContext> {
        let gl = video.gl_attr();
        gl.set_context_version(GL_MAJOR_VERSION, GL_MINOR_VERSION);
        gl.set_context_profile(sdl2::video::GLProfile::Core);
        gl.set_red_size(5);
        gl.set_green_size(5);
        gl.set_blue_size(5);
        gl.set_alpha_size(8);

        let window_builder = video.window(window_title, screen_width, screen_height);
        let (window, gl_context, device, mut factory, color_view, depth_view) =
            gfx_window_sdl::init(window_builder)?;

        println!("Requested GL {}.{} Core profile, actually got GL {}.{} {:?} profile.",
                 GL_MAJOR_VERSION,
                 GL_MINOR_VERSION,
                 gl.context_major_version(),
                 gl.context_minor_version(),
                 gl.context_profile());

        let encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> =
            factory.create_command_buffer()
                .into();

        let pso = factory.create_pipeline_simple(include_bytes!("shader/basic_150.glslv"),
                                    include_bytes!("shader/basic_150.glslf"),
                                    pipe::new())?;

        let (quad_vertex_buffer, quad_slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTS, &QUAD_INDICES[..]);

        let rect_props = factory.create_constant_buffer(1);
        let globals_buffer = factory.create_constant_buffer(1);
        let sampler = factory.create_sampler_linear();
        let white_image = Image::make_raw(&mut factory, 1, 1, &[&[255, 255, 255, 255]])?;
        let texture = white_image.texture.clone();

        let data = pipe::Data {
            vbuf: quad_vertex_buffer.clone(),
            tex: (texture, sampler),
            rect_properties: rect_props,
            globals: globals_buffer,
            out: color_view,
        };

        // Set initial uniform values
        let left = 0.0;
        let right = screen_width as f32;
        let top = 0.0;
        let bottom = screen_height as f32;
        let globals = Globals {
            transform: ortho(left, right, top, bottom, 1.0, -1.0),
            color: types::WHITE.into(),
        };

        let mut gfx = GraphicsContext {
            background_color: Color::new(0.1, 0.2, 0.3, 1.0),
            shader_globals: globals,
            line_width: 1.0,
            point_size: 1.0,
            white_image: white_image,
            screen_rect: Rect::new(left, bottom, (right - left), (top - bottom)),

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
        };
        gfx.update_globals()?;
        Ok(gfx)
    }

    fn update_globals(&mut self) -> GameResult<()> {
        self.encoder.update_buffer(&self.data.globals, &[self.shader_globals], 0)?;
        Ok(())
    }

    fn update_rect_properties(&mut self, draw_params: DrawParam) -> GameResult<()> {
        let properties = draw_params.into();
        self.encoder.update_buffer(&self.data.rect_properties, &[properties], 0)?;
        Ok(())
    }
}


// fn gfx_load_texture<F, R>(factory: &mut F) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
//     where F: gfx::Factory<R>,
//           R: gfx::Resources
// {
//     use gfx::format::Rgba8;
//     let img = image::open("resources/player.png").unwrap().to_rgba();
//     let (width, height) = img.dimensions();
//     let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
//     let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[&img]).unwrap();
//     view
// }

/// Rather than create a dependency on cgmath or nalgebra for this one function,
/// we're just going to define it ourselves.
fn ortho(left: f32, right: f32, top: f32, bottom: f32, far: f32, near: f32) -> [[f32; 4]; 4] {
    let c0r0 = 2.0 / (right - left);
    let c0r1 = 0.0;
    let c0r2 = 0.0;
    let c0r3 = 0.0;

    let c1r0 = 0.0;
    let c1r1 = 2.0 / (top - bottom);
    let c1r2 = 0.0;
    let c1r3 = 0.0;

    let c2r0 = 0.0;
    let c2r1 = 0.0;
    let c2r2 = -2.0 / (far - near);
    let c2r3 = 0.0;

    let c3r0 = -(right + left) / (right - left);
    let c3r1 = -(top + bottom) / (top - bottom);
    let c3r2 = -(far + near) / (far - near);
    let c3r3 = 1.0;

    [[c0r0, c1r0, c2r0, c3r0],
     [c0r1, c1r1, c2r1, c3r1],
     [c0r2, c1r2, c2r2, c3r2],
     [c0r3, c1r3, c2r3, c3r3]]
}

// BUGGO: Has this been obsoleted by the Mesh type?
// fn draw_tessellated(gfx: &mut GraphicsContext, buffers: tessellation::Buffer) -> GameResult<()> {
//     let (buf, slice) = gfx.factory
//         .create_vertex_buffer_with_slice(&buffers.vertices[..], &buffers.indices[..]);

//     gfx.encoder.update_buffer(&gfx.data.rect_properties, &[RectProperties::default()], 0)?;

//     gfx.data.vbuf = buf;
//     gfx.data.tex.0 = gfx.white_image.texture.clone();

//     gfx.encoder.draw(&slice, &gfx.pso, &gfx.data);

//     Ok(())
// }

// **********************************************************************
// DRAWING
// **********************************************************************


/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    gfx.encoder.clear(&gfx.data.out, gfx.background_color.into());
}

/// Draws the given `Drawable` object to the screen.
///
/// * `ctx` - The `Context` this graphic will be rendered to.
/// * `drawable` - The `Drawable` to render.
/// * `dest` - the position to draw the graphic expressed as a `Point`.
/// * `rotation` - orientation of the graphic in radians.
///
pub fn draw(ctx: &mut Context, drawable: &Drawable, dest: Point, rotation: f32) -> GameResult<()> {
    drawable.draw(ctx, dest, rotation)
}


/// Draws the given `Drawable` object to the screen,
/// applying a rotation and mirroring if desired.
///
/// * `ctx` - The `Context` this graphic will be rendered to.
/// * `drawable` - The `Drawable` to render.
/// * `quad` - a portion of the drawable to clip.
/// * `dest` - the position to draw the graphic expressed as a `Point`.
/// * `rotation` - orientation of the graphic in radians.
/// * `scale` - x/y scale factors expressed as a `Point`.
/// * `offset` - used to move the pivot point for transform operations like scale/rotation.
/// * `shear` - x/y shear factors expressed as a `Point`.
///
// #[allow(too_many_arguments)]
pub fn draw_ex(ctx: &mut Context, drawable: &Drawable, params: DrawParam) -> GameResult<()> {
    drawable.draw_ex(ctx, params)
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your `GameState`'s `draw()` method.
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

/// Draw an arc.
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

/// Draw a circle.
pub fn circle(ctx: &mut Context,
              mode: DrawMode,
              point: Point,
              radius: f32,
              segments: u32)
              -> GameResult<()> {
    let m = Mesh::new_circle(ctx, mode, point, radius, segments)?;
    m.draw(ctx, Point::default(), 0.0)
}

/// Draw an ellipse.
pub fn ellipse(ctx: &mut Context,
               mode: DrawMode,
               point: Point,
               radius1: f32,
               radius2: f32,
               segments: u32)
               -> GameResult<()> {
    let m = Mesh::new_ellipse(ctx, mode, point, radius1, radius2, segments)?;
    m.draw(ctx, Point::default(), 0.0)
}

/// Draws a line of one or more connected segments.
pub fn line(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    let w = ctx.gfx_context.line_width;
    let m = Mesh::new_line(ctx, points, w)?;
    m.draw(ctx, Point::default(), 0.0)
}

/// Draws points.
pub fn points(_ctx: &mut Context, _point: &[Point]) -> GameResult<()> {
    unimplemented!();
    // let r = &mut ctx.renderer;
    // let res = r.draw_point(point);
    // res.map_err(GameError::from)
}

/// Draws a closed polygon
pub fn polygon(ctx: &mut Context, mode: DrawMode, vertices: &[Point]) -> GameResult<()> {
    match mode {
        DrawMode::Line => {
            // We append the first vertex to the list as the last vertex,
            // and just draw it with line()
            let mut pts = Vec::with_capacity(vertices.len() + 1);
            pts.extend(vertices);
            pts.push(vertices[0]);
            line(ctx, &pts)
        }
        DrawMode::Fill => unimplemented!(),
    }
}

/// Renders text with the default font.
pub fn print(_ctx: &mut Context, _dest: Point, _text: &str, _size: f32) {
    unimplemented!();
}


/// Draws a rectangle.
pub fn rectangle(ctx: &mut Context, mode: DrawMode, rect: Rect) -> GameResult<()> {
    // TODO: See if we can evade this clone() without a double-borrow being involved?
    // That might actually be invalid considering that drawing an Image involves altering
    // its state.  And it might just be cloning a texture handle.
    // TODO: Draw mode is unimplemented.
    match mode {
        DrawMode::Fill => {
            let img = &mut ctx.gfx_context.white_image.clone();
            let source = Rect::new(0.0, 0.0, 1.0, 1.0);
            let dest = Point::new(rect.x, rect.y);
            let scale = Point::new(rect.w, rect.h);
            draw_ex(ctx,
                    img,
                    DrawParam {
                        src: source,
                        dest: dest,
                        scale: scale,
                        ..Default::default()
                    })
        }
        DrawMode::Line => unimplemented!(),
    }
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Returns the current background color.
pub fn get_background_color(ctx: &Context) -> Color {
    ctx.gfx_context.background_color
}

/// Returns thec urrent foreground color.
pub fn get_color(ctx: &Context) -> Color {
    ctx.gfx_context.shader_globals.color.into()
}

pub fn get_default_filter(ctx: &Context) -> FilterMode {
    let gfx = &ctx.gfx_context;
    let sampler_info = gfx.data.tex.1.get_info();
    sampler_info.filter.into()
}


pub fn get_line_width(ctx: &Context) -> f32 {
    ctx.gfx_context.line_width
}

pub fn get_point_size(ctx: &Context) -> f32 {
    ctx.gfx_context.point_size
}

pub fn get_renderer_info(_ctx: &Context) {
    unimplemented!()
}

// TODO: Better name.  screen_bounds?  Viewport?
pub fn get_screen_coordinates(ctx: &Context) -> Rect {
    ctx.gfx_context.screen_rect
}

// TOOD: Verify!
pub fn is_gamma_correct(_ctx: &Context) -> bool {
    true
}

/// Sets the background color.  Default: blue.
pub fn set_background_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.background_color = color;
}

/// Sets the foreground color, which will be used for drawing
/// rectangles, lines, etc.  Default: white.
pub fn set_color(ctx: &mut Context, color: Color) -> GameResult<()> {
    // TODO: Update buffer!
    let gfx = &mut ctx.gfx_context;
    gfx.shader_globals.color = color.into();
    gfx.update_globals()
}

/// Sets the default filter mode used to scale images.
pub fn set_default_filter(ctx: &mut Context, mode: FilterMode) {
    let gfx = &mut ctx.gfx_context;
    let new_mode = mode.into();
    let sampler_info = texture::SamplerInfo::new(new_mode, texture::WrapMode::Clamp);
    let new_sampler = gfx.factory.create_sampler(sampler_info);

    gfx.data.tex.1 = new_sampler;
}

pub fn set_line_width(ctx: &mut Context, width: f32) {
    ctx.gfx_context.line_width = width;
}

pub fn set_point_size(ctx: &mut Context, size: f32) {
    ctx.gfx_context.point_size = size;
}

pub fn set_screen_coordinates(context: &mut Context,
                              left: f32,
                              right: f32,
                              top: f32,
                              bottom: f32)
                              -> GameResult<()> {
    let gfx = &mut context.gfx_context;
    gfx.screen_rect = Rect::new(left, bottom, (right - left), (top - bottom));
    gfx.shader_globals.transform = ortho(left, right, top, bottom, 1.0, -1.0);
    gfx.update_globals()
}

// **********************************************************************
// TYPES
// **********************************************************************


/// A struct containing all the necessary info for drawing a Drawable.
///
/// * `src` - a portion of the drawable to clip.
/// * `dest` - the position to draw the graphic expressed as a `Point`.
/// * `rotation` - orientation of the graphic in radians.
/// * `scale` - x/y scale factors expressed as a `Point`.
/// * `offset` - used to move the pivot point for transform operations like scale/rotation.
/// * `shear` - x/y shear factors expressed as a `Point`.
///
/// It implements the `Default` trait, so you can just do:
///
/// `draw(drawable, DrawParam{ dest: my_dest, .. Default::default()} )`
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
            dest: Point::zero(),
            rotation: 0.0,
            scale: Point::new(1.0, 1.0),
            offset: Point::new(0.0, 0.0),
            shear: Point::new(0.0, 0.0),
        }
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

/// In-memory image data available to be drawn on the screen.
#[derive(Clone)]
pub struct ImageGeneric<R>
    where R: gfx::Resources
{
    // We should probably keep both the raw image data around,
    // and an Option containing the texture handle if necessary.
    texture: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    width: u32,
    height: u32,
}

pub type Image = ImageGeneric<gfx_device_gl::Resources>;

impl Image {
    /// Load a new image from the file at the given path.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Image> {
        let img = {
            let mut buf = Vec::new();
            let mut reader = context.filesystem.open(path)?;
            // TODO:
            // image's guess_format() stuff is inconvenient.
            // It would be nicer if they just read the first n
            // bytes from the reader, but noooooo...
            // Even though they require a BufReader anyway.
            reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_rgba()
        };
        let (width, height) = img.dimensions();
        Image::from_rgba8(context, width as u16, height as u16, &[&img])
    }

    /// A helper function that just takes a factory directly so we can make an image
    /// without needing the full context object, so we can create an Image while still
    /// creating the GraphicsContext.
    fn make_raw(factory: &mut gfx_device_gl::Factory,
                width: u16,
                height: u16,
                rgba: &[&[u8]])
                -> GameResult<Image> {
        if !(width.is_power_of_two() && height.is_power_of_two()) {
            let w2 = width.next_power_of_two();
            let h2 = height.next_power_of_two();
            let msg = format!("Needed power of 2 texture, got {}x{} (try making it {}x{}",
                              width,
                              height,
                              w2,
                              h2);
            return Err(GameError::ResourceLoadError(msg));
        }
        let kind = gfx::texture::Kind::D2(width, height, gfx::texture::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &rgba)?;
        Ok(Image {
            texture: view,
            width: width as u32,
            height: height as u32,
        })
    }

    /// Creates an Image from an array of u8's arranged in RGBA order.
    pub fn from_rgba8(context: &mut Context,
                      width: u16,
                      height: u16,
                      rgba: &[&[u8]])
                      -> GameResult<Image> {
        Image::make_raw(&mut context.gfx_context.factory, width, height, rgba)
    }

    pub fn from_rgba8_flat(context: &mut Context,
                           width: u16,
                           height: u16,
                           rgba: &[u8])
                           -> GameResult<Image> {
        let uheight = height as usize;
        let uwidth = width as usize;
        let mut buffer = Vec::with_capacity(uheight);
        for i in 0..uheight {
            buffer.push(&rgba[i..i * uwidth]);
        }
        Image::from_rgba8(context, width, height, &buffer)
    }


    /// A little helper function that creates a new Image that is just
    /// a solid square of the given size and color.  Mainly useful for
    /// debugging.
    pub fn solid(context: &mut Context, size: u16, color: Color) -> GameResult<Image> {
        let pixel_array: [u8; 4] = color.into();
        let size_squared = size as usize * size as usize;
        let mut buffer = Vec::with_capacity(size_squared);
        for _i in 0..size_squared {
            buffer.push(&pixel_array[..]);
        }
        Image::from_rgba8(context, size, size, &buffer[..])
    }

    /// Return the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Return the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_filter(&self) {
        unimplemented!()
    }

    pub fn set_filter(&self) {
        unimplemented!()
    }

    /// Returns the dimensions of the image.
    pub fn get_dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
    }

    pub fn get_wrap(&self) {
        unimplemented!()
    }

    pub fn set_wrap(&self) {
        unimplemented!()
    }
}


impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "<Image: {}x{}, {:p}, texture address {:p}>",
               self.width(),
               self.height(),
               self,
               &self.texture)
    }
}


impl Drawable for Image {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        let src_width = param.src.w;
        let src_height = param.src.h;
        let real_scale = Point {
            x: src_width * param.scale.x * self.width as f32,
            y: src_height * param.scale.y * self.height as f32,
        };
        let mut new_param = param;
        new_param.scale = real_scale;
        // Not entirely sure why the inversion is necessary, but oh well.
        new_param.offset.x *= -1.0 * param.scale.x;
        new_param.offset.y *= param.scale.y;
        gfx.update_rect_properties(new_param)?;
        // TODO: BUGGO: Make sure these clones are cheap; they should be.
        let (_, sampler) = gfx.data.tex.clone();
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        gfx.data.tex = (self.texture.clone(), sampler);
        gfx.encoder.draw(&gfx.quad_slice, &gfx.pso, &gfx.data);
        Ok(())
    }
}

/// 2D polygon mesh
pub struct Mesh {
    buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
}

impl Mesh {
    fn from_tessellation(ctx: &mut Context, buffer: tessellation::Buffer) -> GameResult<Mesh> {
        let (vbuf, slice) = ctx.gfx_context
            .factory
            .create_vertex_buffer_with_slice(&buffer.vertices[..], &buffer.indices[..]);

        Ok(Mesh {
            buffer: vbuf,
            slice: slice,
        })
    }

    /// Create a new mesh for a line of one or more connected segments.
    pub fn new_line(ctx: &mut Context, points: &[Point], width: f32) -> GameResult<Mesh> {
        Mesh::from_tessellation(ctx, tessellation::build_line(points, width)?)
    }

    /// Create a new mesh for a circle.
    pub fn new_circle(ctx: &mut Context,
                      mode: DrawMode,
                      point: Point,
                      radius: f32,
                      segments: u32)
                      -> GameResult<Mesh> {
        let buf = match mode {
            DrawMode::Fill => tessellation::build_ellipse_fill(point, radius, radius, segments),
            DrawMode::Line => unimplemented!(),
        }?;

        Mesh::from_tessellation(ctx, buf)
    }

    /// Create a new mesh for an ellipse.
    pub fn new_ellipse(ctx: &mut Context,
                       mode: DrawMode,
                       point: Point,
                       radius1: f32,
                       radius2: f32,
                       segments: u32)
                       -> GameResult<Mesh> {
        let buf = match mode {
            DrawMode::Fill => tessellation::build_ellipse_fill(point, radius1, radius2, segments),
            DrawMode::Line => unimplemented!(),
        }?;

        Mesh::from_tessellation(ctx, buf)
    }
}

impl Drawable for Mesh {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        gfx.update_rect_properties(param)?;

        gfx.data.vbuf = self.buffer.clone();
        gfx.data.tex.0 = gfx.white_image.texture.clone();

        gfx.encoder.draw(&self.slice, &gfx.pso, &gfx.data);

        Ok(())
    }
}
