use glyph_brush::GlyphPositioner;
use glyph_brush::{self, FontId, Layout, Section, Text as GbText};
pub use glyph_brush::{ab_glyph::PxScale, GlyphBrush, HorizontalAlign as Align};
use std::borrow::Cow;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::f32;
use std::fmt;
use std::io::Read;
use std::path;
use std::rc::Rc;

use super::*;

/// A handle referring to a loaded Truetype font.
///
/// This is just an integer referring to a loaded font stored in the
/// `Context`, so is cheap to copy.  Note that fonts are cached and
/// currently never *removed* from the cache, since that would
/// invalidate the whole cache and require re-loading all the other
/// fonts.  So, you do not want to load a font more than once.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Font {
    font_id: FontId,
    // Add DebugId?  It makes Font::default() less convenient.
}

/// The font cache of the engine.
///
/// This type can be useful to measure text efficiently without being tied to
/// the `Context` lifetime.
#[derive(Clone, Debug)]
pub struct FontCache {
    glyph_brush: Rc<RefCell<GlyphBrush<DrawParam>>>,
}

impl FontCache {
    /// Returns the width and height of the formatted and wrapped text.
    pub fn dimensions(&self, text: &Text) -> Rect {
        text.calculate_dimensions(&mut self.glyph_brush.borrow_mut())
    }
}

/// A piece of text with optional color, font and font scale information.
/// Drawing text generally involves one or more of these.
/// These options take precedence over any similar field/argument.
/// Implements `From` for `char`, `&str`, `String` and
/// `(String, Font, PxScale)`.
#[derive(Clone, Debug)]
pub struct TextFragment {
    /// Text string itself.
    pub text: String,
    /// Fragment's color, defaults to text's color.
    pub color: Option<Color>,
    /// Fragment's font, defaults to text's font.
    pub font: Option<Font>,
    /// Fragment's scale, defaults to text's scale.
    pub scale: Option<PxScale>,
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
    pub fn color<C: Into<Color>>(mut self, color: C) -> TextFragment {
        self.color = Some(color.into());
        self
    }

    /// Set fragment's font, overrides text's font.
    pub fn font(mut self, font: Font) -> TextFragment {
        self.font = Some(font);
        self
    }

    /// Set fragment's scale, overrides text's scale. Default is 16.0
    pub fn scale<S: Into<PxScale>>(mut self, scale: S) -> TextFragment {
        self.scale = Some(scale.into());
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

impl<T> From<(T, Font, f32)> for TextFragment
where
    T: Into<TextFragment>,
{
    fn from((text, font, scale): (T, Font, f32)) -> TextFragment {
        text.into().font(font).scale(PxScale::from(scale))
    }
}

/// Cached font metrics that we can keep attached to a `Text`
/// so we don't have to keep recalculating them.
#[derive(Clone, Debug)]
struct CachedMetrics {
    string: Option<String>,
    width: Option<f32>,
    height: Option<f32>,
    glyph_positions: Vec<mint::Point2<f32>>,
}

impl Default for CachedMetrics {
    fn default() -> CachedMetrics {
        CachedMetrics {
            string: None,
            width: None,
            height: None,
            glyph_positions: Vec::new(),
        }
    }
}

/// Drawable text object.  Essentially a list of [`TextFragment`](struct.TextFragment.html)'s
/// and some cached size information.
///
/// It implements [`Drawable`](trait.Drawable.html) so it can be drawn immediately with
/// [`graphics::draw()`](fn.draw.html), or many of them can be queued with [`graphics::queue_text()`](fn.queue_text.html)
/// and then all drawn at once with [`graphics::draw_queued_text()`](fn.draw_queued_text.html).
#[derive(Debug, Clone)]
pub struct Text {
    fragments: Vec<TextFragment>,
    blend_mode: Option<BlendMode>,
    filter_mode: FilterMode,
    bounds: Point2,
    layout: Layout<glyph_brush::BuiltInLineBreaker>,
    font_id: FontId,
    font_scale: PxScale,
    cached_metrics: RefCell<CachedMetrics>,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            fragments: Vec::new(),
            blend_mode: None,
            filter_mode: FilterMode::Linear,
            bounds: Point2::new(f32::INFINITY, f32::INFINITY),
            layout: Layout::default(),
            font_id: FontId::default(),
            font_scale: PxScale::from(Font::DEFAULT_FONT_SCALE),
            cached_metrics: RefCell::new(CachedMetrics::default()),
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
        self.invalidate_cached_metrics();
        &mut self.fragments
    }

    /// Specifies rectangular dimensions to try and fit contents inside of,
    /// by wrapping, and alignment within the bounds.  To disable wrapping,
    /// give it a layout with `f32::INF` for the x value.
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
    pub fn set_font(&mut self, font: Font, font_scale: PxScale) -> &mut Text {
        self.font_id = font.font_id;
        self.font_scale = font_scale;
        self.invalidate_cached_metrics();
        self
    }

    /// Converts `Text` to a type `glyph_brush` can understand and queue.
    fn generate_varied_section(&self, relative_dest: Point2, color: Option<Color>) -> Section {
        let sections: Vec<GbText> = self
            .fragments
            .iter()
            .map(|fragment| {
                let color = fragment.color.or(color).unwrap_or(Color::WHITE);
                let font_id = fragment
                    .font
                    .map(|font| font.font_id)
                    .unwrap_or(self.font_id);
                let scale = fragment.scale.unwrap_or(self.font_scale);
                GbText::default()
                    .with_text(&fragment.text)
                    .with_font_id(font_id)
                    .with_scale(scale)
                    .with_color(<[f32; 4]>::from(color))
            })
            .collect();

        let relative_dest_x = {
            // This positions text within bounds with relative_dest being to the left, always.
            let mut dest_x = relative_dest.x;
            if self.bounds.x != f32::INFINITY {
                use glyph_brush::Layout::Wrap;
                match self.layout {
                    Wrap {
                        h_align: Align::Center,
                        ..
                    } => dest_x += self.bounds.x * 0.5,
                    Wrap {
                        h_align: Align::Right,
                        ..
                    } => dest_x += self.bounds.x,
                    _ => (),
                }
            }
            dest_x
        };
        let relative_dest = (relative_dest_x, relative_dest.y);
        Section {
            screen_position: relative_dest,
            bounds: (self.bounds.x, self.bounds.y),
            layout: self.layout,
            text: sections,
        }
    }

    fn invalidate_cached_metrics(&mut self) {
        if let Ok(mut metrics) = self.cached_metrics.try_borrow_mut() {
            *metrics = CachedMetrics::default();
            // Returning early avoids a double-borrow in the "else"
            // part.
            return;
        }
        warn!("Cached metrics RefCell has been poisoned.");
        self.cached_metrics = RefCell::new(CachedMetrics::default());
    }

    /// Returns the string that the text represents.
    pub fn contents(&self) -> String {
        if let Ok(metrics) = self.cached_metrics.try_borrow() {
            if let Some(ref string) = metrics.string {
                return string.clone();
            }
        }
        let string_accm: String = self
            .fragments
            .iter()
            .map(|frag| frag.text.as_str())
            .collect();

        if let Ok(mut metrics) = self.cached_metrics.try_borrow_mut() {
            metrics.string = Some(string_accm.clone());
        }
        string_accm
    }

    /// Calculates, caches, and returns position of the glyphs
    fn calculate_glyph_positions(
        &self,
        gb: &mut GlyphBrush<DrawParam>,
    ) -> std::cell::Ref<Vec<mint::Point2<f32>>> {
        if let Ok(metrics) = self.cached_metrics.try_borrow() {
            if !metrics.glyph_positions.is_empty() {
                return std::cell::Ref::map(metrics, |metrics| &metrics.glyph_positions);
            }
        }
        let glyph_positions: Vec<mint::Point2<f32>> = {
            let varied_section = self.generate_varied_section(Point2::new(0.0, 0.0), None);
            use glyph_brush::GlyphCruncher;
            gb.glyphs(varied_section)
                .map(|glyph| glyph.glyph.position)
                .map(|pos| mint::Point2 { x: pos.x, y: pos.y })
                .collect()
        };
        if let Ok(mut metrics) = self.cached_metrics.try_borrow_mut() {
            metrics.glyph_positions = glyph_positions;
        } else {
            panic!();
        }
        if let Ok(metrics) = self.cached_metrics.try_borrow() {
            std::cell::Ref::map(metrics, |metrics| &metrics.glyph_positions)
        } else {
            panic!()
        }
    }

    /// Returns a Vec containing the coordinates of the formatted and wrapped text.
    pub fn glyph_positions(&self, context: &Context) -> std::cell::Ref<Vec<mint::Point2<f32>>> {
        self.calculate_glyph_positions(&mut context.gfx_context.glyph_brush.borrow_mut())
    }

    /// Calculates, caches, and returns width and height of formatted and wrapped text.
    fn calculate_dimensions(&self, gb: &mut GlyphBrush<DrawParam>) -> Rect {
        if let Ok(metrics) = self.cached_metrics.try_borrow() {
            if let (Some(width), Some(height)) = (metrics.width, metrics.height) {
                return Rect {
                    x: 0.0,
                    y: 0.0,
                    w: width,
                    h: height,
                };
            }
        }
        let mut max_width = 0.0;
        let mut max_height = 0.0;
        {
            let varied_section = self.generate_varied_section(Point2::new(0.0, 0.0), None);
            use glyph_brush::GlyphCruncher;
            if let Some(bounds) = gb.glyph_bounds(varied_section) {
                max_width = bounds.width().ceil();
                max_height = bounds.height().ceil();
            }
        }
        if let Ok(mut metrics) = self.cached_metrics.try_borrow_mut() {
            metrics.width = Some(max_width);
            metrics.height = Some(max_height);
        }
        Rect {
            x: 0.0,
            y: 0.0,
            w: max_width,
            h: max_height,
        }
    }

    /// Returns a Rect containing the width and height of the formatted and wrapped text.
    pub fn dimensions(&self, context: &Context) -> Rect {
        self.calculate_dimensions(&mut context.gfx_context.glyph_brush.borrow_mut())
    }

    /// Returns the width of formatted and wrapped text, in screen coordinates.
    pub fn width(&self, context: &Context) -> f32 {
        self.dimensions(context).w
    }

    /// Returns the height of formatted and wrapped text, in screen coordinates.
    pub fn height(&self, context: &Context) -> f32 {
        self.dimensions(context).h
    }
}

impl Drawable for Text {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        // Converts fraction-of-bounding-box to screen coordinates, as required by `draw_queued()`.
        queue_text(ctx, self, Point2::new(0.0, 0.0), Some(param.color));
        draw_queued_text(ctx, param, self.blend_mode, self.filter_mode)
    }

    fn dimensions(&self, ctx: &mut Context) -> Option<Rect> {
        Some(self.dimensions(ctx))
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}

impl Font {
    /// Default size for fonts.
    pub const DEFAULT_FONT_SCALE: f32 = 16.0;

    /// Load a new TTF font from the given file.
    pub fn new<P>(context: &mut Context, path: P) -> GameResult<Font>
    where
        P: AsRef<path::Path> + fmt::Debug,
    {
        use crate::filesystem;
        let mut stream = filesystem::open(context, path.as_ref())?;
        let mut buf = Vec::new();
        let _ = stream.read_to_end(&mut buf)?;

        Font::new_glyph_font_bytes(context, &buf)
    }

    /// Loads a new TrueType font from given bytes and into a `gfx::GlyphBrush` owned
    /// by the `Context`.
    pub fn new_glyph_font_bytes(context: &mut Context, bytes: &[u8]) -> GameResult<Self> {
        // Take a Cow here to avoid this clone where unnecessary?
        // Nah, let's not complicate things more than necessary.
        let font = glyph_brush::ab_glyph::FontArc::try_from_vec(bytes.to_vec()).unwrap();
        let font_id = context.gfx_context.glyph_brush.borrow_mut().add_font(font);

        Ok(Font { font_id })
    }

    /// Returns the baked-in bytes of default font (currently `LiberationSans-Regular.ttf`).
    pub(crate) fn default_font_bytes() -> &'static [u8] {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/LiberationMono-Regular.ttf"
        ))
    }
}

impl Default for Font {
    fn default() -> Self {
        Font { font_id: FontId(0) }
    }
}

/// Obtains the font cache.
pub fn font_cache(context: &Context) -> FontCache {
    FontCache {
        glyph_brush: context.gfx_context.glyph_brush.clone(),
    }
}

/// Queues the `Text` to be drawn by [`draw_queued_text()`](fn.draw_queued_text.html).
/// `relative_dest` is relative to the [`DrawParam::dest`](struct.DrawParam.html#structfield.dest)
/// passed to `draw_queued()`. Note, any `Text` drawn via [`graphics::draw()`](fn.draw.html)
/// will also draw everything already the queue.
pub fn queue_text<P>(context: &mut Context, batch: &Text, relative_dest: P, color: Option<Color>)
where
    P: Into<mint::Point2<f32>>,
{
    let p = Point2::from(relative_dest.into());
    let varied_section = batch.generate_varied_section(p, color);
    context
        .gfx_context
        .glyph_brush
        .borrow_mut()
        .queue(varied_section);
}

/// Exposes `glyph_brush`'s drawing API in case `ggez`'s text drawing is insufficient.
/// It takes `glyph_brush`'s `VariedSection` and `GlyphPositioner`, which give you lower-
/// level control over how text is drawn.
pub fn queue_text_raw<'a, S, G>(context: &mut Context, section: S, custom_layout: Option<&G>)
where
    S: Into<Cow<'a, Section<'a>>>,
    G: GlyphPositioner,
{
    let brush = &mut context.gfx_context.glyph_brush.borrow_mut();
    match custom_layout {
        Some(layout) => brush.queue_custom_layout(section, layout),
        None => brush.queue(section),
    }
}

/// Draws all of the [`Text`](struct.Text.html)s added via [`queue_text()`](fn.queue_text.html).
///
/// the `DrawParam` applies to everything in the queue; offset is in
/// screen coordinates; color is ignored - specify it when using
/// `queue_text()` instead.
///
/// Note that all text will, and in fact must, be drawn with the same
/// `BlendMode` and `FilterMode`.  This is unfortunate but currently
/// unavoidable, see [this issue](https://github.com/ggez/ggez/issues/561)
/// for more info.
pub fn draw_queued_text<D>(
    ctx: &mut Context,
    param: D,
    blend: Option<BlendMode>,
    filter: FilterMode,
) -> GameResult
where
    D: Into<DrawParam>,
{
    let param: DrawParam = param.into();

    let gb = &mut ctx.gfx_context.glyph_brush;
    let encoder = &mut ctx.gfx_context.encoder;
    let gc = &ctx.gfx_context.glyph_cache.texture_handle;
    let backend = &ctx.gfx_context.backend_spec;

    let action = gb.borrow_mut().process_queued(
        |rect, tex_data| update_texture::<GlBackendSpec>(backend, encoder, gc, rect, tex_data),
        to_vertex,
    );
    match action {
        Ok(glyph_brush::BrushAction::ReDraw) => {
            let spritebatch = ctx.gfx_context.glyph_state.clone();
            let spritebatch = &mut *spritebatch.borrow_mut();
            spritebatch.set_blend_mode(blend);
            spritebatch.set_filter(filter);
            draw(ctx, &*spritebatch, param)?;
        }
        Ok(glyph_brush::BrushAction::Draw(drawparams)) => {
            // Gotta clone the image to avoid double-borrow's.
            let spritebatch = ctx.gfx_context.glyph_state.clone();
            let spritebatch = &mut *spritebatch.borrow_mut();
            spritebatch.clear();
            spritebatch.set_blend_mode(blend);
            spritebatch.set_filter(filter);
            for p in &drawparams {
                // Ignore returned sprite index.
                let _ = spritebatch.add(*p);
            }
            draw(ctx, &*spritebatch, param)?;
        }
        Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
            let (new_width, new_height) = suggested;
            let data = vec![255; 4 * new_width as usize * new_height as usize];
            let new_glyph_cache = Image::from_rgba8(
                ctx,
                u16::try_from(new_width).unwrap(),
                u16::try_from(new_height).unwrap(),
                &data,
            )?;
            ctx.gfx_context.glyph_cache = new_glyph_cache.clone();
            let spritebatch = ctx.gfx_context.glyph_state.clone();
            let spritebatch = &mut *spritebatch.borrow_mut();
            let _ = spritebatch.set_image(new_glyph_cache);
            ctx.gfx_context
                .glyph_brush
                .borrow_mut()
                .resize_texture(new_width, new_height);
        }
    }
    Ok(())
}

fn update_texture<B>(
    backend: &B,
    encoder: &mut gfx::Encoder<B::Resources, B::CommandBuffer>,
    texture: &gfx::handle::RawTexture<B::Resources>,
    rect: glyph_brush::Rectangle<u32>,
    tex_data: &[u8],
) where
    B: BackendSpec,
{
    let offset = [
        u16::try_from(rect.min[0]).unwrap(),
        u16::try_from(rect.min[1]).unwrap(),
    ];
    let size = [
        u16::try_from(rect.width()).unwrap(),
        u16::try_from(rect.height()).unwrap(),
    ];
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

    let tex_data_chunks: Vec<[u8; 4]> = tex_data.iter().map(|c| [255, 255, 255, *c]).collect();
    let typed_tex = backend.raw_to_typed_texture(texture.clone());
    encoder
        .update_texture::<<super::BuggoSurfaceFormat as gfx::format::Formatted>::Surface, super::BuggoSurfaceFormat>(
            &typed_tex, None, info, &tex_data_chunks,
        )
        .unwrap();
}

/// I THINK what we're going to need to do is have a
/// `SpriteBatch` that actually does the stuff and stores the
/// UV's and verts and such, while
///
/// Basically, `glyph_brush`'s "`to_vertex`" callback is really
/// `to_quad`; in the default code it
fn to_vertex(v: glyph_brush::GlyphVertex) -> DrawParam {
    let src_rect = Rect {
        x: v.tex_coords.min.x,
        y: v.tex_coords.min.y,
        w: v.tex_coords.max.x - v.tex_coords.min.x,
        h: v.tex_coords.max.y - v.tex_coords.min.y,
    };
    // it LOOKS like pixel_coords are the output coordinates?
    // I'm not sure though...
    let dest_pt = Point2::new(v.pixel_coords.min.x, v.pixel_coords.min.y);
    DrawParam::default()
        .src(src_rect)
        .dest(dest_pt)
        .color(v.extra.color.into())
}
