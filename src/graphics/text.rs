use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io::Read;
use std::path;

use gfx_glyph::FontId;
use image;
use image::RgbaImage;

use super::*;

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image (bitmap fonts).
#[derive(Copy, Clone)]
pub enum Font {
    /// A TrueType font stored in `GraphicsContext::glyph_brush`
    GlyphFont(FontId),
}

/*
/// A bitmap font where letter widths are infered
#[derive(Clone, Debug)]
pub struct BitmapFont {
    /// The original glyph image
    bytes: Vec<u8>,
    /// Width of the image
    width: usize,
    /// Height of the image (same as the height of a glyph)
    height: usize,
    /// Glyph to horizontal position (in pixels) and span (in pixels) (does not include space)
    glyphs: BTreeMap<char, (usize, usize)>,
    /// Width in pixels of the space
    space_width: usize,
    letter_separation: usize,
}


/// A mapping from character to glyph location for bitmap fonts.
/// All coordinates are stored in the span `[0-1]`.
#[derive(Debug, Clone)]
pub struct BitmapFontLayout {
    /// The layout information for each character.
    pub mapping: HashMap<char, Rect>,
}

impl BitmapFontLayout {
    /// Creates a new `BitmapFontLayout` by assuming that
    /// the characters form a uniform grid with the glyphs
    /// given by the string going from left to right, top to bottom,
    /// and the size of a grid cell is given by `rect` (in the span
    /// `[0-1]`)
    /// 
    /// Because the grid cells are in `[0-1]` this doesn't need to know
    /// how big the actual image is.
    /// 
    /// TODO: This coordinate system is inconsistent with SpriteBatch.  :-/
    fn uniform(s: &str, rect: Rect) {
        // TODO
    }

    /// Takes a something implementing `IntoIterator` and creates a
    /// `BitmapFontLayout` with the items it yields.
    /// 
    /// TODO: Implement FromIterator?
    fn from_specification<T>(iter: T)
    where T: IntoIterator<Item=(char, Rect)> {
        // TODO
    }
}
*/

impl Font {
    /// Load a new TTF font from the given file.
    pub fn new<P>(context: &mut Context, path: P) -> GameResult<Font>
    where
        P: AsRef<path::Path> + fmt::Debug,
    {
        let name = format!("{:?}", path);

        // TODO: DPI; see winit #548.  Also need point size, pixels, etc...
        Font::new_glyph_font(context, path)
    }

    /// Loads a new TrueType font from given bytes and into `GraphicsContext::glyph_brush`.
    pub fn new_glyph_font_bytes(context: &mut Context, bytes: &[u8]) -> GameResult<Self> {
        // TODO: Take a Cow here to avoid this clone where unnecessary?
        let v = bytes.to_vec();
        let font_id = context.gfx_context.glyph_brush.add_font_bytes(v);

        Ok(Font::GlyphFont(font_id))
    }

    /// Loads a new TrueType font from given file and into `GraphicsContext::glyph_brush`.
    pub fn new_glyph_font<P>(context: &mut Context, path: P) -> GameResult<Self>
    where
        P: AsRef<path::Path> + fmt::Debug,
    {
        let mut stream = context.filesystem.open(path.as_ref())?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;

        Font::new_glyph_font_bytes(context, &buf)
    }

    /// Returns the baked-in bytes of default font (currently `DejaVuSerif.ttf`).
    pub(crate) fn default_font_bytes() -> &'static [u8] {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/DejaVuSerif.ttf"
        ))
    }

    /// Returns baked-in default font (currently DejaVuSerif.ttf).
    /// Note it does create a new `Font` object with every call;
    /// although the actual data should be shared.
    pub fn default_font(context: &Context) -> Self {
        // BUGGO: fix DPI.
        context.default_font.clone()
    }

    /// Get the height of the Font in pixels.
    ///
    /// The height of the font includes any spacing, it will be the total height
    /// a line needs.
    /// TODO: Probably made obsolete by GlyphFont
    pub fn get_height(&self) -> usize {
        match *self {
            Font::GlyphFont(_) => 0,
        }
    }

    /// Returns the width a line of text needs, in pixels.
    /// Does not handle line-breaks.
    /// TODO: Probably made obsolete by GlyphFont
    pub fn get_width(&self, text: &str) -> usize {
        match *self {
            Font::GlyphFont(_) => 0,
        }
    }

    /// Breaks the given text into lines that will not exceed `wrap_limit` pixels
    /// in length when drawn with the given font.
    /// It accounts for newlines correctly but does not
    /// try to break words or handle hyphenated words; it just breaks
    /// at whitespace.  (It also doesn't preserve whitespace.)
    ///
    /// Returns a tuple of maximum line width and a `Vec` of wrapped `String`s.
    /// TODO: Probably made obsolete by GlyphFont
    pub fn get_wrap(&self, text: &str, wrap_limit: usize) -> (usize, Vec<String>) {
        let mut broken_lines = Vec::new();
        for line in text.lines() {
            let mut current_line = Vec::new();
            for word in line.split_whitespace() {
                // I'm sick of trying to do things the clever way and
                // build up a line word by word while tracking how
                // long it should be, so instead I just re-render the whole
                // line, incrementally adding a word at a time until it
                // becomes too long.
                // This is not the most efficient way but it is simple and
                // it works.
                let mut prospective_line = current_line.clone();
                prospective_line.push(word);
                let text = prospective_line.join(" ");
                let prospective_line_width = self.get_width(&text);
                if prospective_line_width > wrap_limit {
                    // Current line is long enough, keep it
                    broken_lines.push(current_line.join(" "));
                    // and overflow the current word onto the next line.
                    current_line.clear();
                    current_line.push(word);
                } else {
                    // Current line with the added word is still short enough
                    current_line.push(word);
                }
            }

            // Push the last line of the text
            broken_lines.push(current_line.join(" "));
        }

        // If we have a line with only whitespace on it,
        // this results in the unwrap_or value.
        // And we can't create a texture of size 0, so
        // we put 1 here.
        // Not entirely sure what this will actually result
        // in though; hopefully a blank line.
        let max_line_length = broken_lines
            .iter()
            .map(|line| self.get_width(line))
            .max()
            .unwrap_or(1);

        (max_line_length, broken_lines)
    }
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Font::GlyphFont { .. } => write!(f, "<GlyphFont: {:p}>", &self),
        }
    }
}

/// Drawable text created from a `Font`.
#[derive(Clone)]
pub struct Text {
    texture: Image,
    contents: String,
    blend_mode: Option<BlendMode>,
}

impl Text {
    /// Renders a new `Text` from the given `Font`.
    ///
    /// Note that this is relatively computationally expensive;
    /// if you want to draw text every frame you probably want to save
    /// it and only update it when the text changes.
    pub fn new(context: &mut Context, text: &str, font: &Font) -> GameResult<Text> {
        match *font {
            Font::GlyphFont(_) => Err(GameError::FontError(
                "`Text` can't be created with a `Font::GlyphFont` (yet)!".into(),
            )),
        }
    }

    /// Returns the width of the rendered text, in pixels.
    pub fn width(&self) -> u32 {
        self.texture.width()
    }

    /// Returns the height of the rendered text, in pixels.
    pub fn height(&self) -> u32 {
        self.texture.height()
    }

    /// Returns the string that the text represents.
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// Returns the dimensions of the rendered text.
    pub fn get_dimensions(&self) -> Rect {
        self.texture.get_dimensions()
    }

    /// Get the filter mode for the the rendered text.
    pub fn get_filter(&self) -> FilterMode {
        self.texture.get_filter()
    }

    /// Set the filter mode for the the rendered text.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.texture.set_filter(mode);
    }

    /// Returns a reference to the `Image` contained
    /// by the `Text` object.
    pub fn get_image(&self) -> &Image {
        &self.texture
    }

    /// Returns a mutable  reference to the `Image` contained
    /// by the `Text` object.
    pub fn get_image_mut(&mut self) -> &mut Image {
        &mut self.texture
    }

    /// Unwraps the `Image` contained
    /// by the `Text` object.
    pub fn into_inner(self) -> Image {
        self.texture
    }
}

impl Drawable for Text {
    fn draw_primitive(&self, ctx: &mut Context, param: PrimitiveDrawParam) -> GameResult {
        draw_primitive(ctx, &self.texture, param)
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }
    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<Text: {}x{}, {:p}>",
            self.texture.width, self.texture.height, &self
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /*
    #[test]
    fn test_metrics() {
        let f = Font::default_font().expect("Could not get default font");
        assert_eq!(f.get_height(), 17);
        assert_eq!(f.get_width("Foo!"), 33);

        // http://www.catipsum.com/index.php
        let text_to_wrap = "Walk on car leaving trail of paw prints on hood and windshield sniff \
                            other cat's butt and hang jaw half open thereafter for give attitude. \
                            Annoy kitten\nbrother with poking. Mrow toy mouse squeak roll over. \
                            Human give me attention meow.";
        let (len, v) = f.get_wrap(text_to_wrap, 250);
        println!("{} {:?}", len, v);
        assert_eq!(len, 249);

        /*
        let wrapped_text = vec![
            "Walk on car leaving trail of paw prints",
            "on hood and windshield sniff other",
            "cat\'s butt and hang jaw half open",
            "thereafter for give attitude. Annoy",
            "kitten",
            "brother with poking. Mrow toy",
            "mouse squeak roll over. Human give",
            "me attention meow."
        ];
*/
        let wrapped_text = vec![
            "Walk on car leaving trail of paw",
            "prints on hood and windshield",
            "sniff other cat\'s butt and hang jaw",
            "half open thereafter for give",
            "attitude. Annoy kitten",
            "brother with poking. Mrow toy",
            "mouse squeak roll over. Human",
            "give me attention meow.",
        ];

        assert_eq!(&v, &wrapped_text);
    }

    // We sadly can't have this test in the general case because it needs to create a Context,
    // which creates a window, which fails on a headless server like our CI systems.  :/
    //#[test]
    #[allow(dead_code)]
    fn test_wrapping() {
        use conf;
        let c = conf::Conf::new();
        let (ctx, _) = &mut Context::load_from_conf("test_wrapping", "ggez", c)
            .expect("Could not create context?");
        let font = Font::default_font().expect("Could not get default font");
        let text_to_wrap = "Walk on car leaving trail of paw prints on hood and windshield sniff \
                            other cat's butt and hang jaw half open thereafter for give attitude. \
                            Annoy kitten\nbrother with poking. Mrow toy mouse squeak roll over. \
                            Human give me attention meow.";
        let wrap_length = 250;
        let (len, v) = font.get_wrap(text_to_wrap, wrap_length);
        assert!(len < wrap_length);
        for line in &v {
            let t = Text::new(ctx, line, &font).unwrap();
            println!(
                "Width is claimed to be <= {}, should be <= {}, is {}",
                len,
                wrap_length,
                t.width()
            );
            // Why does this not match?  x_X
            //assert!(t.width() as usize <= len);
            assert!(t.width() as usize <= wrap_length);
        }
    }
    */
}
