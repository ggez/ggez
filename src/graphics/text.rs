use std::cmp;
use std::fmt;
use std::path;
use std::collections::BTreeMap;
use std::io::Read;

use rusttype;
use image;

use super::*;

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image (bitmap fonts).
#[derive(Clone)]
pub enum Font {
    /// A truetype font
    TTFFont {
        /// The actual font data
        font: rusttype::Font<'static>,
        /// The size of the font
        points: u32,
        /// Scale information for the font
        scale: rusttype::Scale,
    },
    /// A bitmap font.
    BitmapFont {
        /// The original glyph image
        bytes: Vec<u8>,
        /// Width of the image
        width: usize,
        /// Height of the image (same as the height of a glyph)
        height: usize,
        /// Glyph to index mapping
        glyphs: BTreeMap<char, usize>,
        /// Width of the glyph
        glyph_width: usize,
    },
}

impl Font {
    /// Load a new TTF font from the given file.
    pub fn new<P>(context: &mut Context, path: P, points: u32) -> GameResult<Font>
    where
        P: AsRef<path::Path> + fmt::Debug,
    {
        let mut stream = context.filesystem.open(path.as_ref())?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;

        let name = format!("{:?}", path);

        // Get the proper DPI to scale font size accordingly
        let (_diag_dpi, x_dpi, y_dpi) = context.gfx_context.dpi;
        Font::from_bytes(&name, &buf, points, (x_dpi, y_dpi))
    }

    /// Loads a new TTF font from data copied out of the given buffer.
    pub fn from_bytes(name: &str, bytes: &[u8], points: u32, dpi: (f32, f32)) -> GameResult<Font> {
        let collection = rusttype::FontCollection::from_bytes(bytes.to_vec());
        let font_err = GameError::ResourceLoadError(format!(
            "Could not load font collection for \
             font {:?}",
            name
        ));
        let font = collection.into_font().ok_or(font_err)?;
        let (x_dpi, y_dpi) = dpi;
        // println!("DPI: {}, {}", x_dpi, y_dpi);
        let scale = display_independent_scale(points, x_dpi, y_dpi);

        Ok(Font::TTFFont {
            font: font,
            points: points,
            scale: scale,
        })
    }

    /// Loads an `Image` and uses it to create a new bitmap font
    /// The `Image` is a 1D list of glyphs, which maybe isn't
    /// super ideal but should be fine.
    /// The `glyphs` string is the characters in the image from left to right.
    pub fn new_bitmap<P: AsRef<path::Path>>(
        context: &mut Context,
        path: P,
        glyphs: &str,
    ) -> GameResult<Font> {
        let img = {
            let mut buf = Vec::new();
            let mut reader = context.filesystem.open(path)?;
            reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_rgba()
        };
        let (image_width, image_height) = img.dimensions();

        let glyph_width = (image_width as usize) / glyphs.len();
        // println!("Number of glyphs: {}, Glyph width: {}, image width: {}",
        // glyphs.len(), glyph_width, image_width);
        let mut glyphs_map: BTreeMap<char, usize> = BTreeMap::new();
        for (i, c) in glyphs.chars().enumerate() {
            glyphs_map.insert(c, i * glyph_width);
        }
        Ok(Font::BitmapFont {
            bytes: img.into_vec(),
            width: image_width as usize,
            height: image_height as usize,
            glyphs: glyphs_map,
            glyph_width: glyph_width,
        })
    }

    /// Returns a baked-in default font: currently DejaVuSerif.ttf
    /// Note it does create a new `Font` object with every call.
    pub fn default_font() -> GameResult<Self> {
        let size = 16;
        let buf = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/DejaVuSerif.ttf"
        ));
        // BUGGO: fix DPI.  Get from Context?  If we do that we can basically
        // just make Context always keep the default Font itself... hmm.
        Font::from_bytes("default", &buf[..], size, (75.0, 75.0))
    }

    /// Get the height of the Font in pixels.
    ///
    /// The height of the font includes any spacing, it will be the total height
    /// a line needs.
    pub fn get_height(&self) -> usize {
        match *self {
            Font::BitmapFont { height, .. } => height,
            Font::TTFFont { scale, .. } => {
                // let v_metrics = font.v_metrics(scale);
                // v_metrics.
                // TODO: Check and make sure this is right;
                // shouldn't we be using v_metrics instead?
                // if not you will have to change this in the
                // ttf font rendering code as well.
                scale.y.ceil() as usize
            }
        }
    }

    /// Returns the width a line of text needs, in pixels.
    /// Does not handle line-breaks.
    pub fn get_width(&self, text: &str) -> usize {
        match *self {
            Font::BitmapFont { width, .. } => width * text.len(),
            Font::TTFFont {
                ref font, scale, ..
            } => {
                let v_metrics = font.v_metrics(scale);
                let offset = rusttype::point(0.0, v_metrics.ascent);
                let glyphs: Vec<rusttype::PositionedGlyph> =
                    font.layout(text, scale, offset).collect();
                text_width(&glyphs) as usize
            }
        }
    }

    /// Breaks the given text into lines that will not exceed `wrap_limit` pixels
    /// in length when drawn with the given font.  
    /// It accounts for newlines correctly but does not
    /// try to break words or handle hyphenated words; it just breaks
    /// at whitespace.  (It also doesn't preserve whitespace.)
    ///
    /// Returns a tuple of maximum line width and a `Vec` of wrapped `String`s.
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
            Font::TTFFont { .. } => write!(f, "<TTFFont: {:p}>", &self),
            Font::BitmapFont { .. } => write!(f, "<BitmapFont: {:p}>", &self),
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
    rusttype::Scale {
        x: pixels_per_point_w * points,
        y: pixels_per_point_h * points,
    }
}

fn text_width(glyphs: &[rusttype::PositionedGlyph]) -> f32 {
    glyphs
        .iter()
        .rev()
        .filter_map(|g| {
            g.pixel_bounding_box()
                .map(|b| b.min.x as f32 + g.unpositioned().h_metrics().advance_width)
        })
        .next()
        .unwrap_or(0.0)
}

fn render_ttf(
    context: &mut Context,
    text: &str,
    font: &rusttype::Font<'static>,
    scale: rusttype::Scale,
) -> GameResult<Text> {
    // Ripped almost wholesale from
    // https://github.com/dylanede/rusttype/blob/master/examples/simple.rs

    let text_height_pixels = scale.y.ceil() as usize;
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);
    // Then turn them into an array of positioned glyphs...
    // `layout()` turns an abstract glyph, which contains no concrete
    // size or position information, into a PositionedGlyph, which does.
    let glyphs: Vec<rusttype::PositionedGlyph> = font.layout(text, scale, offset).collect();
    // If the string is empty or only whitespace, we end up trying to create a 0-width
    // texture which is invalid.  Instead we create a texture 1 texel wide, with everything
    // set to zero, which probably isn't ideal but is 100% consistent and doesn't require
    // special-casing things like get_filter().
    // See issue #109
    let text_width_pixels = cmp::max(text_width(&glyphs).ceil() as usize, 1);
    let bytes_per_pixel = 4;
    let mut pixel_data = vec![0; text_width_pixels * text_height_pixels * bytes_per_pixel];
    let pitch = text_width_pixels * bytes_per_pixel;

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
                if x >= 0 && x < text_width_pixels as i32 && y >= 0 && y < text_height_pixels as i32
                {
                    let x = x as usize * bytes_per_pixel;
                    let y = y as usize;
                    pixel_data[(x + y * pitch)] = 255;
                    pixel_data[(x + y * pitch + 1)] = 255;
                    pixel_data[(x + y * pitch + 2)] = 255;
                    pixel_data[(x + y * pitch + 3)] = c;
                }
            })
        }
    }

    // Copy the bitmap into an image, and we're basically done!
    assert!(text_width_pixels < u16::MAX as usize);
    assert!(text_height_pixels < u16::MAX as usize);
    let image = Image::from_rgba8(
        context,
        text_width_pixels as u16,
        text_height_pixels as u16,
        &pixel_data,
    )?;

    let text_string = text.to_string();
    Ok(Text {
        texture: image,
        contents: text_string,
        blend_mode: None,
    })
}

/// Treats src and dst as row-major 2D arrays, and blits the given rect from src to dst.
/// Does no bounds checking or anything; if you feed it invalid bounds it will just panic.
/// Generally, you shouldn't need to use this directly.
#[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
fn blit(
    dst: &mut [u8],
    dst_dims: (usize, usize),
    dst_point: (usize, usize),
    src: &[u8],
    src_dims: (usize, usize),
    src_point: (usize, usize),
    rect_size: (usize, usize),
    pitch: usize,
) {
    // The rect properties are all f32's; we truncate them down to integers.
    let area_row_width = rect_size.0 * pitch;
    let src_row_width = src_dims.0 * pitch;
    let dst_row_width = dst_dims.0 * pitch;

    for row_idx in 0..rect_size.1 {
        let src_row = row_idx + src_point.1;
        let dst_row = row_idx + dst_point.1;
        let src_offset = src_row * src_row_width + (src_point.0 * pitch);
        let dst_offset = dst_row * dst_row_width + (dst_point.0 * pitch);

        // println!("from {} to {}, width {}",
        //          dst_offset,
        //          src_offset,
        //          area_row_width);
        let dst_slice = &mut dst[dst_offset..(dst_offset + area_row_width)];
        let src_slice = &src[src_offset..(src_offset + area_row_width)];
        dst_slice.copy_from_slice(src_slice);
    }
}

fn render_bitmap(
    context: &mut Context,
    text: &str,
    bytes: &[u8],
    width: usize,
    height: usize,
    glyphs_map: &BTreeMap<char, usize>,
    glyph_width: usize,
) -> GameResult<Text> {
    let text_length = text.len();
    let glyph_height = height;
    // Same at-least-one-pixel-wide constraint here as with TTF fonts.
    let buf_len = cmp::max(text_length * glyph_width * glyph_height * 4, 1);
    let mut dest_buf = Vec::with_capacity(buf_len);
    dest_buf.resize(buf_len, 0u8);
    for (i, c) in text.chars().enumerate() {
        // println!("Rendering character {}: {}", i, c);
        let error = GameError::FontError(format!("Character '{}' not in bitmap font!", c));
        let source_offset = *glyphs_map.get(&c).ok_or(error)?;
        let dest_offset = glyph_width * i;
        // println!("Blitting {:?} to {:?}", source_offset, dest_offset);
        blit(
            &mut dest_buf,
            (text_length * glyph_width, glyph_height),
            (dest_offset, 0),
            bytes,
            (width, height),
            (source_offset, 0),
            (glyph_width, glyph_height),
            4,
        );
    }
    // println!("width {}, height {}", text_length * glyph_width, glyph_height);
    let image = Image::from_rgba8(
        context,
        (text_length * glyph_width) as u16,
        glyph_height as u16,
        &dest_buf,
    )?;
    let text_string = text.to_string();

    Ok(Text {
        texture: image,
        contents: text_string,
        blend_mode: None,
    })
}

impl Text {
    /// Renders a new `Text` from the given `Font`.
    ///
    /// Note that this is relatively computationally expensive;
    /// if you want to draw text every frame you probably want to save
    /// it and only update it when the text changes.
    pub fn new(context: &mut Context, text: &str, font: &Font) -> GameResult<Text> {
        match *font {
            Font::TTFFont {
                font: ref f, scale, ..
            } => render_ttf(context, text, f, scale),
            Font::BitmapFont {
                ref bytes,
                width,
                height,
                glyph_width,
                ref glyphs,
            } => render_bitmap(context, text, bytes, width, height, glyphs, glyph_width),
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
}

impl Drawable for Text {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        draw_ex(ctx, &self.texture, param)
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
    #[test]
    fn test_blit() {
        let dst = &mut [0; 125][..];
        let src = &[
            1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 9, 1, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ][..];
        assert_eq!(src.len(), 25 * 5);

        // Test just blitting the whole thing
        let rect_dims = (25, 5);
        blit(dst, rect_dims, (0, 0), src, rect_dims, (0, 0), (25, 5), 1);
        //println!("{:?}", src);
        //println!("{:?}", dst);
        assert_eq!(dst, src);
        for i in 0..dst.len() {
            dst[i] = 0;
        }

        // Test blitting the whole thing with a non-1 pitch
        let rect_dims = (5, 5);
        blit(dst, rect_dims, (0, 0), src, rect_dims, (0, 0), (5, 5), 5);
        assert_eq!(dst, src);
    }

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
        let ctx = &mut Context::load_from_conf("test_wrapping", "ggez", c)
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
}
