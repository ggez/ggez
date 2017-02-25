//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text.
//!
//! This module also manages graphics state, coordinate systems, etc.
//! The default coordinate system has the origin in the upper-left
//! corner of the screen.

use std::fmt;
use std::path;
use std::collections::BTreeMap;
use std::convert::From;
use std::io::Read;

use sdl2;
use rusttype;
use image;
use gfx;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_sdl;
use gfx::format::Rgba8;
use gfx::Factory;


use context::Context;
use GameError;
use GameResult;

mod types;
pub use self::types::{Rect, Point, Color, BlendMode};

const GL_MAJOR_VERSION: u8 = 3;
const GL_MINOR_VERSION: u8 = 2;

/// Specifies whether a shape should be drawn
/// filled or as an outline.
#[derive(Debug)]
pub enum DrawMode {
    Line,
    Fill,
}


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
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

impl Default for RectProperties {
    fn default() -> Self {
        RectProperties {
            src: [0.0, 0.0, 0.0, 0.0],
            dest: [0.0, 0.0],
            scale: [1.0, 1.0],
            offset: [0.0, 0.0],
            shear: [0.0, 0.0],
            rotation: 0.0,
        }
    }
}


// BUGGO: TODO: Impl Debug for GraphicsContext

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
    blend_mode: (),
    line_width: f32,
    point_size: f32,

    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    device: Box<D>,
    factory: Box<F>,
    encoder: gfx::Encoder<R, C>,
    // color_view: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
    depth_view: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,

    pso: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
    quad_slice: gfx::Slice<R>,
}

// GL only
pub type GraphicsContext = GraphicsContextGeneric<gfx_device_gl::Resources,
                                                  gfx_device_gl::Factory,
                                                  gfx_device_gl::CommandBuffer,
                                                  gfx_device_gl::Device>;

/// TODO: This can probably be removed before release but might be
/// handy to keep around until then.  Just in case something else
/// crazy happens.
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
        let (mut window, mut gl_context, mut device, mut factory, color_view, depth_view) =
            gfx_window_sdl::init(window_builder).unwrap();

        println!("Requested GL {}.{} Core profile, actually got GL {}.{} {:?} profile.",
                 GL_MAJOR_VERSION,
                 GL_MINOR_VERSION,
                 gl.context_major_version(),
                 gl.context_minor_version(),
                 gl.context_profile());

        let mut encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> =
            factory.create_command_buffer()
                .into();

        let pso = factory.create_pipeline_simple(include_bytes!("shader/basic_150.glslv"),
                                    include_bytes!("shader/basic_150.glslf"),
                                    pipe::new())
            .unwrap();

        let (quad_vertex_buffer, quad_slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTS, &QUAD_INDICES[..]);

        let rect_props = factory.create_constant_buffer(1);
        let globals_buffer = factory.create_constant_buffer(1);
        let sampler = factory.create_sampler_linear();
        let white_image = Image::make_raw(&mut factory, 1, 1, &[&[255, 255, 255, 255]]).unwrap();
        let texture = white_image.texture.clone();
        let data = pipe::Data {
            vbuf: quad_vertex_buffer,
            tex: (texture, sampler),
            rect_properties: rect_props,
            globals: globals_buffer,
            out: color_view,
        };


        // Set initial uniform values
        let globals = Globals {
            transform: ortho(0.0,
                             screen_width as f32,
                             0.0,
                             screen_height as f32,
                             1.0,
                             -1.0),
            // color: types::WHITE.into(),
            color: [1.0, 1.0, 1.0, 0.5],
        };

        let mut gfx = GraphicsContext {
            background_color: Color::new(0.1, 0.2, 0.3, 1.0),
            shader_globals: globals,
            line_width: 1.0,
            point_size: 1.0,
            blend_mode: (),
            white_image: white_image,

            window: window,
            gl_context: gl_context,
            device: Box::new(device),
            factory: Box::new(factory),
            encoder: encoder,
            depth_view: depth_view,

            pso: pso,
            data: data,
            quad_slice: quad_slice,
        };
        gfx.update_globals();
        Ok(gfx)
    }

    fn update_globals(&mut self) {
        self.encoder.update_buffer(&self.data.globals, &[self.shader_globals], 0);
    }
}


fn gfx_load_texture<F, R>(factory: &mut F) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    use gfx::format::Rgba8;
    let img = image::open("resources/player.png").unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[&img]).unwrap();
    view
}

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
/// * `quad` - a portion of the drawable to clip.
/// * `dest` - the position to draw the graphic expressed as a `Point`.
/// * `rotation` - orientation of the graphic in radians.
///
pub fn draw(ctx: &mut Context,
            drawable: &mut Drawable,
            quad: Rect,
            dest: Point,
            rotation: f32)
            -> GameResult<()> {
    drawable.draw(ctx, quad, dest, rotation)
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
pub fn draw_ex(ctx: &mut Context,
               drawable: &mut Drawable,
               quad: Rect,
               dest: Point,
               rotation: f32,
               scale: Point,
               offset: Point,
               shear: Point)
               -> GameResult<()> {
    drawable.draw_ex(ctx, quad, dest, rotation, scale, offset, shear)
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
}

pub fn arc(ctx: &mut Context,
           mode: DrawMode,
           point: Point,
           radius: f32,
           angle1: f32,
           angle2: f32,
           segments: u32)
           -> GameResult<()> {
    unimplemented!();
}

pub fn circle(ctx: &mut Context,
              mode: DrawMode,
              point: Point,
              radius: f32,
              segments: u32)
              -> GameResult<()> {
    unimplemented!();
}

pub fn ellipse(ctx: &mut Context,
               mode: DrawMode,
               point: Point,
               radius1: f32,
               radius2: f32,
               segments: u32)
               -> GameResult<()> {
    unimplemented!();
}

/// Draws a line of one or more connected segments.
pub fn line(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    unimplemented!();
}

/// Draws points.
pub fn points(ctx: &mut Context, point: &[Point]) -> GameResult<()> {
    unimplemented!();
    // let r = &mut ctx.renderer;
    // let res = r.draw_point(point);
    // res.map_err(GameError::from)
}

pub fn polygon(ctx: &mut Context, mode: DrawMode, vertices: &[Point]) -> GameResult<()> {
    unimplemented!();
}

/// Not implemented
pub fn print(_ctx: &mut Context) {
    unimplemented!();
}

/// Not implemented
pub fn printf(_ctx: &mut Context) {
    unimplemented!();
}


/// Draws a rectangle.
pub fn rectangle(ctx: &mut Context, mode: DrawMode, rect: Rect) -> GameResult<()> {
    // TODO: See if we can evade this clone() without a double-borrow being involved?
    // That might actually be invalid considering that drawing an Image involves altering
    // its state.
    // TODO: Draw mode is unimplemented.
    let img = &mut ctx.gfx_context.white_image.clone();
    let source = Rect::new(0.0, 0.0, 1.0, 1.0);
    let dest = Point::new(rect.x, rect.y);
    let scale = Point::new(rect.w, rect.h);
    draw_ex(ctx,
            img,
            source,
            dest,
            0.0,
            scale,
            Point::zero(),
            Point::zero())
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

pub fn get_background_color(ctx: &Context) -> Color {
    ctx.gfx_context.background_color
}

pub fn get_blend_mode(ctx: &Context) {
    unimplemented!()
}

pub fn get_color(ctx: &Context) -> Color {
    ctx.gfx_context.shader_globals.color.into()
}

pub fn get_default_filter(ctx: &Context) {
    unimplemented!()
}

pub fn get_font(ctx: &Context) -> Font {
    unimplemented!()
}

pub fn get_line_width(ctx: &Context) -> f32 {
    unimplemented!()
}

pub fn get_point_size(ctx: &Context) -> f32 {
    unimplemented!()
}

pub fn get_renderer_info(ctx: &Context) {
    unimplemented!()
}

// TODO: Better name.  screen_bounds?  Viewport?
pub fn get_screen_coordinates(ctx: &Context) {
    unimplemented!()
}

pub fn is_gamma_correct(ctx: &Context) -> bool {
    unimplemented!()
}

/// Sets the background color.  Default: blue.
pub fn set_background_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.background_color = color;
}

pub fn set_blend_mode(ctx: &mut Context) {
    unimplemented!()
}

/// Sets the foreground color, which will be used for drawing
/// rectangles, lines, etc.  Default: white.
pub fn set_color(ctx: &mut Context, color: Color) {
    // TODO: Update buffer!
    let gfx = &mut ctx.gfx_context;
    gfx.shader_globals.color = color.into();
    gfx.update_globals();
    // gfx.encoder.update_buffer(&gfx.data.globals, &[gfx.shader_globals], 0);
}

pub fn set_default_filter(ctx: &mut Context) {
    unimplemented!()
}


pub fn set_font(ctx: &mut Context, font: Font) {
    unimplemented!()
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
                              bottom: f32) {
    let gfx = &mut context.gfx_context;
    gfx.shader_globals.transform = ortho(left, right, top, bottom, 1.0, -1.0);
    gfx.update_globals();
    // gfx.encoder.update_buffer(&gfx.data.globals, &[gfx.shader_globals], 0);
}

// **********************************************************************
// TYPES
// **********************************************************************

/// All types that can be drawn on the screen implement the `Drawable` trait.
pub trait Drawable {
    /// Actually draws the object to the screen.
    ///
    /// This is the most general version of the operation, which is all that
    /// is required for implementing this trait.
    /// (It also maps nicely onto SDL2's Renderer::copy_ex(), we might want to
    /// wrap the types up a bit more nicely someday.)
    ///
    /// * `ctx` - The `Context` this graphic will be rendered to.
    /// * `quad` - a portion of the drawable to clip.
    /// * `dest` - the position to draw the graphic expressed as a `Point`.
    /// * `rotation` - orientation of the graphic in radians.
    /// * `scale` - x/y scale factors expressed as a `Point`.
    /// * `offset` - used to move the pivot point for transform operations like scale/rotation.
    /// * `shear` - x/y shear factors expressed as a `Point`.
    ///
    // #[allow(too_many_arguments)]
    fn draw_ex(&mut self,
               ctx: &mut Context,
               quad: Rect,
               dest: Point,
               rotation: f32,
               scale: Point,
               offset: Point,
               shear: Point)
               -> GameResult<()>;

    /// Draws the drawable onto the rendering target.
    ///
    /// * `ctx` - The `Context` this graphic will be rendered to.
    /// * `quad` - a portion of the drawable to clip.
    /// * `dest` - the position to draw the graphic expressed as a `Point`.
    /// * `rotation` - orientation of the graphic in radians.
    ///
    fn draw(&mut self,
            ctx: &mut Context,
            quad: Rect,
            dest: Point,
            rotation: f32)
            -> GameResult<()> {
        self.draw_ex(ctx,
                     quad,
                     dest,
                     rotation,
                     Point::new(1.0, 1.0),
                     Point::new(0.0, 0.0),
                     Point::new(0.0, 0.0))
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
    properties: RectProperties,
    width: u32,
    height: u32,
}

pub type Image = ImageGeneric<gfx_device_gl::Resources>;

impl Image {
    /// Load a new image from the file at the given path.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Image> {
        let img = image::open(path).unwrap().to_rgba();
        let (width, height) = img.dimensions();
        Image::from_rgba8(context, width as u16, height as u16, &[&img])
    }

    /// A helper function that just takes a factory directly so we can make an image
    /// without needing the full context object, so we can create one inside the context
    /// object
    fn make_raw(factory: &mut gfx_device_gl::Factory,
                width: u16,
                height: u16,
                rgba: &[&[u8]])
                -> GameResult<Image> {
        let kind = gfx::texture::Kind::D2(width, height, gfx::texture::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &rgba).unwrap();
        Ok(Image {
            texture: view,
            width: width as u32,
            height: height as u32,
            properties: RectProperties::default(),
        })
    }

    /// Creates an Image from an array of u8's arranged in RGBA order.
    /// TODO: Refactor the from_* functions, make_raw, and new() to
    /// be a little more orthogonal.  Also see Love2D's ImageData type.
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
        for i in 0..size_squared {
            buffer.push(&pixel_array[..]);
        }
        Image::from_rgba8(context, size, size, &buffer[..])
    }

    /// Returns the dimensions of the image.
    pub fn rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
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

    pub fn get_dimensions(&self) {
        unimplemented!()
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
    // #[allow(too_many_arguments)]
    fn draw_ex(&mut self,
               context: &mut Context,
               quad: Rect,
               dest: Point,
               rotation: f32,
               scale: Point,
               offset: Point,
               shear: Point)
               -> GameResult<()> {

        let gfx = &mut context.gfx_context;
        self.properties.dest = dest.into();
        self.properties.scale = [scale.x * self.width as f32, scale.y * self.height as f32];
        gfx.encoder.update_buffer(&gfx.data.rect_properties, &[self.properties], 0);

        // let transform = Transform { transform: ortho(-1.5, 1.5, 1.0, -1.0, 1.0, -1.0) };
        // gfx.encoder.update_buffer(&gfx.data.transform, &[transform], 0);
        // TODO: BUGGO: Make sure these clones are cheap; they should be.
        let (_, sampler) = gfx.data.tex.clone();
        gfx.data.tex = (self.texture.clone(), sampler);
        gfx.encoder.draw(&gfx.quad_slice, &gfx.pso, &gfx.data);
        Ok(())
    }
}

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image (bitmap fonts).
pub enum Font {
    TTFFont {
        font: rusttype::Font<'static>,
        points: u32,
    },
    BitmapFont {
        surface: Image,
        glyphs: BTreeMap<char, u32>,
        glyph_width: u32,
    },
}

impl Font {
    /// Load a new TTF font from the given file.
    pub fn new<P>(context: &mut Context, path: P, size: u32) -> GameResult<Font>
        where P: AsRef<path::Path> + fmt::Debug
    {
        // let mut buffer: Vec<u8> = Vec::new();
        // let mut rwops = util::rwops_from_path(context, path, &mut buffer)?;
        let mut stream = context.filesystem.open(path.as_ref())?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;

        let name = format!("{:?}", path);
        Font::font_from_bytes(&name, buf, size)
    }

    fn font_from_bytes<B>(name: &str, bytes: B, size: u32) -> GameResult<Font>
        where B: Into<rusttype::SharedBytes<'static>>
    {
        let collection = rusttype::FontCollection::from_bytes(bytes);
        let font_err = GameError::ResourceLoadError(format!("Could not load font collection for \
                                                             font {:?}",
                                                            name));
        let font = collection.into_font().ok_or(font_err)?;

        Ok(Font::TTFFont {
            font: font,
            points: size,
        })
    }

    /// Loads an `Image` and uses it to create a new bitmap font
    /// The `Image` is a 1D list of glyphs, which maybe isn't
    /// super ideal but should be fine.
    /// The `glyphs` string is the characters in the image from left to right.
    pub fn new_bitmap<P: AsRef<path::Path>>(context: &mut Context,
                                            path: P,
                                            glyphs: &str)
                                            -> GameResult<Font> {
        let s2 = Image::new(context, path.as_ref())?;

        let image_width = s2.width();
        let glyph_width = image_width / (glyphs.len() as u32);
        // println!("Number of glyphs: {}, Glyph width: {}, image width: {}",
        // glyphs.len(), glyph_width, image_width);
        let mut glyphs_map: BTreeMap<char, u32> = BTreeMap::new();
        for (i, c) in glyphs.chars().enumerate() {
            let small_i = i as u32;
            glyphs_map.insert(c, small_i * glyph_width);
        }
        Ok(Font::BitmapFont {
            surface: s2,

            glyphs: glyphs_map,
            glyph_width: glyph_width,
        })
    }

    pub fn default_font() -> GameResult<Self> {
        let size = 16;
        let buf = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/DejaVuSerif.ttf"));
        Font::font_from_bytes("default", &buf[..], size)
    }
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Font::TTFFont { .. } => write!(f, "<TTFFont: {:p}>", &self),
            Font::BitmapFont { .. } => write!(f, "<BitmapFont: {:p}>", &self),
        }
    }
}

/// Drawable text created from a `Font`.
pub struct Text {
    texture: Image,
    // texture_query: render::TextureQuery,
    contents: String,
}

/// Compute a scale for a font of a given size.
// This basically does the points->pixels unit conversion,
// taking the display DPI into account.
fn display_independent_scale(points: u32, dpi_w: f32, dpi_h: f32) -> rusttype::Scale {
    // Calculate pixels per point
    let points = points as f32;
    let points_per_inch = 72.0;
    let pixels_per_point_w = dpi_w * (1.0 / points_per_inch);
    let pixels_per_point_h = dpi_h * (1.0 / points_per_inch);

    // rusttype::Scale is in units of pixels, so.
    let scale = rusttype::Scale {
        x: pixels_per_point_w * points,
        y: pixels_per_point_h * points,
    };
    scale
}

fn render_ttf(context: &mut Context,
              text: &str,
              font: &rusttype::Font<'static>,
              size: u32)
              -> GameResult<Text> {
    // Ripped almost wholesale from
    // https://github.com/dylanede/rusttype/blob/master/examples/simple.rs

    // TODO: Figure out
    // Also, 72 DPI is default but might not always be valid; 4K screens etc
    // SDL has a way to get the proper DPI.

    // let size: f32 = 24.0;
    // let pixel_height = size.ceil() as usize;
    let (_diag_dpi, x_dpi, y_dpi) = context.dpi;
    let scale = display_independent_scale(size, x_dpi, y_dpi);
    let pixel_height = scale.y.ceil() as usize;
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);
    // Then turn them into an array of positioned glyphs...
    // `layout()` turns an abstract glyph, which contains no concrete
    // size or position information, into a PositionedGlyph, which does.
    let glyphs: Vec<rusttype::PositionedGlyph> = font.layout(text, scale, offset).collect();
    let width = glyphs.iter()
        .rev()
        .filter_map(|g| {
            g.pixel_bounding_box()
                .map(|b| b.min.x as f32 + g.unpositioned().h_metrics().advance_width)
        })
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;
    // Make an array for our rendered bitmap
    let bytes_per_pixel = 4;
    let mut pixel_data = vec![0; width * pixel_height * bytes_per_pixel];
    let pitch = width * bytes_per_pixel;

    // Now we actually render the glyphs to a bitmap...
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            // v is the amount of the pixel covered
            // by the glyph, in the range 0.0 to 1.0
            g.draw(|x, y, v| {
                let c = (v * 255.0) as u8;
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                    let x = x as usize * bytes_per_pixel;
                    let y = y as usize;
                    pixel_data[(x + y * pitch + 0)] = c;
                    pixel_data[(x + y * pitch + 1)] = c;
                    pixel_data[(x + y * pitch + 2)] = c;
                    pixel_data[(x + y * pitch + 3)] = c;
                }
            })
        }
    }

    // Copy the bitmap onto a surface, and we're basically done!
    // BUGGO: TODO: Make sure conversions will not fail
    let image = Image::from_rgba8_flat(context, width as u16, pixel_height as u16, &pixel_data)?;
    // let format = pixels::PixelFormatEnum::RGBA8888;
    // let surface = try!(surface::Surface::from_data(&mut pixel_data,
    //                                               width as u32,
    //                                               pixel_height as u32,
    //                                               pitch as u32,
    //                                               format));

    // let image = Image::from_surface(context, surface)?;
    // let tq = image.texture.query();

    let text_string = text.to_string();
    Ok(Text {
        texture: image,
        contents: text_string,
    })

}

fn render_bitmap(context: &Context,
                 text: &str,
                 image: &Image,
                 glyphs_map: &BTreeMap<char, u32>,
                 glyph_width: u32)
                 -> GameResult<Text> {
    let text_length = text.len() as u32;
    let glyph_height = image.height;
    // let format = pixels::PixelFormatEnum::RGBA8888;
    // let mut dest_surface = surface::Surface::new(text_length * glyph_width, glyph_height, format)?;
    // for (i, c) in text.chars().enumerate() {
    //     let small_i = i as u32;
    //     let error_message = format!("Character '{}' not in bitmap font!", c);
    //     let source_offset = glyphs_map.get(&c)
    //         .ok_or(GameError::FontError(String::from(error_message)))?;
    //     let dest_offset = glyph_width * small_i;
    //     let source_rect = Rect::new(*source_offset as f32,
    //                                 0.0,
    //                                 glyph_width as f32,
    //                                 glyph_height as f32);
    //     let dest_rect = Rect::new(dest_offset as f32,
    //                               0.0,
    //                               glyph_width as f32,
    //                               glyph_height as f32);
    //     // println!("Blitting letter {} to {:?}", c, dest_rect);
    //     // surface.blit(Some(source_rect), &mut dest_surface, Some(dest_rect))?;
    // }
    // // let image = Image::from_surface(context, dest_surface)?;
    let text_string = text.to_string();

    unimplemented!();
    // let tq = image.texture.query();
    // Ok(Text {
    //     texture: image.texture,
    //     texture_query: tq,
    //     contents: text_string,
    // })
}


impl Text {
    /// Renders a new `Text` from the given `Font`
    pub fn new(context: &mut Context, text: &str, font: &Font) -> GameResult<Text> {
        match *font {
            Font::TTFFont { font: ref f, points } => render_ttf(context, text, f, points),
            Font::BitmapFont { ref surface, glyph_width, glyphs: ref glyphs_map, .. } => {
                render_bitmap(context, text, surface, glyphs_map, glyph_width)
            }
        }
    }

    /// Returns the width of the rendered text, in pixels.
    pub fn width(&self) -> u32 {
        self.texture.width
    }

    /// Returns the height of the rendered text, in pixels.
    pub fn height(&self) -> u32 {
        self.texture.height
    }

    /// Returns the string that the text represents.
    pub fn contents(&self) -> &str {
        &self.contents
    }
}


impl Drawable for Text {
    // #[allow(too_many_arguments)]
    fn draw_ex(&mut self,
               context: &mut Context,
               quad: Rect,
               dest: Point,
               rotation: f32,
               scale: Point,
               offset: Point,
               shear: Point)
               -> GameResult<()> {
        unimplemented!();
        // let renderer = &mut context.renderer;
        // renderer.copy_ex(&self.texture,
        //              src,
        //              dst,
        //              angle,
        //              center,
        //              flip_horizontal,
        //              flip_vertical)
        //     .map_err(GameError::RenderError)
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "<Text: {}x{}, {:p}>",
               self.texture.width,
               self.texture.height,
               &self)

    }
}
