//! This module is an **experimental** integration of the `gfx_glyph`
//! text cache crate.  This should offer both much higher performance than
//! `ggez::graphics::Text`, and also more features.  Use it, enjoy it,
//! experiment with it, and offer suggestions for improving the API.
//! Hopefully this will be the default method of rendering text for
//! ggez 0.5.0.


/*
Hokay, we want to rename queue() to add() for symmetry with SpriteBatch
Similarly, draw_queued() should probably just become draw().

Then we need a global TextBatch (I think?) that stores immediate drawing
state, and some draw_text() functions that expose it.  Text basically stops
being its own Drawable, unless we want to fill that role with TextFragment
I suppose.  ...that might actually work perfectly.

I think FOR NOW we really should just pull bitmap fonts out entirely
and see how things look without them.  We REALLY SHOULD be able to add
things to rusttype's cache or gfx_glyph to handle them easily, since it's
doing the same damn thing.
Can always re-implement our own using SpriteBatch, which might be simplest!
*/

use super::*;

use gfx_glyph::{self, GlyphPositioner, Layout, SectionText, VariedSection};
pub use gfx_glyph::{FontId, HorizontalAlign as Align, Scale};
use mint;
use std::borrow::Cow;
use std::f32;
use std::sync::{Arc, RwLock};

// TODO: consider adding bits from example to docs.

/// Default scale, used as `Scale::uniform(DEFAULT_FONT_SCALE)` when no explicit scale is given.
pub const DEFAULT_FONT_SCALE: f32 = 16.0;

/// A piece of text with optional color, font and font scale information.
/// These options take precedence over any similar field/argument.
/// Can be implicitly constructed from `String`, `(String, Color)`, and `(String, FontId, Scale)`.
#[derive(Clone, Debug)]
pub struct TextFragment {
    /// Text string itself.
    pub text: String,
    /// Fragment's color, defaults to text's color.
    pub color: Option<Color>,
    /// Fragment's font ID, defaults to text's font ID.
    pub font_id: Option<FontId>,
    /// Fragment's scale, defaults to text's scale.
    pub scale: Option<Scale>,
}

impl Default for TextFragment {
    fn default() -> Self {
        TextFragment {
            text: "".into(),
            color: None,
            font_id: None,
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
    pub fn set_color(mut self, color: Color) -> TextFragment {
        self.color = Some(color);
        self
    }

    /// Set fragment's font, overrides text's font.
    pub fn set_font<T: Into<FontId>>(mut self, font_id: T) -> TextFragment {
        self.font_id = Some(font_id.into());
        self
    }

    /// Set fragment's scale, overrides text's scale.
    pub fn set_scale(mut self, scale: Scale) -> TextFragment {
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

impl Into<FontId> for Font {
    fn into(self) -> FontId {
        if let Font::GlyphFont(font_id) = self {
            return font_id;
        }
        FontId(0)
    }
}

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

/// Drawable text.
/// Can be either monolithic, or consist of differently-formatted fragments.
#[derive(Debug)]
pub struct TextBatch {
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
impl Clone for TextBatch {
    fn clone(&self) -> Self {
        TextBatch
     {
            fragments: self.fragments.clone(),
            blend_mode: self.blend_mode.clone(),
            bounds: self.bounds.clone(),
            layout: self.layout.clone(),
            font_id: self.font_id.clone(),
            font_scale: self.font_scale.clone(),
            cached_metrics: Arc::new(RwLock::new(CachedMetrics::default())),
        }
    }
}

impl Default for TextBatch {
    fn default() -> Self {
        TextBatch
     {
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

impl TextBatch {
    /// Creates a `TextBatch
    ///` from a `TextFragment`.
    pub fn new<F>(fragment: F) -> TextBatch

    where
        F: Into<TextFragment>,
    {
        let mut text = TextBatch
    ::new_empty();
        text.add_fragment(fragment);
        text
    }

    /// Creates an empty `TextBatch
    ///`.
    /// 
    /// TODO: Same as Default?
    pub fn new_empty() -> TextBatch
 {
        TextBatch
    ::default()
    }

    /// Appends a `TextFragment`.
    pub fn add_fragment<F>(&mut self, fragment: F) -> &mut TextBatch

    where
        F: Into<TextFragment>,
    {
        self.fragments.push(fragment.into());
        self.invalidate_cached_metrics();
        self
    }

    /// Returns a slice with all fragments, for reading.
    pub fn fragments(&self) -> &[TextFragment] {
        &self.fragments
    }

    /// Returns a mutable slice with all fragments.
    pub fn fragments_mut(&mut self) -> &mut [TextFragment] {
        &mut self.fragments
    }

    /// Specifies rectangular dimensions to try and fit contents inside of,
    /// by wrapping, and alignment within the bounds.
    pub fn set_bounds<P>(&mut self, bounds: P, alignment: Align) -> &mut TextBatch

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
    pub fn set_font<FI>(&mut self, font_id: FI, font_scale: Scale) -> &mut TextBatch

    where
        FI: Into<FontId>,
    {
        self.font_id = font_id.into();
        self.font_scale = font_scale;
        self.invalidate_cached_metrics();
        self
    }

    /// Converts `TextBatch
    ///` to a type `gfx_glyph` can understand and queue.
    fn generate_varied_section<'a>(
        &'a self,
        relative_dest: Point2,
        color: Option<Color>,
    ) -> VariedSection<'a> {
        let mut sections = Vec::new();
        for fragment in &self.fragments {
            let color = match fragment.color {
                Some(c) => c,
                None => match color {
                    Some(c) => c,
                    None => WHITE,
                },
            };
            let font_id = match fragment.font_id {
                Some(font_id) => font_id,
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

    /// Calculates, caches, and returns width and height of formatted and wrapped text.
    fn calculate_dimensions(&self, context: &Context) -> (u32, u32) {
        let mut max_width = 0;
        let mut max_height = 0;
        {
            let varied_section = self.generate_varied_section(Point2::new(0.0, 0.0), None);
            let glyphed_section_texts = self.layout
                .calculate_glyphs(context.gfx_context.glyph_brush.fonts(), &varied_section);
            for glyphed_section_text in &glyphed_section_texts {
                let &gfx_glyph::GlyphedSectionText(ref positioned_glyphs, ..) =
                    glyphed_section_text;
                for positioned_glyph in positioned_glyphs {
                    if let Some(rect) = positioned_glyph.pixel_bounding_box() {
                        if rect.max.x > max_width {
                            max_width = rect.max.x;
                        }
                        if rect.max.y > max_height {
                            max_height = rect.max.y;
                        }
                    }
                }
            }
        }
        let (width, height) = (max_width as u32, max_height as u32);
        if let Ok(mut metrics) = self.cached_metrics.write() {
            metrics.width = Some(width);
            metrics.height = Some(height);
        }
        (width, height)
    }

    // TODO: Do we want a dimensions() function as well that returns
    // both width and height?

    /// Returns the width of formatted and wrapped text, in screen coordinates.
    pub fn width(&self, context: &Context) -> u32 {
        if let Ok(metrics) = self.cached_metrics.read() {
            if let Some(width) = metrics.width {
                return width;
            }
        }
        self.calculate_dimensions(context).0
    }

    /// Returns the height of formatted and wrapped text, in screen coordinates.
    pub fn height(&self, context: &Context) -> u32 {
        if let Ok(metrics) = self.cached_metrics.read() {
            if let Some(height) = metrics.height {
                return height;
            }
        }
        self.calculate_dimensions(context).1
    }

    /// Queues the `TextBatch
    ///` to be drawn by `draw_queued()`.
    /// `relative_dest` is relative to the `DrawParam::dest` passed to `draw_queued()`.
    /// Note, any `TextBatch
    ///` drawn via `graphics::draw()` will also draw the queue.
    pub fn queue<P>(&self, context: &mut Context, relative_dest: P, color: Option<Color>)
    where
        P: Into<mint::Point2<f32>>,
    {
        let p = Point2::from(relative_dest.into());
        let varied_section = self.generate_varied_section(p, color);
        context.gfx_context.glyph_brush.queue(varied_section);
    }

    /// Exposes `gfx_glyph`'s `GlyphBrush::queue()` and `GlyphBrush::queue_custom_layout()`,
    /// in case `ggez`' API is insufficient.
    pub fn queue_raw<'a, S, G>(context: &mut Context, section: S, custom_layout: Option<&G>)
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

    /// Draws all of `queue()`d `TextBatch
    ///`.
    /// `DrawParam` apply to everything in the queue; offset is in screen coordinates;
    /// color is ignored - specify it when `queue()`ing instead.
    pub fn draw_queued<D>(context: &mut Context, param: D) -> GameResult
    where
        D: Into<PrimitiveDrawParam>,
    {
        let param: PrimitiveDrawParam = param.into();
        // TODO: Fix allllll this crap...
        /*
        type Mat4 = na::Matrix4<f32>;
        type Vec3 = na::Vector3<f32>;

        // TODO: fix non-pixel screen coordinates.
        let screen_rect = get_screen_coordinates(context);
        let (screen_w, screen_h) = (screen_rect.w, screen_rect.h);
        let (offset_x, offset_y) = (
            -1.0 + 2.0 * param.offset.x / screen_w,
            1.0 - 2.0 * param.offset.y / screen_h,
        );
        let (aspect, aspect_inv) = (screen_h / screen_w, screen_w / screen_h);
        let m_aspect = Mat4::new_nonuniform_scaling(&Vec3::new(1.0, aspect_inv, 1.0));
        let m_aspect_inv = Mat4::new_nonuniform_scaling(&Vec3::new(1.0, aspect, 1.0));
        let m_scale = Mat4::new_nonuniform_scaling(&Vec3::new(param.scale.x, param.scale.y, 1.0));
        let m_shear = Mat4::new(
            1.0,
            -param.shear.x,
            0.0,
            0.0,
            -param.shear.y,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );
        let m_rotation = Mat4::new_rotation(-param.rotation * Vec3::z());
        let m_offset = Mat4::new_translation(&Vec3::new(offset_x, offset_y, 0.0));
        let m_offset_inv = Mat4::new_translation(&Vec3::new(-offset_x, -offset_y, 0.0));
        let m_translate = Mat4::new_translation(&Vec3::new(
            2.0 * param.dest.x / screen_w,
            2.0 * -param.dest.y / screen_h,
            0.0,
        ));

        let m_transform = m_translate * m_offset * m_aspect * m_rotation * m_shear * m_scale
            * m_aspect_inv * m_offset_inv;
        */

        let (encoder, render_tgt, depth_view) = (
            &mut context.gfx_context.encoder,
            &context.gfx_context.screen_render_target,
            &context.gfx_context.depth_view,
        );

        // Typed() is an Undocumented Feature of gfx
        type ColorFormat = <GlBackendSpec as BackendSpec>::SurfaceType;
        let typed_render_target: gfx::handle::RenderTargetView<
            gfx_device_gl::Resources,
            ColorFormat,
        > = gfx::memory::Typed::new(render_tgt.clone());

        let typed_depth_target: gfx::handle::DepthStencilView<
            gfx_device_gl::Resources,
            gfx::format::DepthStencil,
        > = gfx::memory::Typed::new(depth_view.clone());
        context.gfx_context.glyph_brush.draw_queued_with_transform(
            param.matrix.into(),
            encoder,
            &typed_render_target,
            &typed_depth_target,
        )?;
        Ok(())
    }
}

impl Drawable for TextBatch {
    fn draw_primitive(&self, ctx: &mut Context, param: PrimitiveDrawParam) -> GameResult {
        // Converts fraction-of-bounding-box to screen coordinates, as required by `draw_queued()`.
        // TODO: Fix for PrimitiveDrawParam
        // let offset = Point2::new(
        //     param.offset.x * self.width(ctx) as f32,
        //     param.offset.y * self.height(ctx) as f32,
        // );
        // let param = param.offset(offset);
        self.queue(ctx, Point2::new(0.0, 0.0), Some(param.color));
        TextBatch
    ::draw_queued(ctx, param)
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
