use std::fmt;
use std::path;
use std::collections::BTreeMap;
use std::convert::From;
use std::io::Read;

use rusttype;
use image;

use super::*;

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image (bitmap fonts).
pub enum Font {
    TTFFont {
        font: rusttype::Font<'static>,
        points: u32,
    },
    BitmapFont {
        // Width, height and data for the original glyph image.
        // This is always going to be RGBA.
        bytes: Vec<u8>,
        width: usize,
        height: usize,
        // Glyph to index mapping
        glyphs: BTreeMap<char, usize>,
        glyph_width: usize,
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
        Font::from_bytes(&name, &buf, size)
    }

    /// Loads a new TTF font from data copied out of the given buffer.
    pub fn from_bytes(name: &str, bytes: &[u8], size: u32) -> GameResult<Font> {
        let collection = rusttype::FontCollection::from_bytes(bytes.to_vec());
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

    pub fn default_font() -> GameResult<Self> {
        let size = 16;
        let buf = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/DejaVuSerif.ttf"));
        Font::from_bytes("default", &buf[..], size)
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

    // Get the proper DPI to scale font size accordingly
    let (_diag_dpi, x_dpi, y_dpi) = context.gfx_context.dpi;
    // println!("DPI: {}, {}", x_dpi, y_dpi);
    let scale = display_independent_scale(size, x_dpi, y_dpi);
    let text_height_pixels = scale.y.ceil() as usize;
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);
    // Then turn them into an array of positioned glyphs...
    // `layout()` turns an abstract glyph, which contains no concrete
    // size or position information, into a PositionedGlyph, which does.
    let glyphs: Vec<rusttype::PositionedGlyph> = font.layout(text, scale, offset).collect();
    let text_width_pixels = glyphs
        .iter()
        .rev()
        .filter_map(|g| {
                        g.pixel_bounding_box()
                            .map(|b| {
                                     b.min.x as f32 + g.unpositioned().h_metrics().advance_width
                                 })
                    })
        .next()
        .unwrap_or(0.0)
        .ceil() as usize;
    // Make an array for our rendered bitmap
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
                if x >= 0 && x < text_width_pixels as i32 && y >= 0 &&
                   y < text_height_pixels as i32 {
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

    // println!("Creating text {}, {}x{}: {:?}",
    //text, text_width_pixels, text_height_pixels, pixel_data);

    // Copy the bitmap into an image, and we're basically done!
    // BUGGO: TODO: Make sure int conversions will not fail
    let image = Image::from_rgba8(context,
                                  text_width_pixels as u16,
                                  text_height_pixels as u16,
                                  &pixel_data)?;

    let text_string = text.to_string();
    Ok(Text {
           texture: image,
           contents: text_string,
       })

}

/// Treats src and dst as row-major 2D arrays, and blits the given rect from src to dst.
/// Does no bounds checking or anything; if you feed it invalid bounds it will just panic.
/// Generally, you shouldn't need or use this.
fn blit(dst: &mut [u8],
        dst_dims: (usize, usize),
        dst_point: (usize, usize),
        src: &[u8],
        src_dims: (usize, usize),
        src_point: (usize, usize),
        rect_size: (usize, usize),
        pitch: usize) {
    // The rect properties are all f32's; we truncate them down to integers.
    let area_row_width = rect_size.0 * pitch;
    let src_row_width = src_dims.0 * pitch;
    let dst_row_width = dst_dims.0 * pitch;

    for row_idx in 0..rect_size.1 {
        let src_row = row_idx + src_point.1;
        let dst_row = row_idx + dst_point.1;
        let src_offset = src_row * src_row_width;
        let dst_offset = dst_row * dst_row_width;

        println!("from {} to {}, width {}",
                 dst_offset,
                 src_offset,
                 area_row_width);
        let dst_slice = &mut dst[dst_offset..(dst_offset + area_row_width)];
        let src_slice = &src[src_offset..(src_offset + area_row_width)];
        dst_slice.copy_from_slice(src_slice);
    }
}

fn render_bitmap(context: &mut Context,
                 text: &str,
                 bytes: &[u8],
                 width: usize,
                 height: usize,
                 glyphs_map: &BTreeMap<char, usize>,
                 glyph_width: usize)
                 -> GameResult<Text> {
    let text_length = text.len();
    let glyph_height = height;
    let buf_len = text_length * glyph_width * glyph_height * 4;
    let mut dest_buf = Vec::with_capacity(buf_len);
    dest_buf.resize(buf_len, 0u8);
    for (i, c) in text.chars().enumerate() {
        let error_message = format!("Character '{}' not in bitmap font!", c);
        let source_offset = *glyphs_map
                                 .get(&c)
                                 .ok_or(GameError::FontError(String::from(error_message)))?;
        let dest_offset = glyph_width * i;
        blit(&mut dest_buf,
             (text_length * glyph_width, glyph_height),
             (dest_offset, 0),
             &bytes,
             (width, height),
             (source_offset, 0),
             (glyph_width, glyph_height),
             4);
    }
    let image = Image::from_rgba8(context,
                                  (text_length * glyph_width) as u16,
                                  glyph_height as u16,
                                  &dest_buf)?;
    let text_string = text.to_string();

    Ok(Text {
           texture: image,
           contents: text_string,
       })
}


impl Text {
    /// Renders a new `Text` from the given `Font`
    pub fn new(context: &mut Context, text: &str, font: &Font) -> GameResult<Text> {
        match *font {
            Font::TTFFont {
                font: ref f,
                points,
            } => render_ttf(context, text, f, points),
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
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        draw_ex(ctx, &self.texture, param)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_blit() {
        let dst = &mut [0; 125][..];
        let src = &[1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 9,
                    1, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 0, 1, 0,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
                       [..];
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
}
