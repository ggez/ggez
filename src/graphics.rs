//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text, apparently.
//!
//! Also manages graphics state, coordinate systems, etc.  The default coordinate system
//! has the origin in the upper-left corner of the screen, unless it should be
//! something else, then we should change it.

use std::fmt;
use std::path;
use std::collections::BTreeMap;

use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2::surface;
use sdl2_image::ImageRWops;
use sdl2_ttf;

use context::Context;
use GameError;
use GameResult;
use util::rwops_from_path;

/// The same as an `sdl2::rect::Rect`
pub type Rect = rect::Rect;
/// The same as an `sdl2::rect::Point`
pub type Point = rect::Point;
/// The same as an `sdl2::pixels::Color`
pub type Color = pixels::Color;

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

pub fn draw(ctx: &mut Context,
            drawable: &Drawable,
            src: Option<Rect>,
            dst: Option<Rect>)
            -> GameResult<()> {
    drawable.draw(ctx, src, dst)
}

// #[allow(too_many_arguments)]
pub fn draw_ex(ctx: &mut Context,
               drawable: &Drawable,
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
/// since we don't have anything resembling a default font.
pub fn print(_ctx: &mut Context) {
    unimplemented!();
}

/// Not implemented
/// since we don't have anything resembling a default font.
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
/// Not part of the Love2D API but no reason not to include it.
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
    fn draw_ex(&self,
               context: &mut Context,
               src: Option<Rect>,
               dst: Option<Rect>,
               angle: f64,
               center: Option<Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
               -> GameResult<()>;

    /// Draws the drawable onto the rendering target.
    fn draw(&self, context: &mut Context, src: Option<Rect>, dst: Option<Rect>) -> GameResult<()> {
        self.draw_ex(context, src, dst, 0.0, None, false, false)
    }
}

/// In-memory image data available to be drawn on the screen.
/// TODO: Implement width, height, etc!
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
    pub fn new(context: &mut Context, path: &path::Path) -> GameResult<Image> {
        let mut buffer: Vec<u8> = Vec::new();
        let rwops = try!(rwops_from_path(context, path, &mut buffer));
        // SDL2_image SNEAKILY adds the load() method to RWops.
        let surf = try!(rwops.load());
        let renderer = &context.renderer;

        let tex = try!(renderer.create_texture_from_surface(surf));
        let tq = tex.query();
        Ok(Image {
            texture: tex,
            texture_query: tq,
        })

    }

    fn from_surface(context: &Context, surface: surface::Surface) -> GameResult<Image> {
        let renderer = &context.renderer;
        let tex = try!(renderer.create_texture_from_surface(surface));
        let tq = tex.query();
        Ok(Image {
            texture: tex,
            texture_query: tq,
        })
    }

    pub fn width(&self) -> u32 {
        self.texture_query.width
    }
    pub fn height(&self) -> u32 {
        self.texture_query.height
    }

    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width(), self.height())
    }
}


impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "<Image: {}x{}, {:p}, texture address {:p}>",
               self.width(),
               self.height(),
               &self,
               &self.texture)
    }
}


impl Drawable for Image {
    // #[allow(too_many_arguments)]
    fn draw_ex(&self,
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
/// Can be created from a .ttf file or from an image.
pub enum Font {
    TTFFont {
        font: sdl2_ttf::Font,
    },
    BitmapFont {
        surface: surface::Surface<'static>,
        glyphs: BTreeMap<char, u32>,
        glyph_width: u32,
    },
}

// Here you should just imagine me frothing at the mouth as I
// fight the lifetime checker in circles.
fn clone_surface<'a>(s: surface::Surface<'a>) -> GameResult<surface::Surface<'static>> {
    // let format = pixels::PixelFormatEnum::RGBA8888;
    let format = s.pixel_format();
    // convert() copies the surface anyway, so.
    let res = try!(s.convert(&format));
    Ok(res)
}

impl Font {
    /// Load a new TTF font from the given file.
    pub fn new(context: &mut Context, path: &path::Path, size: u16) -> GameResult<Font> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut rwops = try!(rwops_from_path(context, path, &mut buffer));

        let ttf_context = &context.ttf_context;
        let ttf_font = try!(ttf_context.load_font_from_rwops(&mut rwops, size));
        Ok(Font::TTFFont { font: ttf_font })
    }

    /// Loads an `Image` and uses it to create a new bitmap font
    /// The `Image` is a 1D list of glyphs, which maybe isn't
    /// super ideal but should be fine.
    /// The `glyphs` string is the characters in the image from left to right.
    pub fn new_bitmap(context: &mut Context, path: &path::Path, glyphs: &str) -> GameResult<Font> {
        let mut buffer: Vec<u8> = Vec::new();
        let rwops = try!(rwops_from_path(context, path, &mut buffer));
        // SDL2_image SNEAKILY adds the load() method to RWops.
        let surface = try!(rwops.load().map_err(GameError::ResourceLoadError));
        // We *really really* need to clone this surface here because
        // otherwise lifetime interactions between rwops, buffer and surface become
        // intensely painful.
        let s2 = try!(clone_surface(surface));

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
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Font::TTFFont { .. } => write!(f, "<TTFFont: {:p}>", &self),
            Font::BitmapFont { .. } => write!(f, "<BitmapFont: {:p}>", &self),
        }
    }
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
    let mut dest_surface = try!(surface::Surface::new(text_length * glyph_width,
                                                      glyph_height,
                                                      format));
    for (i, c) in text.chars().enumerate() {
        let small_i = i as u32;
        let error_message = format!("Character '{}' not in bitmap font!", c);
        let source_offset = try!(glyphs_map.get(&c)
            .ok_or(GameError::FontError(String::from(error_message))));
        let dest_offset = glyph_width * small_i;
        let source_rect = Rect::new(*source_offset as i32, 0, glyph_width, glyph_height);
        let dest_rect = Rect::new(dest_offset as i32, 0, glyph_width, glyph_height);
        // println!("Blitting letter {} to {:?}", c, dest_rect);
        try!(surface.blit(Some(source_rect), &mut dest_surface, Some(dest_rect)));
    }
    let image = try!(Image::from_surface(context, dest_surface));
    Ok(Text { texture: image.texture })
}

/// Drawable text created from a `Font`.
/// SO FAR this doesn't need to be a separate type from Image, really.
/// But looking at various API's its functionality will probably diverge
/// eventually, so.
pub struct Text {
    texture: render::Texture,
}

impl Text {
    /// Renders a new `Text` from the given `Font`
    pub fn new(context: &Context, text: &str, font: &Font) -> GameResult<Text> {
        let renderer = &context.renderer;
        match *font {
            Font::TTFFont { font: ref f } => {
                let surf = try!(f.render(text)
                                 .blended(pixels::Color::RGB(255, 255, 255)));
                // BUGGO: SEGFAULTS HERE!  But only when using solid(), not blended()!
                // Loading the font from a file rather than a RWops makes it work fine.
                // See https://github.com/andelf/rust-sdl2_ttf/issues/43
                let texture = try!(renderer.create_texture_from_surface(surf));
                Ok(Text { texture: texture })
            }
            Font::BitmapFont { ref surface, glyph_width, glyphs: ref glyphs_map, .. } => {
                render_bitmap(context, text, surface, glyphs_map, glyph_width)
            }
        }
    }
}


impl Drawable for Text {
    // #[allow(too_many_arguments)]
    fn draw_ex(&self,
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
