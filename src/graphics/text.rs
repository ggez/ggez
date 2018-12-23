use gfx_glyph::{self, Layout, SectionText, VariedSection};
pub use gfx_glyph::{FontId, HorizontalAlign as Align, Scale};
use glyph_brush::GlyphPositioner;
use mint;
use std::borrow::Cow;
use std::f32;
use std::fmt;
use std::io::Read;
use std::path;
use std::sync::{Arc, RwLock};

use super::*;

// TODO: consider adding bits from example to docs.

/// Default size for fonts.
pub const DEFAULT_FONT_SCALE: f32 = 16.0;

/// A handle referring to a loaded Truetype font.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Font {
    font_id: FontId,
    // Add DebugId?  It makes Font::default() less convenient.
}

/// A piece of text with optional color, font and font scale information.
/// These options take precedence over any similar field/argument.
/// Can be implicitly constructed from `String`, `(String, Color)`, and `(String, FontId, Scale)`.
///
/// TODO: Construction should be full builder pattern, if it's not already.
#[derive(Clone, Debug)]
pub struct TextFragment {
    /// Text string itself.
    pub text: String,
    /// Fragment's color, defaults to text's color.
    pub color: Option<Color>,
    /// Fragment's font, defaults to text's font.
    pub font: Option<Font>,
    /// Fragment's scale, defaults to text's scale.
    pub scale: Option<Scale>,
}

impl Default for TextFragment {
    fn default() -> Self {
        TextFragment {
            text: "".into(),
            color: None,
            font: None,
            scale: None,
        }
    }
}

impl TextFragment {
    /// Creates a new fragment from `String` or `&str`.
    pub fn new<T: Into<Self>>(text: T) -> Self {
        text.into()
    }

    /// Set fragment's color, overrides text's color.
    pub fn color(mut self, color: Color) -> TextFragment {
        self.color = Some(color);
        self
    }

    /// Set fragment's font, overrides text's font.
    pub fn font(mut self, font: Font) -> TextFragment {
        self.font = Some(font);
        self
    }

    /// Set fragment's scale, overrides text's scale.
    pub fn scale(mut self, scale: Scale) -> TextFragment {
        self.scale = Some(scale);
        self
    }
}

impl<'a> From<&'a str> for TextFragment {
    fn from(text: &'a str) -> TextFragment {
        TextFragment {
            text: text.to_owned(),
            ..Default::default()
        }
    }
}

impl From<char> for TextFragment {
    fn from(ch: char) -> TextFragment {
        TextFragment {
            text: ch.to_string(),
            ..Default::default()
        }
    }
}

impl From<String> for TextFragment {
    fn from(text: String) -> TextFragment {
        TextFragment {
            text,
            ..Default::default()
        }
    }
}

// TODO: Scale ergonomics need to be better
impl<T> From<(T, Font, f32)> for TextFragment
where
    T: Into<TextFragment>,
{
    fn from((text, font, scale): (T, Font, f32)) -> TextFragment {
        text.into().font(font).scale(Scale::uniform(scale))
    }
}

/// Cached font metrics that we can keep attached to a `Text`
/// so we don't have to keep recalculating them.
#[derive(Clone, Debug)]
struct CachedMetrics {
    string: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

impl Default for CachedMetrics {
    fn default() -> CachedMetrics {
        CachedMetrics {
            string: None,
            width: None,
            height: None,
        }
    }
}

/// Drawable text object.  Essentially a list of [`TextFragment`](struct.TextFragment.html)'s
/// and some metrics information.
///
/// It implements [`Drawable`](trait.Drawable.html) so it can be drawn immediately with
/// [`graphics::draw()`](fn.draw.html), or many of them can be queued with [`graphics::queue_text()`](fn.queue_text.html)
/// and then all drawn at once with [`graphics::draw_queued_text()`](fn.draw_queued_text.html).
#[derive(Debug)]
pub struct Text {
    fragments: Vec<TextFragment>,
    // TODO: make it do something, maybe.
    blend_mode: Option<BlendMode>,
    bounds: Point2,
    layout: Layout<gfx_glyph::BuiltInLineBreaker>,
    font_id: FontId,
    font_scale: Scale,
    cached_metrics: Arc<RwLock<CachedMetrics>>,
}

// This has to be explicit. Derived `Clone` clones the `Arc`, so clones end up sharing the metrics.
impl Clone for Text {
    fn clone(&self) -> Self {
        Text {
            fragments: self.fragments.clone(),
            blend_mode: self.blend_mode,
            bounds: self.bounds,
            layout: self.layout,
            font_id: self.font_id,
            font_scale: self.font_scale,
            cached_metrics: Arc::new(RwLock::new(CachedMetrics::default())),
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Text {
            fragments: Vec::new(),
            blend_mode: None,
            bounds: Point2::new(f32::INFINITY, f32::INFINITY),
            layout: Layout::default(),
            font_id: FontId::default(),
            font_scale: Scale::uniform(DEFAULT_FONT_SCALE),
            cached_metrics: Arc::new(RwLock::new(CachedMetrics::default())),
        }
    }
}

impl Text {
    /// Creates a `Text` from a `TextFragment`.
    ///
    /// ```rust
    /// # use ggez::graphics::Text;
    /// # fn main() {
    /// let text = Text::new("foo");
    /// # }
    /// ```
    pub fn new<F>(fragment: F) -> Text
    where
        F: Into<TextFragment>,
    {
        let mut text = Text::default();
        let _ = text.add(fragment);
        text
    }

    /// Appends a `TextFragment` to the `Text`.
    pub fn add<F>(&mut self, fragment: F) -> &mut Text
    where
        F: Into<TextFragment>,
    {
        self.fragments.push(fragment.into());
        self.invalidate_cached_metrics();
        self
    }

    /// Returns a read-only slice of all `TextFragment`'s.
    pub fn fragments(&self) -> &[TextFragment] {
        &self.fragments
    }

    /// Returns a mutable slice with all fragments.
    pub fn fragments_mut(&mut self) -> &mut [TextFragment] {
        &mut self.fragments
    }

    /// Specifies rectangular dimensions to try and fit contents inside of,
    /// by wrapping, and alignment within the bounds.
    pub fn set_bounds<P>(&mut self, bounds: P, alignment: Align) -> &mut Text
    where
        P: Into<mint::Point2<f32>>,
    {
        self.bounds = Point2::from(bounds.into());
        if self.bounds.x == f32::INFINITY {
            // Layouts don't make any sense if we don't wrap text at all.
            self.layout = Layout::default();
        } else {
            self.layout = self.layout.h_align(alignment);
        }
        self.invalidate_cached_metrics();
        self
    }

    /// Specifies text's font and font scale; used for fragments that don't have their own.
    pub fn set_font(&mut self, font: Font, font_scale: Scale) -> &mut Text {
        self.font_id = font.font_id;
        self.font_scale = font_scale;
        self.invalidate_cached_metrics();
        self
    }

    /// Converts `Text` to a type `gfx_glyph` can understand and queue.
    fn generate_varied_section(
        &self,
        relative_dest: Point2,
        color: Option<Color>,
    ) -> VariedSection {
        let mut sections = Vec::with_capacity(self.fragments.len());
        for fragment in &self.fragments {
            let color = match fragment.color {
                Some(c) => c,
                None => match color {
                    Some(c) => c,
                    None => WHITE,
                },
            };
            let font_id = match fragment.font {
                Some(font) => font.font_id,
                None => self.font_id,
            };
            let scale = match fragment.scale {
                Some(scale) => scale,
                None => self.font_scale,
            };
            sections.push(SectionText {
                text: &fragment.text,
                color: <[f32; 4]>::from(color),
                font_id,
                scale,
            });
        }
        let relative_dest_x = {
            // This positions text within bounds with relative_dest being to the left, always.
            let mut dest_x = relative_dest.x;
            if self.bounds.x != f32::INFINITY {
                use gfx_glyph::Layout::Wrap;
                if let Wrap { h_align, .. } = self.layout {
                    match h_align {
                        Align::Center => dest_x += self.bounds.x * 0.5,
                        Align::Right => dest_x += self.bounds.x,
                        _ => (),
                    }
                }
            }
            dest_x
        };
        let relative_dest = (relative_dest_x, relative_dest.y);
        VariedSection {
            screen_position: relative_dest,
            bounds: (self.bounds.x, self.bounds.y),
            //z: f32,
            layout: self.layout,
            text: sections,
            ..Default::default()
        }
    }

    fn invalidate_cached_metrics(&mut self) {
        if let Ok(mut metrics) = self.cached_metrics.write() {
            *metrics = CachedMetrics::default();
            // Returning early avoids a double-borrow in the "else"
            // part.
            return;
        }
        warn!("Cached metrics RwLock has been poisoned.");
        self.cached_metrics = Arc::new(RwLock::new(CachedMetrics::default()));
    }

    /// Returns the string that the text represents.
    pub fn contents(&self) -> String {
        if let Ok(metrics) = self.cached_metrics.read() {
            if let Some(ref string) = metrics.string {
                return string.clone();
            }
        }
        let mut string_accm = String::new();
        for frg in &self.fragments {
            string_accm += &frg.text;
        }
        if let Ok(mut metrics) = self.cached_metrics.write() {
            metrics.string = Some(string_accm.clone());
        }
        string_accm
    }

    // /// Calculates, caches, and returns width and height of formatted and wrapped text.
    // fn calculate_dimensions(&self, context: &Context) -> (u32, u32) {
    //     let mut max_width = 0;
    //     let mut max_height = 0;
    //     {
    //         let varied_section = self.generate_varied_section(Point2::new(0.0, 0.0), None);
    //         let glyphed_section_texts = self
    //             .layout

    //             .calculate_glyphs(context.gfx_context.fonts, &varied_section);
    //             // .calculate_glyphs(context.gfx_context.glyph_brush.fonts(), &varied_section);
    //         for glyphed_section_text in &glyphed_section_texts {
    //             let (ref positioned_glyph, ..) = glyphed_section_text;
    //             if let Some(rect) = positioned_glyph.pixel_bounding_box() {
    //                 if rect.max.x > max_width {
    //                     max_width = rect.max.x;
    //                 }
    //                 if rect.max.y > max_height {
    //                     max_height = rect.max.y;
    //                 }
    //             }
    //         }
    //     }
    //     let (width, height) = (max_width as u32, max_height as u32);
    //     if let Ok(mut metrics) = self.cached_metrics.write() {
    //         metrics.width = Some(width);
    //         metrics.height = Some(height);
    //     }
    //     (width, height)
    // }

    // /// Returns the width and height of the formatted and wrapped text.
    // ///
    // /// TODO: Should these return f32 rather than u32?
    // pub fn dimensions(&self, context: &Context) -> (u32, u32) {
    //     if let Ok(metrics) = self.cached_metrics.read() {
    //         if let (Some(width), Some(height)) = (metrics.width, metrics.height) {
    //             return (width, height);
    //         }
    //     }
    //     self.calculate_dimensions(context)
    // }

    // /// Returns the width of formatted and wrapped text, in screen coordinates.
    // pub fn width(&self, context: &Context) -> u32 {
    //     self.dimensions(context).0
    // }

    // /// Returns the height of formatted and wrapped text, in screen coordinates.
    // pub fn height(&self, context: &Context) -> u32 {
    //     self.dimensions(context).1
    // }
}

impl Drawable for Text {
    fn draw<D>(&self, ctx: &mut Context, param: D) -> GameResult
    where
        D: Into<DrawParam>,
    {
        let param = param.into();
        // Converts fraction-of-bounding-box to screen coordinates, as required by `draw_queued()`.
        // TODO: Fix for DrawTransform
        // let offset = Point2::new(
        //     param.offset.x * self.width(ctx) as f32,
        //     param.offset.y * self.height(ctx) as f32,
        // );
        // let param = param.offset(offset);
        queue_text(ctx, self, Point2::new(0.0, 0.0), Some(param.color));
        draw_queued_text(ctx, param)
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}

impl Font {
    /// Load a new TTF font from the given file.
    pub fn new<P>(context: &mut Context, path: P) -> GameResult<Font>
    where
        P: AsRef<path::Path> + fmt::Debug,
    {
        use crate::filesystem;
        let mut stream = filesystem::open(context, path.as_ref())?;
        let mut buf = Vec::new();
        let _ = stream.read_to_end(&mut buf)?;

        // TODO: DPI; see winit #548.  Also need point size, pixels, etc...
        Font::new_glyph_font_bytes(context, &buf)
    }

    /// Loads a new TrueType font from given bytes and into a `gfx::GlyphBrush` owned
    /// by the `Context`.
    pub fn new_glyph_font_bytes(context: &mut Context, bytes: &[u8]) -> GameResult<Self> {
        // Take a Cow here to avoid this clone where unnecessary?
        // Nah, let's not complicate things more than necessary.
        let v = bytes.to_vec();
        let font_id = context.gfx_context.glyph_brush.add_font_bytes(v);

        Ok(Font { font_id })
    }

    /// Returns the baked-in bytes of default font (currently `DejaVuSerif.ttf`).
    pub(crate) fn default_font_bytes() -> &'static [u8] {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/DejaVuSerif.ttf"
        ))
    }
}

impl Default for Font {
    fn default() -> Self {
        Font { font_id: FontId(0) }
    }
}

/// Queues the `Text` to be drawn by [`draw_queued()`](fn.draw_queued.html).
/// `relative_dest` is relative to the [`DrawParam::dest`](struct.DrawParam.html#structfield.dest)
/// passed to `draw_queued()`./ Note, any `Text` drawn via [`graphics::draw()`](fn.draw.html)
/// will also draw the queue.
pub fn queue_text<P>(context: &mut Context, batch: &Text, relative_dest: P, color: Option<Color>)
where
    P: Into<mint::Point2<f32>>,
{
    let p = Point2::from(relative_dest.into());
    let varied_section = batch.generate_varied_section(p, color);
    context.gfx_context.glyph_brush.queue(varied_section);
}

/// Exposes `gfx_glyph`'s `GlyphBrush::queue()` and `GlyphBrush::queue_custom_layout()`,
/// in case `ggez`' API is insufficient.
pub fn queue_text_raw<'a, S, G>(context: &mut Context, section: S, custom_layout: Option<&G>)
where
    S: Into<Cow<'a, VariedSection<'a>>>,
    G: GlyphPositioner,
{
    let brush = &mut context.gfx_context.glyph_brush;
    match custom_layout {
        Some(layout) => brush.queue_custom_layout(section, layout),
        None => brush.queue(section),
    }
}

/// Draws all of the [`Text`](struct.Text.html)s added via [`queue_text()`](fn.queue_text.html).
///
/// `DrawParam` apply to everything in the queue; offset is in screen coordinates;
/// color is ignored - specify it when `queue_text()`ing instead.
pub fn draw_queued_text<D>(context: &mut Context, param: D) -> GameResult
where
    D: Into<DrawTransform>,
{
    let param: DrawTransform = param.into();
    let screen_rect = screen_coordinates(context);

    let (screen_x, screen_y, screen_w, screen_h) =
        (screen_rect.x, screen_rect.y, screen_rect.w, screen_rect.h);
    let scale_x = screen_w / 2.0;
    let scale_y = screen_h / -2.0;

    // gfx_glyph rotates things around the center (1.0, -1.0) with
    // a window rect of (x, y, w, h) = (0.0, 0.0, 2.0, -2.0).
    // We need to a) translate it so that the rotation has the
    // top right corner as its origin, b) scale it so that the
    // translation can happen in screen coordinates, and
    // c) translate it *again* in case our screen coordinates
    // don't start at (0.0, 0.0) for the top left corner.
    // And obviously, the whole tour back!

    // Unoptimized implementation for final_matrix below
    /*
    type Vec3 = na::Vector3<f32>;
    type Mat4 = na::Matrix4<f32>;

    let m_translate_glyph = Mat4::new_translation(&Vec3::new(1.0, -1.0, 0.0));
    let m_translate_glyph_inv = Mat4::new_translation(&Vec3::new(-1.0, 1.0, 0.0));
    let m_scale = Mat4::new_nonuniform_scaling(&Vec3::new(scale_x, scale_y, 1.0));
    let m_scale_inv = Mat4::new_nonuniform_scaling(&Vec3::new(1.0 / scale_x, 1.0 / scale_y, 1.0));
    let m_translate = Mat4::new_translation(&Vec3::new(-screen_x, -screen_y, 0.0));
    let m_translate_inv = Mat4::new_translation(&Vec3::new(screen_x, screen_y, 0.0));

    let final_matrix = m_translate_glyph_inv * m_scale_inv * m_translate_inv * param.matrix * m_translate * m_scale * m_translate_glyph;
    */
    // Optimized version has a speedup of ~1.29 (175ns vs 225ns)
    type Mat4 = na::Matrix4<f32>;
    let m_transform = Mat4::new(
        scale_x,
        0.0,
        0.0,
        scale_x - screen_x,
        0.0,
        scale_y,
        0.0,
        -scale_y - screen_y,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    let m_transform_inv = Mat4::new(
        1.0 / scale_x,
        0.0,
        0.0,
        (screen_x / scale_x) - 1.0,
        0.0,
        1.0 / scale_y,
        0.0,
        (scale_y + screen_y) / scale_y,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    let final_matrix = m_transform_inv * param.matrix * m_transform;

    let color_format = context.gfx_context.color_format();
    let depth_format = context.gfx_context.depth_format();
    let (encoder, render_tgt, depth_view) = (
        &mut context.gfx_context.encoder,
        &context.gfx_context.screen_render_target,
        &context.gfx_context.depth_view,
    );

    context
        .gfx_context
        .glyph_brush
        .draw_queued_with_transform(
            final_matrix.into(),
            encoder,
            &(render_tgt, color_format),
            &(depth_view, depth_format),
        )
        .map_err(|e| GameError::RenderError(e.to_string()))
}

#[cfg(test)]
mod tests {
    /*
        use super::*;
        #[test]
        fn test_metrics() {
            let f = Font::default_font().expect("Could not get default font");
            assert_eq!(f.height(), 17);
            assert_eq!(f.width("Foo!"), 33);

            // http://www.catipsum.com/index.php
            let text_to_wrap = "Walk on car leaving trail of paw prints on hood and windshield sniff \
                                other cat's butt and hang jaw half open thereafter for give attitude. \
                                Annoy kitten\nbrother with poking. Mrow toy mouse squeak roll over. \
                                Human give me attention meow.";
            let (len, v) = f.wrap(text_to_wrap, 250);
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
    let (len, v) = font.wrap(text_to_wrap, wrap_length);
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

mod ggez_glyph {
    //! Fast GPU cached text rendering using gfx-rs & rusttype.
    //!
    //! Makes use of three kinds of caching to optimise frame performance.
    //!
    //! * Caching of glyph positioning output to avoid repeated cost of identical text
    //! rendering on sequential frames.
    //! * Caches draw calculations to avoid repeated cost of identical text rendering on
    //! sequential frames.
    //! * GPU cache logic to dynamically maintain a GPU texture of rendered glyphs.
    //!
    //! # Example
    //!
    //! ```no_run
    //! # extern crate gfx;
    //! # extern crate gfx_window_glutin;
    //! # extern crate glutin;
    //! extern crate gfx_glyph;
    //! use gfx_glyph::{GlyphBrushBuilder, Section};
    //! # fn main() -> Result<(), String> {
    //! # let events_loop = glutin::EventsLoop::new();
    //! # let (_window, _device, mut gfx_factory, gfx_color, gfx_depth) =
    //! #     gfx_window_glutin::init::<gfx::format::Srgba8, gfx::format::Depth>(
    //! #         glutin::WindowBuilder::new(),
    //! #         glutin::ContextBuilder::new(),
    //! #         &events_loop).unwrap();
    //! # let mut gfx_encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();
    //!
    //! let dejavu: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
    //! let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(dejavu).build(gfx_factory.clone());
    //!
    //! # let some_other_section = Section { text: "another", ..Section::default() };
    //! let section = Section {
    //!     text: "Hello gfx_glyph",
    //!     ..Section::default()
    //! };
    //!
    //! glyph_brush.queue(section);
    //! glyph_brush.queue(some_other_section);
    //!
    //! glyph_brush.draw_queued(&mut gfx_encoder, &gfx_color, &gfx_depth)?;
    //! # Ok(())
    //! # }
    //! ```
    //!

    // builder.rs
    use glyph_brush::delegate_glyph_brush_builder_fns;

    /// Builder for a [`GlyphBrush`](struct.GlyphBrush.html).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # extern crate gfx;
    /// # extern crate gfx_window_glutin;
    /// # extern crate glutin;
    /// extern crate gfx_glyph;
    /// use gfx_glyph::GlyphBrushBuilder;
    /// # fn main() {
    /// # let events_loop = glutin::EventsLoop::new();
    /// # let (_window, _device, gfx_factory, _gfx_target, _main_depth) =
    /// #     gfx_window_glutin::init::<gfx::format::Srgba8, gfx::format::Depth>(
    /// #         glutin::WindowBuilder::new(),
    /// #         glutin::ContextBuilder::new(),
    /// #         &events_loop).unwrap();
    ///
    /// let dejavu: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
    /// let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(dejavu).build(gfx_factory.clone());
    /// # let _ = glyph_brush;
    /// # }
    /// ```
    pub struct GlyphBrushBuilder<'a, H = DefaultSectionHasher> {
        inner: glyph_brush::GlyphBrushBuilder<'a, H>,
        depth_test: gfx::state::Depth,
        texture_filter_method: texture::FilterMethod,
    }

    impl<'a> GlyphBrushBuilder<'a> {
        /// Specifies the default font data used to render glyphs.
        /// Referenced with `FontId(0)`, which is default.
        #[inline]
        pub fn using_font_bytes<B: Into<SharedBytes<'a>>>(font_0_data: B) -> Self {
            Self::using_font(Font::from_bytes(font_0_data).unwrap())
        }

        #[inline]
        pub fn using_fonts_bytes<B, V>(font_data: V) -> Self
        where
            B: Into<SharedBytes<'a>>,
            V: Into<Vec<B>>,
        {
            Self::using_fonts(
                font_data
                    .into()
                    .into_iter()
                    .map(|data| Font::from_bytes(data).unwrap())
                    .collect::<Vec<_>>(),
            )
        }

        /// Specifies the default font used to render glyphs.
        /// Referenced with `FontId(0)`, which is default.
        #[inline]
        pub fn using_font(font_0: Font<'a>) -> Self {
            Self::using_fonts(vec![font_0])
        }

        pub fn using_fonts<V: Into<Vec<Font<'a>>>>(fonts: V) -> Self {
            GlyphBrushBuilder {
                inner: glyph_brush::GlyphBrushBuilder::using_fonts(fonts),
                depth_test: gfx::preset::depth::PASS_TEST,
                texture_filter_method: texture::FilterMethod::Bilinear,
            }
        }
    }

    impl<'a, H: BuildHasher> GlyphBrushBuilder<'a, H> {
        delegate_glyph_brush_builder_fns!(inner);

        /// Sets the depth test to use on the text section **z** values.
        ///
        /// Defaults to: *Always pass the depth test, never write to the depth buffer write*
        ///
        /// # Example
        ///
        /// ```no_run
        /// # extern crate gfx;
        /// # extern crate gfx_glyph;
        /// # use gfx_glyph::GlyphBrushBuilder;
        /// # fn main() {
        /// # let some_font: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
        /// GlyphBrushBuilder::using_font_bytes(some_font)
        ///     .depth_test(gfx::preset::depth::LESS_EQUAL_WRITE)
        ///     // ...
        /// # ;
        /// # }
        /// ```
        pub fn depth_test(mut self, depth_test: gfx::state::Depth) -> Self {
            self.depth_test = depth_test;
            self
        }

        /// Sets the texture filtering method.
        ///
        /// Defaults to `Bilinear`
        ///
        /// # Example
        /// ```no_run
        /// # extern crate gfx;
        /// # extern crate gfx_glyph;
        /// # use gfx_glyph::GlyphBrushBuilder;
        /// # fn main() {
        /// # let some_font: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
        /// GlyphBrushBuilder::using_font_bytes(some_font)
        ///     .texture_filter_method(gfx::texture::FilterMethod::Scale)
        ///     // ...
        /// # ;
        /// # }
        /// ```
        pub fn texture_filter_method(mut self, filter_method: texture::FilterMethod) -> Self {
            self.texture_filter_method = filter_method;
            self
        }

        /// Sets the section hasher. `GlyphBrush` cannot handle absolute section hash collisions
        /// so use a good hash algorithm.
        ///
        /// This hasher is used to distinguish sections, rather than for hashmap internal use.
        ///
        /// Defaults to [seahash](https://docs.rs/seahash).
        ///
        /// # Example
        /// ```no_run
        /// # extern crate gfx;
        /// # extern crate gfx_glyph;
        /// # use gfx_glyph::GlyphBrushBuilder;
        /// # fn main() {
        /// # let some_font: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
        /// # type SomeOtherBuildHasher = std::collections::hash_map::RandomState;
        /// GlyphBrushBuilder::using_font_bytes(some_font)
        ///     .section_hasher(SomeOtherBuildHasher::default())
        ///     // ...
        /// # ;
        /// # }
        /// ```
        pub fn section_hasher<T: BuildHasher>(self, section_hasher: T) -> GlyphBrushBuilder<'a, T> {
            GlyphBrushBuilder {
                inner: self.inner.section_hasher(section_hasher),
                depth_test: self.depth_test,
                texture_filter_method: self.texture_filter_method,
            }
        }

        /// Builds a `GlyphBrush` using the input gfx factory
        pub fn build<R, F>(self, mut factory: F) -> GlyphBrush<'a, R, F, H>
        where
            R: gfx::Resources,
            F: gfx::Factory<R>,
        {
            let (cache_width, cache_height) = self.inner.initial_cache_size;
            let font_cache_tex = create_texture(&mut factory, cache_width, cache_height).unwrap();
            let program = factory
                .link_program(
                    include_bytes!("shader/glyphbrush.glslv"),
                    include_bytes!("shader/glyphbrush.glslf"),
                )
                .unwrap();

            GlyphBrush {
                font_cache_tex,
                texture_filter_method: self.texture_filter_method,
                glyph_brush: self.inner.build(),

                factory,
                program,
                draw_cache: None,

                depth_test: self.depth_test,
            }
        }
    }

    // pipe.rs
    use gfx::{
        self,
        format::{Format, Formatted},
        handle::{DepthStencilView, RawDepthStencilView, RawRenderTargetView, RenderTargetView},
        memory::Typed,
        pso::*,
        *,
    };
    use gfx_core::pso;
    // use gfx_core::pso;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RawDepthTarget;

    impl DataLink<'_> for RawDepthTarget {
        type Init = (format::Format, state::Depth);
        fn new() -> Self {
            RawDepthTarget
        }
        fn is_active(&self) -> bool {
            true
        }
        fn link_depth_stencil(&mut self, init: &Self::Init) -> Option<pso::DepthStencilDesc> {
            Some((init.0, init.1.into()))
        }
    }

    impl<R: Resources> DataBind<R> for RawDepthTarget {
        type Data = handle::RawDepthStencilView<R>;
        fn bind_to(
            &self,
            out: &mut RawDataSet<R>,
            data: &Self::Data,
            man: &mut handle::Manager<R>,
            _: &mut AccessInfo<R>,
        ) {
            let dsv = data;
            out.pixel_targets.add_depth_stencil(
                man.ref_dsv(dsv),
                true,
                false,
                dsv.get_dimensions(),
            );
        }
    }

    gfx_defines! {
        vertex GlyphVertex {
            /// screen position
            left_top: [f32; 3] = "left_top",
            right_bottom: [f32; 2] = "right_bottom",
            /// texture position
            tex_left_top: [f32; 2] = "tex_left_top",
            tex_right_bottom: [f32; 2] = "tex_right_bottom",
            /// text color
            color: [f32; 4] = "color",
        }
    }

    gfx_pipeline_base!( glyph_pipe {
    vbuf: InstanceBuffer<GlyphVertex>,
    font_tex: gfx::pso::resource::TextureSampler<TexFormView>,
    transform: Global<[[f32; 4]; 4]>,
    out: RawRenderTarget,
    out_depth: RawDepthTarget,
});

    impl glyph_pipe::Init<'_> {
        pub fn new(
            color_format: format::Format,
            depth_format: format::Format,
            depth_test: state::Depth,
        ) -> Self {
            glyph_pipe::Init {
                vbuf: (),
                font_tex: "font_tex",
                transform: "transform",
                out: (
                    "Target0",
                    color_format,
                    state::ColorMask::all(),
                    Some(preset::blend::ALPHA),
                ),
                out_depth: (depth_format, depth_test),
            }
        }
    }

    /// A view that can produce an inner "raw" view & a `Format`.
    pub trait RawAndFormat {
        type Raw;
        fn as_raw(&self) -> &Self::Raw;
        fn format(&self) -> Format;
    }

    impl<R: Resources, T: Formatted> RawAndFormat for RenderTargetView<R, T> {
        type Raw = RawRenderTargetView<R>;
        #[inline]
        fn as_raw(&self) -> &Self::Raw {
            self.raw()
        }
        #[inline]
        fn format(&self) -> Format {
            T::get_format()
        }
    }

    impl<R: Resources, T: Formatted> RawAndFormat for DepthStencilView<R, T> {
        type Raw = RawDepthStencilView<R>;
        #[inline]
        fn as_raw(&self) -> &Self::Raw {
            self.raw()
        }
        #[inline]
        fn format(&self) -> Format {
            T::get_format()
        }
    }

    impl<R> RawAndFormat for (&R, Format) {
        type Raw = R;
        #[inline]
        fn as_raw(&self) -> &Self::Raw {
            self.0
        }
        #[inline]
        fn format(&self) -> Format {
            self.1
        }
    }

    pub use glyph_brush::{
        rusttype::{self, Font, Point, PositionedGlyph, Rect, Scale, SharedBytes},
        BuiltInLineBreaker, FontId, FontMap, GlyphCruncher, HorizontalAlign, Layout, LineBreak,
        LineBreaker, OwnedSectionText, OwnedVariedSection, PositionedGlyphIter, Section,
        SectionText, VariedSection, VerticalAlign,
    };

    use gfx::{format, handle, texture, traits::FactoryExt};
    use glyph_brush::{
        rusttype::point, BrushAction, BrushError, DefaultSectionHasher, GlyphPositioner,
    };
    use log::{log_enabled, warn};
    use std::{
        borrow::Cow,
        error::Error,
        fmt,
        hash::{BuildHasher, Hash},
        i32,
    };

    // Type for the generated glyph cache texture
    type TexForm = format::U8Norm;
    type TexSurface = <TexForm as format::Formatted>::Surface;
    type TexChannel = <TexForm as format::Formatted>::Channel;
    type TexFormView = <TexForm as format::Formatted>::View;
    type TexSurfaceHandle<R> = handle::Texture<R, TexSurface>;
    type TexShaderView<R> = handle::ShaderResourceView<R, TexFormView>;

    const IDENTITY_MATRIX4: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    /// Object allowing glyph drawing, containing cache state. Manages glyph positioning cacheing,
    /// glyph draw caching & efficient GPU texture cache updating and re-sizing on demand.
    ///
    /// Build using a [`GlyphBrushBuilder`](struct.GlyphBrushBuilder.html).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # extern crate gfx;
    /// # extern crate gfx_window_glutin;
    /// # extern crate glutin;
    /// extern crate gfx_glyph;
    /// # use gfx_glyph::{GlyphBrushBuilder};
    /// use gfx_glyph::Section;
    /// # fn main() -> Result<(), String> {
    /// # let events_loop = glutin::EventsLoop::new();
    /// # let (_window, _device, mut gfx_factory, gfx_color, gfx_depth) =
    /// #     gfx_window_glutin::init::<gfx::format::Srgba8, gfx::format::Depth>(
    /// #         glutin::WindowBuilder::new(),
    /// #         glutin::ContextBuilder::new(),
    /// #         &events_loop).unwrap();
    /// # let mut gfx_encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();
    /// # let dejavu: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
    /// # let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(dejavu)
    /// #     .build(gfx_factory.clone());
    /// # let some_other_section = Section { text: "another", ..Section::default() };
    ///
    /// let section = Section {
    ///     text: "Hello gfx_glyph",
    ///     ..Section::default()
    /// };
    ///
    /// glyph_brush.queue(section);
    /// glyph_brush.queue(some_other_section);
    ///
    /// glyph_brush.draw_queued(&mut gfx_encoder, &gfx_color, &gfx_depth)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Caching behaviour
    ///
    /// Calls to [`GlyphBrush::queue`](#method.queue),
    /// [`GlyphBrush::pixel_bounds`](#method.pixel_bounds), [`GlyphBrush::glyphs`](#method.glyphs)
    /// calculate the positioned glyphs for a section.
    /// This is cached so future calls to any of the methods for the same section are much
    /// cheaper. In the case of [`GlyphBrush::queue`](#method.queue) the calculations will also be
    /// used for actual drawing.
    ///
    /// The cache for a section will be **cleared** after a
    /// [`GlyphBrush::draw_queued`](#method.draw_queued) call when that section has not been used since
    /// the previous draw call.
    pub struct GlyphBrush<'font, R: gfx::Resources, F: gfx::Factory<R>, H = DefaultSectionHasher> {
        font_cache_tex: (
            gfx::handle::Texture<R, TexSurface>,
            gfx::handle::ShaderResourceView<R, f32>,
        ),
        texture_filter_method: texture::FilterMethod,
        factory: F,
        program: gfx::handle::Program<R>,
        draw_cache: Option<DrawnGlyphBrush<R>>,
        glyph_brush: glyph_brush::GlyphBrush<'font, H>,

        // config
        depth_test: gfx::state::Depth,
    }

    impl<R: gfx::Resources, F: gfx::Factory<R>, H> fmt::Debug for GlyphBrush<'_, R, F, H> {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "GlyphBrush")
        }
    }

    impl<'font, R: gfx::Resources, F: gfx::Factory<R>, H: BuildHasher> GlyphCruncher<'font>
        for GlyphBrush<'font, R, F, H>
    {
        #[inline]
        fn pixel_bounds_custom_layout<'a, S, L>(
            &mut self,
            section: S,
            custom_layout: &L,
        ) -> Option<Rect<i32>>
        where
            L: GlyphPositioner + Hash,
            S: Into<Cow<'a, VariedSection<'a>>>,
        {
            self.glyph_brush
                .pixel_bounds_custom_layout(section, custom_layout)
        }

        #[inline]
        fn glyphs_custom_layout<'a, 'b, S, L>(
            &'b mut self,
            section: S,
            custom_layout: &L,
        ) -> PositionedGlyphIter<'b, 'font>
        where
            L: GlyphPositioner + Hash,
            S: Into<Cow<'a, VariedSection<'a>>>,
        {
            self.glyph_brush
                .glyphs_custom_layout(section, custom_layout)
        }
    }

    impl<'font, R: gfx::Resources, F: gfx::Factory<R>, H: BuildHasher> GlyphBrush<'font, R, F, H> {
        /// Queues a section/layout to be drawn by the next call of
        /// [`draw_queued`](struct.GlyphBrush.html#method.draw_queued). Can be called multiple times
        /// to queue multiple sections for drawing.
        ///
        /// Used to provide custom `GlyphPositioner` logic, if using built-in
        /// [`Layout`](enum.Layout.html) simply use [`queue`](struct.GlyphBrush.html#method.queue)
        ///
        /// Benefits from caching, see [caching behaviour](#caching-behaviour).
        #[inline]
        pub fn queue_custom_layout<'a, S, G>(&mut self, section: S, custom_layout: &G)
        where
            G: GlyphPositioner,
            S: Into<Cow<'a, VariedSection<'a>>>,
        {
            self.glyph_brush.queue_custom_layout(section, custom_layout)
        }

        /// Queues a section/layout to be drawn by the next call of
        /// [`draw_queued`](struct.GlyphBrush.html#method.draw_queued). Can be called multiple times
        /// to queue multiple sections for drawing.
        ///
        /// Benefits from caching, see [caching behaviour](#caching-behaviour).
        #[inline]
        pub fn queue<'a, S>(&mut self, section: S)
        where
            S: Into<Cow<'a, VariedSection<'a>>>,
        {
            self.glyph_brush.queue(section)
        }

        /// Retains the section in the cache as if it had been used in the last draw-frame.
        ///
        /// Should not be necessary unless using multiple draws per frame with distinct transforms,
        /// see [caching behaviour](#caching-behaviour).
        #[inline]
        pub fn keep_cached_custom_layout<'a, S, G>(&mut self, section: S, custom_layout: &G)
        where
            S: Into<Cow<'a, VariedSection<'a>>>,
            G: GlyphPositioner,
        {
            self.glyph_brush
                .keep_cached_custom_layout(section, custom_layout)
        }

        /// Retains the section in the cache as if it had been used in the last draw-frame.
        ///
        /// Should not be necessary unless using multiple draws per frame with distinct transforms,
        /// see [caching behaviour](#caching-behaviour).
        #[inline]
        pub fn keep_cached<'a, S>(&mut self, section: S)
        where
            S: Into<Cow<'a, VariedSection<'a>>>,
        {
            self.glyph_brush.keep_cached(section)
        }

        /// Draws all queued sections onto a render target, applying a position transform (e.g.
        /// a projection).
        /// See [`queue`](struct.GlyphBrush.html#method.queue).
        ///
        /// Trims the cache, see [caching behaviour](#caching-behaviour).
        ///
        /// # Raw usage
        /// Can also be used with gfx raw render & depth views if necessary. The `Format` must also
        /// be provided. [See example.](struct.GlyphBrush.html#raw-usage-1)
        #[inline]
        pub fn draw_queued<C, CV, DV>(
            &mut self,
            encoder: &mut gfx::Encoder<R, C>,
            target: &CV,
            depth_target: &DV,
        ) -> Result<(), String>
        where
            C: gfx::CommandBuffer<R>,
            CV: RawAndFormat<Raw = RawRenderTargetView<R>>,
            DV: RawAndFormat<Raw = RawDepthStencilView<R>>,
        {
            self.draw_queued_with_transform(IDENTITY_MATRIX4, encoder, target, depth_target)
        }

        /// Draws all queued sections onto a render target, applying a position transform (e.g.
        /// a projection).
        /// See [`queue`](struct.GlyphBrush.html#method.queue).
        ///
        /// Trims the cache, see [caching behaviour](#caching-behaviour).
        ///
        /// # Raw usage
        /// Can also be used with gfx raw render & depth views if necessary. The `Format` must also
        /// be provided.
        ///
        /// ```no_run
        /// # extern crate gfx;
        /// # extern crate gfx_window_glutin;
        /// # extern crate glutin;
        /// # extern crate gfx_glyph;
        /// # use gfx_glyph::{GlyphBrushBuilder};
        /// # use gfx_glyph::Section;
        /// # use gfx::format;
        /// # use gfx::format::Formatted;
        /// # use gfx::memory::Typed;
        /// # fn main() -> Result<(), String> {
        /// # let events_loop = glutin::EventsLoop::new();
        /// # let (_window, _device, mut gfx_factory, gfx_color, gfx_depth) =
        /// #     gfx_window_glutin::init::<gfx::format::Srgba8, gfx::format::Depth>(
        /// #         glutin::WindowBuilder::new(),
        /// #         glutin::ContextBuilder::new(),
        /// #         &events_loop).unwrap();
        /// # let mut gfx_encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();
        /// # let dejavu: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
        /// # let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(dejavu)
        /// #     .build(gfx_factory.clone());
        /// # let raw_render_view = gfx_color.raw();
        /// # let raw_depth_view = gfx_depth.raw();
        /// # let transform = [[0.0; 4]; 4];
        /// glyph_brush.draw_queued_with_transform(
        ///     transform,
        ///     &mut gfx_encoder,
        ///     &(raw_render_view, format::Srgba8::get_format()),
        ///     &(raw_depth_view, format::Depth::get_format()),
        /// )?
        /// # ;
        /// # Ok(())
        /// # }
        /// ```
        pub fn draw_queued_with_transform<C, CV, DV>(
            &mut self,
            transform: [[f32; 4]; 4],
            mut encoder: &mut gfx::Encoder<R, C>,
            target: &CV,
            depth_target: &DV,
        ) -> Result<(), String>
        where
            C: gfx::CommandBuffer<R>,
            CV: RawAndFormat<Raw = RawRenderTargetView<R>>,
            DV: RawAndFormat<Raw = RawDepthStencilView<R>>,
        {
            let (screen_width, screen_height, ..) = target.as_raw().get_dimensions();
            let screen_dims = (u32::from(screen_width), u32::from(screen_height));

            let mut brush_action;

            loop {
                let tex = self.font_cache_tex.0.clone();

                brush_action = self.glyph_brush.process_queued(
                    screen_dims,
                    |rect, tex_data| {
                        let offset = [rect.min.x as u16, rect.min.y as u16];
                        let size = [rect.width() as u16, rect.height() as u16];
                        update_texture(&mut encoder, &tex, offset, size, tex_data);
                    },
                    to_vertex,
                );

                match brush_action {
                    Ok(_) => break,
                    Err(BrushError::TextureTooSmall { suggested }) => {
                        let (new_width, new_height) = suggested;

                        if log_enabled!(log::Level::Warn) {
                            warn!(
                                "Increasing glyph texture size {old:?} -> {new:?}. \
                                 Consider building with `.initial_cache_size({new:?})` to avoid \
                                 resizing. Called from:\n (NO BACKTRACE)",
                                old = self.glyph_brush.texture_dimensions(),
                                new = (new_width, new_height),
                            );
                        }

                        match create_texture(&mut self.factory, new_width, new_height) {
                            Ok((new_tex, tex_view)) => {
                                self.glyph_brush.resize_texture(new_width, new_height);

                                if let Some(ref mut cache) = self.draw_cache {
                                    cache.pipe_data.font_tex.0 = tex_view.clone();
                                }

                                self.font_cache_tex.1 = tex_view;
                                self.font_cache_tex.0 = new_tex;
                            }
                            Err(_) => {
                                return Err(format!(
                                    "Failed to create {}x{} glyph texture",
                                    new_width, new_height
                                ));
                            }
                        }
                    }
                }
            }

            match brush_action.unwrap() {
                BrushAction::Draw(verts) => {
                    let vbuf = self.factory.create_vertex_buffer(&verts);

                    let draw_cache = if let Some(mut cache) = self.draw_cache.take() {
                        cache.pipe_data.vbuf = vbuf;
                        if &cache.pipe_data.out != target.as_raw() {
                            cache.pipe_data.out.clone_from(target.as_raw());
                        }
                        if &cache.pipe_data.out_depth != depth_target.as_raw() {
                            cache.pipe_data.out_depth.clone_from(depth_target.as_raw());
                        }
                        if cache.pso.0 != target.format() {
                            cache.pso = (
                                target.format(),
                                self.pso_using(target.format(), depth_target.format()),
                            );
                        }
                        cache.slice.instances.as_mut().unwrap().0 = verts.len() as _;
                        cache
                    } else {
                        DrawnGlyphBrush {
                            pipe_data: {
                                let sampler =
                                    self.factory.create_sampler(texture::SamplerInfo::new(
                                        self.texture_filter_method,
                                        texture::WrapMode::Clamp,
                                    ));
                                glyph_pipe::Data {
                                    vbuf,
                                    font_tex: (self.font_cache_tex.1.clone(), sampler),
                                    transform,
                                    out: target.as_raw().clone(),
                                    out_depth: depth_target.as_raw().clone(),
                                }
                            },
                            pso: (
                                target.format(),
                                self.pso_using(target.format(), depth_target.format()),
                            ),
                            slice: gfx::Slice {
                                instances: Some((verts.len() as _, 0)),
                                ..Self::empty_slice()
                            },
                        }
                    };

                    self.draw_cache = Some(draw_cache);
                }
                BrushAction::ReDraw => {}
            };

            if let Some(&mut DrawnGlyphBrush {
                ref pso,
                ref slice,
                ref mut pipe_data,
                ..
            }) = self.draw_cache.as_mut()
            {
                pipe_data.transform = transform;
                encoder.draw(slice, &pso.1, pipe_data);
            }

            Ok(())
        }

        /// Returns the available fonts.
        ///
        /// The `FontId` corresponds to the index of the font data.
        #[inline]
        pub fn fonts(&self) -> &[Font<'_>] {
            self.glyph_brush.fonts()
        }

        fn pso_using(
            &mut self,
            color_format: gfx::format::Format,
            depth_format: gfx::format::Format,
        ) -> gfx::PipelineState<R, glyph_pipe::Meta> {
            self.factory
                .create_pipeline_from_program(
                    &self.program,
                    gfx::Primitive::TriangleStrip,
                    gfx::state::Rasterizer::new_fill(),
                    glyph_pipe::Init::new(color_format, depth_format, self.depth_test),
                )
                .unwrap()
        }

        fn empty_slice() -> gfx::Slice<R> {
            gfx::Slice {
                start: 0,
                end: 4,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0,
                instances: None,
            }
        }

        /// Adds an additional font to the one(s) initially added on build.
        ///
        /// Returns a new [`FontId`](struct.FontId.html) to reference this font.
        ///
        /// # Example
        ///
        /// ```no_run
        /// # extern crate gfx;
        /// # extern crate gfx_window_glutin;
        /// # extern crate glutin;
        /// extern crate gfx_glyph;
        /// use gfx_glyph::{GlyphBrushBuilder, Section};
        /// # fn main() {
        /// # let events_loop = glutin::EventsLoop::new();
        /// # let (_window, _device, mut gfx_factory, gfx_color, gfx_depth) =
        /// #     gfx_window_glutin::init::<gfx::format::Srgba8, gfx::format::Depth>(
        /// #         glutin::WindowBuilder::new(),
        /// #         glutin::ContextBuilder::new(),
        /// #         &events_loop).unwrap();
        /// # let mut gfx_encoder: gfx::Encoder<_, _> = gfx_factory.create_command_buffer().into();
        ///
        /// // dejavu is built as default `FontId(0)`
        /// let dejavu: &[u8] = include_bytes!("../../fonts/DejaVuSans.ttf");
        /// let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(dejavu).build(gfx_factory.clone());
        ///
        /// // some time later, add another font referenced by a new `FontId`
        /// let open_sans_italic: &[u8] = include_bytes!("../../fonts/OpenSans-Italic.ttf");
        /// let open_sans_italic_id = glyph_brush.add_font_bytes(open_sans_italic);
        /// # glyph_brush.draw_queued(&mut gfx_encoder, &gfx_color, &gfx_depth).unwrap();
        /// # let _ = open_sans_italic_id;
        /// # }
        /// ```
        pub fn add_font_bytes<'a: 'font, B: Into<SharedBytes<'a>>>(
            &mut self,
            font_data: B,
        ) -> FontId {
            self.glyph_brush.add_font_bytes(font_data)
        }

        /// Adds an additional font to the one(s) initially added on build.
        ///
        /// Returns a new [`FontId`](struct.FontId.html) to reference this font.
        pub fn add_font<'a: 'font>(&mut self, font_data: Font<'a>) -> FontId {
            self.glyph_brush.add_font(font_data)
        }
    }

    struct DrawnGlyphBrush<R: gfx::Resources> {
        pipe_data: glyph_pipe::Data<R>,
        pso: (gfx::format::Format, gfx::PipelineState<R, glyph_pipe::Meta>),
        slice: gfx::Slice<R>,
    }

    #[inline]
    fn to_vertex(
        glyph_brush::GlyphVertex {
            mut tex_coords,
            pixel_coords,
            bounds,
            screen_dimensions: (screen_w, screen_h),
            color,
            z,
        }: glyph_brush::GlyphVertex,
    ) -> GlyphVertex {
        let gl_bounds = Rect {
            min: point(
                2.0 * (bounds.min.x / screen_w - 0.5),
                2.0 * (0.5 - bounds.min.y / screen_h),
            ),
            max: point(
                2.0 * (bounds.max.x / screen_w - 0.5),
                2.0 * (0.5 - bounds.max.y / screen_h),
            ),
        };

        let mut gl_rect = Rect {
            min: point(
                2.0 * (pixel_coords.min.x as f32 / screen_w - 0.5),
                2.0 * (0.5 - pixel_coords.min.y as f32 / screen_h),
            ),
            max: point(
                2.0 * (pixel_coords.max.x as f32 / screen_w - 0.5),
                2.0 * (0.5 - pixel_coords.max.y as f32 / screen_h),
            ),
        };

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if gl_rect.max.x > gl_bounds.max.x {
            let old_width = gl_rect.width();
            gl_rect.max.x = gl_bounds.max.x;
            tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
        }
        if gl_rect.min.x < gl_bounds.min.x {
            let old_width = gl_rect.width();
            gl_rect.min.x = gl_bounds.min.x;
            tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
        }
        // note: y access is flipped gl compared with screen,
        // texture is not flipped (ie is a headache)
        if gl_rect.max.y < gl_bounds.max.y {
            let old_height = gl_rect.height();
            gl_rect.max.y = gl_bounds.max.y;
            tex_coords.max.y =
                tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
        }
        if gl_rect.min.y > gl_bounds.min.y {
            let old_height = gl_rect.height();
            gl_rect.min.y = gl_bounds.min.y;
            tex_coords.min.y =
                tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
        }

        GlyphVertex {
            left_top: [gl_rect.min.x, gl_rect.max.y, z],
            right_bottom: [gl_rect.max.x, gl_rect.min.y],
            tex_left_top: [tex_coords.min.x, tex_coords.max.y],
            tex_right_bottom: [tex_coords.max.x, tex_coords.min.y],
            color,
        }
    }

    // Creates a gfx texture with the given data
    fn create_texture<F, R>(
        factory: &mut F,
        width: u32,
        height: u32,
    ) -> Result<(TexSurfaceHandle<R>, TexShaderView<R>), Box<dyn Error>>
    where
        R: gfx::Resources,
        F: gfx::Factory<R>,
    {
        let kind = texture::Kind::D2(
            width as texture::Size,
            height as texture::Size,
            texture::AaMode::Single,
        );

        let tex = factory.create_texture(
            kind,
            1 as texture::Level,
            gfx::memory::Bind::SHADER_RESOURCE,
            gfx::memory::Usage::Dynamic,
            Some(<TexChannel as format::ChannelTyped>::get_channel_type()),
        )?;

        let view = factory.view_texture_as_shader_resource::<TexForm>(
            &tex,
            (0, 0),
            format::Swizzle::new(),
        )?;

        Ok((tex, view))
    }

    // Updates a texture with the given data (used for updating the GlyphCache texture)
    #[inline]
    fn update_texture<R, C>(
        encoder: &mut gfx::Encoder<R, C>,
        texture: &handle::Texture<R, TexSurface>,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
    {
        let info = texture::ImageInfoCommon {
            xoffset: offset[0],
            yoffset: offset[1],
            zoffset: 0,
            width: size[0],
            height: size[1],
            depth: 0,
            format: (),
            mipmap: 0,
        };
        encoder
            .update_texture::<TexSurface, TexForm>(texture, None, info, data)
            .unwrap();
    }

}
