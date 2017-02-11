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
use sdl2::pixels;
use sdl2::render;
use sdl2::surface;
use sdl2::image::ImageRWops;
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
use util;

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

// This is all placeholder for now just to get us going.
// const TRIANGLE: [Vertex; 3] =
//     [Vertex { pos: [-0.5, -0.5] }, Vertex { pos: [0.5, -0.5] }, Vertex { pos: [0.0, 0.5] }];

const QUAD_VERTS: [Vertex; 4] = [Vertex {
                                     pos: [-0.5, -0.5],
                                     uv: [0.0, 1.0],
                                 },
                                 Vertex {
                                     pos: [0.5, -0.5],
                                     uv: [1.0, 1.0],
                                 },
                                 Vertex {
                                     pos: [0.5, 0.5],
                                     uv: [1.0, 0.0],
                                 },
                                 Vertex {
                                     pos: [-0.5, 0.5],
                                     uv: [0.0, 0.0],
                                 }];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Transform {
        transform: [[f32; 4];4] = "u_Transform",
    }

    // Values that are different for each rect.
    constant RectProperties {
        offset: [f32; 2] = "u_Offset",
        size: [f32; 2] = "u_Size",
        color_mod: [f32; 4] = "u_ColorMod",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        rect_properties: gfx::ConstantBuffer<RectProperties> = "RectProperties",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

impl Default for RectProperties {
    fn default() -> Self {
        RectProperties {
            offset: [0.0, 0.0],
            size: [1.0, 1.0],
            color_mod: types::WHITE.into(),
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
    foreground_color: Color,

    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    device: Box<D>,
    factory: Box<F>,
    encoder: gfx::Encoder<R, C>,
    // color_view: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
    depth_view: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,

    pso: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
    // slice: gfx::Slice<R>,
    quad_slice: gfx::Slice<R>,
}

// GL only
pub type GraphicsContext = GraphicsContextGeneric<gfx_device_gl::Resources,
                                                  gfx_device_gl::Factory,
                                                  gfx_device_gl::CommandBuffer,
                                                  gfx_device_gl::Device>;

impl GraphicsContext {
    pub fn new(video: sdl2::VideoSubsystem,
               window_title: &str,
               screen_width: u32,
               screen_height: u32)
               -> GameResult<GraphicsContext> {

        let window_builder = video.window(window_title, screen_width, screen_height);
        let (mut window, mut gl_context, mut device, mut factory, color_view, depth_view) =
            gfx_window_sdl::init(window_builder).unwrap();


        let gl = video.gl_attr();
        gl.set_context_version(GL_MAJOR_VERSION, GL_MINOR_VERSION);
        gl.set_context_profile(sdl2::video::GLProfile::Core);
        println!("Requested GL {}.{} Core profile, actually got GL {}.{} {:?} profile.",
                 GL_MAJOR_VERSION,
                 GL_MINOR_VERSION,
                 gl.context_major_version(),
                 gl.context_minor_version(),
                 gl.context_profile());

        let mut encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> =
            factory.create_command_buffer()
                .into();

        let pso = factory.create_pipeline_simple(include_bytes!("shader/triangle_150.glslv"),
                                    include_bytes!("shader/triangle_150.glslf"),
                                    pipe::new())
            .unwrap();

        // let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());

        let (quad_vertex_buffer, quad_slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTS, &QUAD_INDICES[..]);

        let rect_props = factory.create_constant_buffer(1);
        let transform_buffer = factory.create_constant_buffer(1);
        let sampler = factory.create_sampler_linear();
        let texture = gfx_load_texture(&mut factory);
        let data = pipe::Data {
            vbuf: quad_vertex_buffer,
            tex: (texture, sampler),
            rect_properties: rect_props,
            transform: transform_buffer,
            out: color_view,
        };


        // Set initial uniform values
        let transform = Transform {
            transform: ortho(0.0,
                             screen_width as f32,
                             0.0,
                             screen_height as f32,
                             1.0,
                             -1.0),
        };
        let transform = Transform { transform: ortho(-1.5, 1.5, 1.0, -1.0, 1.0, -1.0) };
        // let transform = Transform { transform: ortho(1.5, -1.5, -1.0, -1.0, -1.0, 1.0) };
        encoder.update_buffer(&data.transform, &[transform], 0);

        Ok(GraphicsContext {
            background_color: Color::new(0.1, 0.2, 0.3, 1.0),
            foreground_color: Color::new(1.0, 1.0, 1.0, 1.0),

            window: window,
            gl_context: gl_context,
            device: Box::new(device),
            factory: Box::new(factory),
            encoder: encoder,
            // color_view: color_view,
            depth_view: depth_view,

            pso: pso,
            data: data,
            // slice: slice,
            quad_slice: quad_slice,
        })
    }
}


pub fn set_screen_coordinates(context: &mut Context,
                              left: f32,
                              right: f32,
                              top: f32,
                              bottom: f32) {
    let gfx = &mut context.gfx_context;
    let transform = Transform { transform: ortho(left, right, top, bottom, 1.0, -1.0) };
    gfx.encoder.update_buffer(&gfx.data.transform, &[transform], 0);
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

    [[c0r0, c0r1, c0r2, c0r3],
     [c1r0, c1r1, c1r2, c1r3],
     [c2r0, c2r1, c2r2, c2r3],
     [c3r0, c3r1, c3r2, c3r3]]
}


/// Sets the background color.  Default: blue.
pub fn set_background_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.background_color = color;
}

/// Sets the foreground color, which will be used for drawing
/// rectangles, lines, etc.  Default: white.
pub fn set_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.foreground_color = color;
}

/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    gfx.encoder.clear(&gfx.data.out, gfx.background_color.into());
}

/// Draws the given `Drawable` object to the screen.
pub fn draw(ctx: &mut Context,
            drawable: &mut Drawable,
            src: Option<Rect>,
            dst: Option<Rect>)
            -> GameResult<()> {
    drawable.draw(ctx, src, dst)
}


/// Draws the given `Drawable` object to the screen,
/// applying a rotation and mirroring if desired.
// #[allow(too_many_arguments)]
pub fn draw_ex(ctx: &mut Context,
               drawable: &mut Drawable,
               src: Option<Rect>,
               dst: Option<Rect>,
               angle: f64,
               center: Option<Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
               -> GameResult<()> {
    drawable.draw_ex(ctx, src, dst, angle, center, flip_horizontal, flip_vertical)
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
    unimplemented!();
    // let r = &mut ctx.renderer;
    // match mode {
    //     DrawMode::Line => {
    //         let res = r.draw_rect(rect);
    //         res.map_err(GameError::from)
    //     }
    //     DrawMode::Fill => {
    //         let res = r.fill_rect(rect);
    //         res.map_err(GameError::from)
    //     }
    // }
}

/// Draws many rectangles.
/// Not part of the LOVE API but no reason not to include it.
pub fn rectangles(ctx: &mut Context, mode: DrawMode, rect: &[Rect]) -> GameResult<()> {
    unimplemented!();
    // let r = &mut ctx.renderer;
    // match mode {
    //     DrawMode::Line => {
    //         let res = r.draw_rects(rect);
    //         res.map_err(GameError::from)
    //     }
    //     DrawMode::Fill => {
    //         let res = r.fill_rects(rect);
    //         res.map_err(GameError::from)

    //     }
    // }
}


/// Draws a line.
/// Currently lines are 1 pixel wide and generally ugly.
pub fn line(ctx: &mut Context, start: Point, end: Point) -> GameResult<()> {
    unimplemented!();
    // let r = &mut ctx.renderer;
    // let res = r.draw_line(start, end);
    // res.map_err(GameError::from)
}

/// Draws a series of connected lines.
pub fn lines(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    unimplemented!();
    // let r = &mut ctx.renderer;
    // let res = r.draw_lines(points);
    // res.map_err(GameError::from)
}

/// Draws a 1-pixel point.
pub fn point(ctx: &mut Context, point: Point) -> GameResult<()> {
    unimplemented!();
    // let r = &mut ctx.renderer;
    // let res = r.draw_point(point);
    // res.map_err(GameError::from)
}

/// Draws a set of points.
pub fn points(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    unimplemented!();
    // let r = &mut ctx.renderer;
    // let res = r.draw_points(points);
    // res.map_err(GameError::from)
}

/// All types that can be drawn on the screen implement the `Drawable` trait.
pub trait Drawable {
    /// Actually draws the object to the screen.
    /// This is the most general version of the operation, which is all that
    /// is required for implementing this trait.
    /// (It also maps nicely onto SDL2's Renderer::copy_ex(), we might want to
    /// wrap the types up a bit more nicely someday.)
    // #[allow(too_many_arguments)]
    fn draw_ex(&mut self,
               context: &mut Context,
               src: Option<Rect>,
               dst: Option<Rect>,
               angle: f64,
               center: Option<Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
               -> GameResult<()>;

    /// Draws the drawable onto the rendering target.
    fn draw(&mut self,
            context: &mut Context,
            src: Option<Rect>,
            dst: Option<Rect>)
            -> GameResult<()> {
        self.draw_ex(context, src, dst, 0.0, None, false, false)
    }
}

/// In-memory image data available to be drawn on the screen.
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

    /// Creates an Image from an array of u8's arranged in RGBA order.
    pub fn from_rgba8(context: &mut Context,
                      width: u16,
                      height: u16,
                      rgba: &[&[u8]])
                      -> GameResult<Image> {
        let gfx = &mut context.gfx_context;
        let kind = gfx::texture::Kind::D2(width, height, gfx::texture::AaMode::Single);
        let (_, view) = gfx.factory.create_texture_immutable_u8::<Rgba8>(kind, &rgba).unwrap();
        Ok(Image {
            texture: view,
            width: width as u32,
            height: height as u32,
            properties: RectProperties::default(),
        })
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

    /// Returns the `BlendMode` of the image.
    pub fn blend_mode(&self) -> BlendMode {
        unimplemented!();
    }

    /// Sets the `BlendMode` of the image.
    /// See <https://wiki.libsdl.org/SDL_SetRenderDrawBlendMode>
    /// for detailed description of blend modes.
    pub fn set_blend_mode(&mut self, blend: BlendMode) {
        unimplemented!();
    }

    /// Get the color mod of the image.
    pub fn color_mod(&self) -> Color {
        unimplemented!();
    }

    /// Set the color mod of the image.
    /// Each pixel of the image is multiplied by this color
    /// when drawn.
    pub fn set_color_mod(&mut self, color: Color) {
        unimplemented!();
    }

    /// Get the alpha mod of the image.
    pub fn alpha_mod(&self) -> u8 {
        unimplemented!();
    }

    /// Set the alpha mod of the image.
    /// Each pixel's alpha will be multiplied by this value
    /// when drawn.
    pub fn set_alpha_mod(&mut self, alpha: u8) {
        unimplemented!();
    }

    /// Return the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Return the height of the image.
    pub fn height(&self) -> u32 {
        self.height
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
               src: Option<Rect>,
               dst: Option<Rect>,
               angle: f64,
               center: Option<Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
               -> GameResult<()> {

        let gfx = &mut context.gfx_context;
        let dst = dst.unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0));
        // let thing = RectProperties { offset: [dst.x, dst.y] };
        self.properties.offset = [dst.x, dst.y];
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
        surface: surface::Surface<'static>,
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
        let s2 = util::load_surface(context, path.as_ref())?;

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
                 surface: &surface::Surface,
                 glyphs_map: &BTreeMap<char, u32>,
                 glyph_width: u32)
                 -> GameResult<Text> {
    let text_length = text.len() as u32;
    let glyph_height = surface.height();
    let format = pixels::PixelFormatEnum::RGBA8888;
    let mut dest_surface = surface::Surface::new(text_length * glyph_width, glyph_height, format)?;
    for (i, c) in text.chars().enumerate() {
        let small_i = i as u32;
        let error_message = format!("Character '{}' not in bitmap font!", c);
        let source_offset = glyphs_map.get(&c)
            .ok_or(GameError::FontError(String::from(error_message)))?;
        let dest_offset = glyph_width * small_i;
        let source_rect = Rect::new(*source_offset as f32,
                                    0.0,
                                    glyph_width as f32,
                                    glyph_height as f32);
        let dest_rect = Rect::new(dest_offset as f32,
                                  0.0,
                                  glyph_width as f32,
                                  glyph_height as f32);
        // println!("Blitting letter {} to {:?}", c, dest_rect);
        // surface.blit(Some(source_rect), &mut dest_surface, Some(dest_rect))?;
    }
    // let image = Image::from_surface(context, dest_surface)?;
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
               src: Option<Rect>,
               dst: Option<Rect>,
               angle: f64,
               center: Option<Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
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
