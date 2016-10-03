//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text, apparently.
//!
//! Also manages graphics state, coordinate systems, etc.  The default coordinate system
//! has the origin in the upper-left corner of the screen, unless it should be
//! something else, then we should change it.  

use std::path;

use sdl2::pixels;
use sdl2::rect;
use sdl2::render;
use sdl2_image::ImageRWops;
use sdl2_ttf;

use context::Context;
use GameError;
use GameResult;
use util::rwops_from_path;

pub type Rect = rect::Rect;
pub type Point = rect::Point;
pub type Color = pixels::Color;

#[derive(Debug)]
pub enum DrawMode {
    Line,
    Fill
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


pub fn set_background_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.background = color;
}

pub fn set_color(ctx: &mut Context, color: Color) {
    let ref mut r = ctx.renderer;
    ctx.gfx_context.foreground = color;
    r.set_draw_color(ctx.gfx_context.foreground);
}

pub fn clear(ctx: &mut Context) {
    let ref mut r = ctx.renderer;
    r.set_draw_color(ctx.gfx_context.background);
    r.clear();

    // We assume we are usually going to be wanting to draw the foreground color.
    // While clear() is a relatively rare operation (probably once per frame).
    // So we keep SDL's render state set to the foreground color by default.
    r.set_draw_color(ctx.gfx_context.foreground);
}

pub fn draw(ctx: &mut Context, drawable: &Drawable, src: Option<Rect>, dst: Option<Rect>) -> GameResult<()> {
    drawable.draw(ctx, src, dst)
}

pub fn draw_ex(ctx: &mut Context, drawable: &Drawable, src: Option<Rect>, dst: Option<Rect>,
           angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
           -> GameResult<()> {
    drawable.draw_ex(ctx, src, dst, angle, center, flip_horizontal, flip_vertical)
}

pub fn present(ctx: &mut Context) {
    let ref mut r = ctx.renderer;
    r.present()
}

pub fn print(ctx: &mut Context) {
    unimplemented!();
}

pub fn printf(ctx: &mut Context) {
    unimplemented!();
}

pub fn rectangle(ctx: &mut Context, mode: DrawMode, rect: Rect) -> GameResult<()> {
    let ref mut r = ctx.renderer;
    match mode {
        DrawMode::Line => {
            let res = r.draw_rect(rect);
            res.map_err(GameError::from)                
        },
        DrawMode::Fill => {
            let res = r.fill_rect(rect);
            res.map_err(GameError::from)

        }
    }
}

/// Not part of the Love2D API but no reason not to include it.
pub fn rectangles(ctx: &mut Context, mode: DrawMode, rect: &[Rect]) -> GameResult<()> {
    let ref mut r = ctx.renderer;
    match mode {
        DrawMode::Line => {
            let res = r.draw_rects(rect);
            res.map_err(GameError::from)
        },
        DrawMode::Fill => {
            let res = r.fill_rects(rect);
            res.map_err(GameError::from)

        }
    }
}


pub fn line(ctx: &mut Context, start: Point, end: Point) -> GameResult<()> {
    let ref mut r = ctx.renderer;
    let res = r.draw_line(start, end);
    res.map_err(GameError::from)
}

pub fn lines(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    let ref mut r = ctx.renderer;
    let res = r.draw_lines(points);
    res.map_err(GameError::from)
}

pub fn point(ctx: &mut Context, point: Point) -> GameResult<()> {
    let ref mut r = ctx.renderer;
    let res = r.draw_point(point);
    res.map_err(GameError::from)
}

pub fn points(ctx: &mut Context, points: &[Point]) -> GameResult<()> {
    let ref mut r = ctx.renderer;
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
    fn draw_ex(&self, context: &mut Context, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> GameResult<()>;

    /// Draws the drawable onto the rendering target.
    fn draw(&self, context: &mut Context, src: Option<Rect>, dst: Option<Rect>) -> GameResult<()> {
        let res = self.draw_ex(context, src, dst, 0.0, None, false, false);
        res
    }
}

/// In-graphics-memory image data available to be drawn on the screen.
pub struct Image {
    texture: render::Texture,
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
        // SDL2_image SNEAKILY adds this method to RWops.
        let surf = try!(rwops.load());
        let renderer = &context.renderer;
        let tex = try!(renderer.create_texture_from_surface(surf));
        Ok(Image {
            texture: tex,
        })
    }
}

impl Drawable for Image {
    fn draw_ex(&self, context: &mut Context, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> GameResult<()> {
        let ref mut renderer = context.renderer;
        renderer.copy_ex(&self.texture, src, dst, angle, center, flip_horizontal, flip_vertical)
            .map_err(|s| GameError::RenderError(s))
    }

}

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image.
pub struct Font {
    font: sdl2_ttf::Font,
}

impl Font {
    /// Load a new TTF font from the given file.
    pub fn new(context: &mut Context, path: &path::Path, size: u16) -> GameResult<Font> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut rwops = try!(rwops_from_path(context, path, &mut buffer));

        let ttf_context = &context.ttf_context;
        let ttf_font = try!(ttf_context.load_font_from_rwops(&mut rwops, size));
        Ok(Font {
            font: ttf_font,
        })
    }

    /// Create a new bitmap font from an image file.
    /// The `glyphs` string is the characters in the image from left to right.
    /// TODO: Implement this!  Love2D just uses a 1D image for glyphs, which is
    /// maybe not ideal but is fine.
    pub fn from_image(name: &str, glyphs: &str) {
        unimplemented!()
    }
}




/// Drawable text.
/// SO FAR this doesn't need to be a separate type from Image, really.
/// But looking at various API's its functionality will probably diverge
/// eventually, so.
pub struct Text {
    texture: render::Texture,
}

impl Text {
    pub fn new(context: &Context, text: &str, font: &Font) -> GameResult<Text> {
        let renderer = &context.renderer;
        let surf = try!(font.font.render(text)
            .blended(pixels::Color::RGB(255,255,255)));
        // BUGGO: SEGFAULTS HERE!  But only when using solid(), not blended()!
        // Loading the font from a file rather than a RWops makes it work fine.
        // See https://github.com/andelf/rust-sdl2_ttf/issues/43
        let texture = try!(renderer.create_texture_from_surface(surf));
        Ok(Text {
            texture: texture,
        })
            
    }
}


impl Drawable for Text {
    fn draw_ex(&self, context: &mut Context, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> GameResult<()> {
        let ref mut renderer = context.renderer;
        renderer.copy_ex(&self.texture, src, dst, angle, center, flip_horizontal, flip_vertical)
                    .map_err(|s| GameError::RenderError(s))
    }
}

use std::fmt;
impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Text")
    }
}
