/// The `graphics` module performs the actual drawing of images, text, and other
/// objects with the `Drawable` trait.  It also handles basic loading of images
/// and text, apparently.
///
/// Also manages graphics state, coordinate systems, etc.  The default coordinate system
/// has the origin in the upper-left corner of the screen, unless it should be
/// something else, then we should change it.  

use context::Context;

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
trait Drawable {
}

/// In-memory image data available to be drawn on the screen.
struct Image {
}

impl Image {
    fn new() {
    }
}

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image.
struct Font {
}

impl Font {
    fn new() {
    }

    fn from_image() {
    }
}

/// Drawable text.
struct Text {
}

impl Text {
    fn new() {
    }
}
