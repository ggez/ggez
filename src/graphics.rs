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
use std::io::Read;

use sdl2::pixels;
use sdl2::render;
use sdl2::surface;
use sdl2::image::ImageRWops;
use rusttype;

use context::Context;
use GameError;
use GameResult;
use util;

pub use sdl2::rect::Rect;
pub use sdl2::rect::Point;
pub use sdl2::pixels::Color;
pub use sdl2::render::BlendMode;

/// Specifies whether a shape should be drawn
/// filled or as an outline.
#[derive(Debug)]
pub enum DrawMode {
    Line,
    Fill,
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
#[derive(Debug)]
pub struct GraphicsContext {
    background: pixels::Color,
    foreground: pixels::Color,
}

impl GraphicsContext {
    pub fn new() -> GraphicsContext {
        GraphicsContext {
            background: pixels::Color::RGB(0u8, 0u8, 255u8),
            foreground: pixels::Color::RGB(255, 255, 255),
        }
    }
}

impl Default for GraphicsContext {
    fn default() -> Self {
        Self::new()
    }
}


/// Sets the background color.  Default: blue.
pub fn set_background_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.background = color;
}

/// Sets the foreground color, which will be used for drawing
/// rectangles, lines, etc.  Default: white.
pub fn set_color(ctx: &mut Context, color: Color) {
    let r = &mut ctx.renderer;
    ctx.gfx_context.foreground = color;
    r.set_draw_color(ctx.gfx_context.foreground);
}

/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context) {
    let r = &mut ctx.renderer;
    r.set_draw_color(ctx.gfx_context.background);
    r.clear();

    // We assume we are usually going to be wanting to draw the foreground color.
    // While clear() is a relatively rare operation (probably once per frame).
    // So we keep SDL's render state set to the foreground color by default.
    r.set_draw_color(ctx.gfx_context.foreground);
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
    let r = &mut ctx.renderer;
    r.present()
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
    let r = &mut ctx.renderer;
    match mode {
        DrawMode::Line => {
            let res = r.draw_rect(rect);
            res.map_err(GameError::from)
        }
        DrawMode::Fill => {
            let res = r.fill_rect(rect);
            res.map_err(GameError::from)

        }
    }
}

/// Draws many rectangles.
/// Not part of the LOVE API but no reason not to include it.
pub fn rectangles(ctx: &mut Context, mode: DrawMode, rect: &[Rect]) -> GameResult<()> {
    let r = &mut ctx.renderer;
    match mode {
        DrawMode::Line => {
            let res = r.draw_rects(rect);
            res.map_err(GameError::from)
        }
        DrawMode::Fill => {
            let res = r.fill_rects(rect);
            res.map_err(GameError::from)

        }
    }
}


/// Draws a line.
/// Currently lines are 1 pixel wide and generally ugly.
pub fn line(ctx: &mut Context, start: Point, end: Point) -> GameResult<()> {
    let r = &mut ctx.renderer;
    let res = r.draw_line(start, end);
    res.map_err(GameError::from)
}

/// Draws a series of connected lines.
pub fn lines(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    let r = &mut ctx.renderer;
    let res = r.draw_lines(points);
    res.map_err(GameError::from)
}

/// Draws a 1-pixel point.
pub fn point(ctx: &mut Context, point: Point) -> GameResult<()> {
    let r = &mut ctx.renderer;
    let res = r.draw_point(point);
    res.map_err(GameError::from)
}

/// Draws a set of points.
pub fn points(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    let r = &mut ctx.renderer;
    let res = r.draw_points(points);
    res.map_err(GameError::from)
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
pub struct Image {
    // Keeping a hold of both a surface and texture is a pain in the butt
    // but I can't see of a good way to manage both if we ever want to generate
    // or modify an image... such as creating bitmap fonts.
    // Hmmm.
    // For now, bitmap fonts is the only time we need to do that, so we'll special
    // case that rather than trying to create textures on-demand or something...
    texture: render::Texture,
    texture_query: render::TextureQuery,
}

impl Image {
    // An Image is implemented as an sdl2 Texture which has to be associated
    // with a particular Renderer.
    // This may eventually cause problems if there's ever ways to change
    // renderer, such as changing windows or something.
    // Suffice to say for now, Images are bound to the Context in which
    // they are created.
    /// Load a new image from the file at the given path.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Image> {
        let mut buffer: Vec<u8> = Vec::new();
        let rwops = util::rwops_from_path(context, path.as_ref(), &mut buffer)?;
        // SDL2_image SNEAKILY adds the load() method to RWops.
        let surf = rwops.load()?;
        let renderer = &context.renderer;

        let tex = renderer.create_texture_from_surface(surf)?;
        let tq = tex.query();
        Ok(Image {
            texture: tex,
            texture_query: tq,
        })

    }

    /// A little helper function that creates a new Image that is just
    /// a solid square of the given size and color.  Mainly useful for
    /// debugging.
    pub fn solid(context: &mut Context, size: u32, color: Color) -> GameResult<Image> {
        let mut surf = surface::Surface::new(size, size, pixels::PixelFormatEnum::RGBA8888)?;
        surf.fill_rect(None, color)?;
        Image::from_surface(context, surf)
    }

    fn from_surface(context: &Context, surface: surface::Surface) -> GameResult<Image> {
        let renderer = &context.renderer;
        let tex = renderer.create_texture_from_surface(surface)?;
        let tq = tex.query();
        Ok(Image {
            texture: tex,
            texture_query: tq,
        })
    }


    /// Returns the dimensions of the image.
    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width(), self.height())
    }

    /// Returns the `BlendMode` of the image.
    pub fn blend_mode(&self) -> BlendMode {
        self.texture.blend_mode()
    }

    /// Sets the `BlendMode` of the image.
    /// See <https://wiki.libsdl.org/SDL_SetRenderDrawBlendMode>
    /// for detailed description of blend modes.
    pub fn set_blend_mode(&mut self, blend: BlendMode) {
        self.texture.set_blend_mode(blend)
    }

    /// Get the color mod of the image.
    pub fn color_mod(&self) -> Color {
        let (r, g, b) = self.texture.color_mod();
        pixels::Color::RGB(r, g, b)
    }

    /// Set the color mod of the image.
    /// Each pixel of the image is multiplied by this color
    /// when drawn.
    pub fn set_color_mod(&mut self, color: Color) {
        match color {
            pixels::Color::RGB(r, g, b) |
            pixels::Color::RGBA(r, g, b, _) => self.texture.set_color_mod(r, g, b),
        }
    }

    /// Get the alpha mod of the image.
    pub fn alpha_mod(&self) -> u8 {
        self.texture.alpha_mod()
    }

    /// Set the alpha mod of the image.
    /// Each pixel's alpha will be multiplied by this value
    /// when drawn.
    pub fn set_alpha_mod(&mut self, alpha: u8) {
        self.texture.set_alpha_mod(alpha)
    }

    /// Return the width of the image.
    pub fn width(&self) -> u32 {
        self.texture_query.width
    }

    /// Return the height of the image.
    pub fn height(&self) -> u32 {
        self.texture_query.height
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
        let renderer = &mut context.renderer;
        renderer.copy_ex(&self.texture,
                     src,
                     dst,
                     angle,
                     center,
                     flip_horizontal,
                     flip_vertical)
            .map_err(GameError::RenderError)
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
    texture: render::Texture,
    texture_query: render::TextureQuery,
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

fn render_ttf(context: &Context,
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
    let format = pixels::PixelFormatEnum::RGBA8888;
    let surface = try!(surface::Surface::from_data(&mut pixel_data,
                                                   width as u32,
                                                   pixel_height as u32,
                                                   pitch as u32,
                                                   format));

    let image = Image::from_surface(context, surface)?;
    let text_string = text.to_string();
    let tq = image.texture.query();
    Ok(Text {
        texture: image.texture,
        texture_query: tq,
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
        let source_rect = Rect::new(*source_offset as i32, 0, glyph_width, glyph_height);
        let dest_rect = Rect::new(dest_offset as i32, 0, glyph_width, glyph_height);
        // println!("Blitting letter {} to {:?}", c, dest_rect);
        surface.blit(Some(source_rect), &mut dest_surface, Some(dest_rect))?;
    }
    let image = Image::from_surface(context, dest_surface)?;
    let text_string = text.to_string();
    let tq = image.texture.query();
    Ok(Text {
        texture: image.texture,
        texture_query: tq,
        contents: text_string,
    })
}


impl Text {
    /// Renders a new `Text` from the given `Font`
    pub fn new(context: &Context, text: &str, font: &Font) -> GameResult<Text> {
        match *font {
            Font::TTFFont { font: ref f, points } => render_ttf(context, text, f, points),
            Font::BitmapFont { ref surface, glyph_width, glyphs: ref glyphs_map, .. } => {
                render_bitmap(context, text, surface, glyphs_map, glyph_width)
            }
        }
    }

    /// Returns the width of the rendered text, in pixels.
    pub fn width(&self) -> u32 {
        self.texture_query.width
    }

    /// Returns the height of the rendered text, in pixels.
    pub fn height(&self) -> u32 {
        self.texture_query.height
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
        let renderer = &mut context.renderer;
        renderer.copy_ex(&self.texture,
                     src,
                     dst,
                     angle,
                     center,
                     flip_horizontal,
                     flip_vertical)
            .map_err(GameError::RenderError)
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tq = self.texture.query();
        write!(f,
               "<Text: {}x{}, {:p}, texture address {:p}>",
               tq.width,
               tq.height,
               &self,
               &self.texture)

    }
}
