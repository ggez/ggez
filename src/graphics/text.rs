
use std::fmt;
use std::path;
use std::collections::BTreeMap;
use std::convert::From;
use std::io::Read;

use sdl2;
use rusttype;
use image;
use gfx;
use gfx::traits::Device;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_sdl;
use gfx::format::Rgba8;
use gfx::Factory;

use super::*;

/// A font that defines the shape of characters drawn on the screen.
/// Can be created from a .ttf file or from an image (bitmap fonts).
pub enum Font {
    TTFFont {
        font: rusttype::Font<'static>,
        points: u32,
    },
    BitmapFont {
        surface: Image,
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
        let s2 = Image::new(context, path.as_ref())?;

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
                 image: &Image,
                 glyphs_map: &BTreeMap<char, u32>,
                 glyph_width: u32)
                 -> GameResult<Text> {
    let text_length = text.len() as u32;
    let glyph_height = image.height;
    // let format = pixels::PixelFormatEnum::RGBA8888;
    // let mut dest_surface = surface::Surface::new(
    // text_length * glyph_width, glyph_height, format)?;
    // for (i, c) in text.chars().enumerate() {
    //     let small_i = i as u32;
    //     let error_message = format!("Character '{}' not in bitmap font!", c);
    //     let source_offset = glyphs_map.get(&c)
    //         .ok_or(GameError::FontError(String::from(error_message)))?;
    //     let dest_offset = glyph_width * small_i;
    //     let source_rect = Rect::new(*source_offset as f32,
    //                                 0.0,
    //                                 glyph_width as f32,
    //                                 glyph_height as f32);
    //     let dest_rect = Rect::new(dest_offset as f32,
    //                               0.0,
    //                               glyph_width as f32,
    //                               glyph_height as f32);
    //     // println!("Blitting letter {} to {:?}", c, dest_rect);
    //     // surface.blit(Some(source_rect), &mut dest_surface, Some(dest_rect))?;
    // }
    // // let image = Image::from_surface(context, dest_surface)?;
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
        // BUGGO: Fix!
        // Seems that things get frumple when the resulting text texture
        // is not a power of 2, for one thing.
        let txt = Text {
            texture: context.gfx_context.white_image.clone(),
            contents: text.to_string(),
        };
        return Ok(txt);
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
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        unimplemented!();
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
