/// The `graphics` module performs the actual drawing of images, text, and other
/// objects with the `Drawable` trait.  It also handles basic loading of images
/// and text, apparently.
///
/// Also manages graphics state, coordinate systems, etc.  The default coordinate system
/// has the origin in the upper-left corner of the screen, unless it should be
/// something else, then we should change it.  

use std::path;
use std::io;
use std::marker::Sized;

use sdl2::pixels::Color;
use sdl2::rect;
use sdl2::render;
use sdl2::rwops;
use sdl2::surface;
use sdl2_image::ImageRWops;
use sdl2_ttf;

use context::Context;
use GameError;

pub type Rect = rect::Rect;
pub type Point = rect::Point;

// This is actually very inconvenient 'cause sdl2::rwops
// can be created from bytes, or from a file path, but not
// from a std::io::Read
// Which is what we need to read from streams.
fn rwops_from_read<'a, T>(r: &mut T, buffer: &'a mut Vec<u8>) -> Result<rwops::RWops<'a>, String>
    where T: io::Read + Sized {
    // For now, we just rather messily slurp the whole thing into memory,
    // then hand that to from_bytes.
    //let bytes: Vec<Result<u8, io::Error>> =
    r.read_to_end(buffer).unwrap();
    rwops::RWops::from_bytes(buffer)
}

fn rwops_from_path<'a>(context: &Context, path: &path::Path, buffer: &'a mut Vec<u8>) -> rwops::RWops<'a> {
    let fs = &context.filesystem;
    let mut stream = fs.open(path).unwrap();
    let mut rw = rwops_from_read(&mut stream, buffer);
    match &rw {
        &Ok(_) => (),
        &Err(ref msg) => println!("rwops_from_read error: {}", msg),
    }
    
    let mut rwops = rw.unwrap();
    rwops
}


// Not yet sure exactly how we should split this up;
// do we want to define our own GraphicsContext struct
// that a Context is a part of, or what?
impl<'a> Context<'a> {
    fn clear() {
    }

    fn draw() {
    }

    fn present() {
    }

    fn print() {
    }

    fn printf() {
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
               -> Result<(), GameError>;

    /// Draws the drawable onto the rendering target.
    fn draw(&self, context: &mut Context, src: Option<Rect>, dst: Option<Rect>) {
        let renderer = &mut context.renderer;
        let res = self.draw_ex(renderer, src, dst, 0.0, None, false, false);
        res.expect("Rendering error in Drawable.draw()");
    }
}

/// In-memory image data available to be drawn on the screen.
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
    // TODO: Make it use sdl2_gfx to load all image types.
    pub fn new(context: &Context, path: &path::Path) -> Image {
        let renderer = &context.renderer;

        let mut buffer: Vec<u8> = Vec::new();
        let mut rwops = rwops_from_path(context, path, &mut buffer);
        // SDL2_image SNEAKILY adds this method to RWops.
        let surf = rwops.load().unwrap();
        let tex = renderer.create_texture_from_surface(surf).unwrap();
        Image {
            texture: tex,
        }
    }
}

impl Drawable for Image {
    fn draw_ex(&self, renderer: &mut render::Renderer, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> Result<(), GameError> {
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
    pub fn new(context: &Context, path: &path::Path, size: u16) -> Font {
        let mut buffer: Vec<u8> = Vec::new();
        let mut rwops = rwops_from_path(context, path, &mut buffer);

        let ttf_context = &context.ttf_context;
        let ttf_font = ttf_context.load_font_from_rwops(rwops, size).unwrap();
        //ttf_context.load_font(path, size).unwrap();
        Font {
            font: ttf_font,
        }
        
    }

    /// Create a new bitmap font from an image file.
    /// The `glyphs` string is the characters in the image from left to right.
    /// TODO: Implement this!  Love2D just uses a 1D image for glyphs, which is
    /// probably not ideal but is fine.
    fn from_image(name: &str, glyphs: &str) {
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
    pub fn new(context: &Context, text: &str, font: &Font) -> Text {
        let renderer = &context.renderer;
        let surf = font.font.render(text)
            .solid(Color::RGB(255,255,255))
            .unwrap();
        let texture = renderer.create_texture_from_surface(surf).unwrap();
        Text {
            texture: texture,
        }
            
    }
}


impl Drawable for Text {
    fn draw_ex(&self, renderer: &mut render::Renderer, src: Option<Rect>, dst: Option<Rect>,
               angle: f64, center: Option<Point>, flip_horizontal: bool, flip_vertical: bool)
               -> Result<(), GameError> {
        renderer.copy_ex(&self.texture, src, dst, angle, center, flip_horizontal, flip_vertical)
            .map_err(|s| GameError::RenderError(s))
    }
}
