//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text, apparently.
//!
//! Also manages graphics state, coordinate systems, etc.  The default coordinate system
//! has the origin in the upper-left corner of the screen, unless it should be
//! something else, then we should change it.  

use std::path;

use sdl2::pixels::Color;
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


// Not yet sure exactly how we should split this up;
// do we want to define our own GraphicsContext struct
// that a Context is a part of, or what?
impl<'a> Context<'a> {
    fn clear() {
        unimplemented!();
    }

    fn draw() {
        unimplemented!();
    }

    fn present() {
        unimplemented!();
    }

    fn print() {
        unimplemented!();
    }

    fn printf() {
        unimplemented!();
    }
}

/// All types that can be drawn on the screen implement the `Drawable` trait.
pub trait Drawable {
    /// Actually draws the object to the screen.
    /// This is the most general version of the operation, which is all that
    /// is required for implementing this trait.
    /// (It also maps nicely onto SDL2's Renderer::copy_ex(), we might want to
    /// wrap the types up a bit more nicely someday.)
    fn draw_ex(&self, renderer: &mut render::Renderer, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> GameResult<()>;

    /// Draws the drawable onto the rendering target.
    fn draw(&self, context: &mut Context, src: Option<Rect>, dst: Option<Rect>) {
        let renderer = &mut context.renderer;
        let res = self.draw_ex(renderer, src, dst, 0.0, None, false, false);
        res.expect("Rendering error in Drawable.draw()");
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
    fn draw_ex(&self, renderer: &mut render::Renderer, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> GameResult<()> {
        renderer.copy_ex(&self.texture, src, dst, angle, center, flip_horizontal, flip_vertical)
            .map_err(|s| GameError::RenderError(s))
    }

}

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image.
pub struct Font {
    font: sdl2_ttf::Font,
//    _rwops: &'a rwops::RWops<'a>,
//    _buffer: &'a Vec<u8>,
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
//            _rwops: &rwops,
//            _buffer: &buffer,
        })
        
    }

    /// Create a new bitmap font from an image file.
    /// The `glyphs` string is the characters in the image from left to right.
    /// TODO: Implement this!  Love2D just uses a 1D image for glyphs, which is
    /// probably not ideal but is fine.
    fn from_image(name: &str, glyphs: &str) {
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
            .blended(Color::RGB(255,255,255)));
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
    fn draw_ex(&self, renderer: &mut render::Renderer, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> GameResult<()> {
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
