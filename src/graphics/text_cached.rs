use super::*;

pub use gfx_glyph::{FontId, Scale};
use gfx_glyph::{self, BuiltInLineBreaker, GlyphPositioner, SectionText, VariedSection};
use rusttype::{point, PositionedGlyph};
use std::borrow::Cow;

/// Aliased type from `gfx_glyph`.
pub type Layout = gfx_glyph::Layout<BuiltInLineBreaker>;

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

impl From<String> for TextFragment {
    fn from(text: String) -> TextFragment {
        TextFragment {
            text: text,
            ..Default::default()
        }
    }
}

impl From<(String, Color)> for TextFragment {
    fn from(tuple: (String, Color)) -> TextFragment {
        TextFragment {
            text: tuple.0,
            color: Some(tuple.1),
            ..Default::default()
        }
    }
}

impl From<(String, FontId, Scale)> for TextFragment {
    fn from(tuple: (String, FontId, Scale)) -> TextFragment {
        TextFragment {
            text: tuple.0,
            font_id: Some(tuple.1),
            scale: Some(tuple.2),
            ..Default::default()
        }
    }
}

impl<'a> From<&'a str> for TextFragment {
    fn from(text: &'a str) -> TextFragment {
        TextFragment {
            text: text.to_string(),
            ..Default::default()
        }
    }
}

impl<'a> From<(&'a str, Color)> for TextFragment {
    fn from(tuple: (&'a str, Color)) -> TextFragment {
        TextFragment {
            text: tuple.0.to_string(),
            color: Some(tuple.1),
            ..Default::default()
        }
    }
}

impl<'a> From<(&'a str, FontId, Scale)> for TextFragment {
    fn from(tuple: (&'a str, FontId, Scale)) -> TextFragment {
        TextFragment {
            text: tuple.0.to_string(),
            font_id: Some(tuple.1),
            scale: Some(tuple.2),
            ..Default::default()
        }
    }
}

impl From<(Point2, f32)> for DrawParam {
    fn from(tuple: (Point2, f32)) -> DrawParam {
        DrawParam {
            dest: tuple.0,
            rotation: tuple.1,
            ..Default::default()
        }
    }
}

/// Drawable text.
/// Can be either monolithic, or consist of differently-formatted fragments.
#[derive(Clone, Debug)]
pub struct TextCached {
    fragments: Vec<TextFragment>,
    // TODO: make it do something, maybe.
    blend_mode: Option<BlendMode>,
    bounds: Point2,
    layout: Layout,
    font_id: FontId,
    font_scale: Scale,
}

impl Default for TextCached {
    fn default() -> Self {
        use std::f32;
        TextCached {
            fragments: Vec::new(),
            blend_mode: None,
            bounds: Point2::new(f32::INFINITY, f32::INFINITY),
            layout: Layout::default(),
            font_id: FontId::default(),
            font_scale: Scale::uniform(DEFAULT_FONT_SCALE),
        }
    }
}

impl TextCached {
    // TODO: consider ditching context - it's here for consistency's sake, that's it.
    /// Creates a `TextCached` from a `TextFragment`.
    pub fn new<F>(context: &mut Context, fragment: F) -> GameResult<TextCached>
    where
        F: Into<TextFragment>,
    {
        let mut text = TextCached::new_empty(context)?;
        text.add_fragment(fragment);
        Ok(text)
    }

    /// Creates an empty `TextCached`.
    pub fn new_empty(context: &mut Context) -> GameResult<TextCached> {
        Ok(TextCached::default())
    }

    /// Appends a `TextFragment`.
    pub fn add_fragment<F>(&mut self, fragment: F) -> &mut TextCached
    where
        F: Into<TextFragment>,
    {
        self.fragments.push(fragment.into());
        self
    }

    /// Specifies rectangular dimensions to try and fit contents inside of, by wrapping.
    /// Alignment within bounds can be changed by passing a `Layout`; defaults to top left corner.
    pub fn set_bounds(&mut self, bounds: Point2, layout: Option<Layout>) -> &mut TextCached {
        self.bounds = bounds;
        if let Some(layout) = layout {
            self.layout = layout;
        }
        self
    }

    /// Specifies text's font and font scale; used for fragments that don't have their own.
    pub fn set_font(&mut self, font_id: FontId, font_scale: Scale) -> &mut TextCached {
        self.font_id = font_id;
        self.font_scale = font_scale;
        self
    }

    /// Returns the string that the text represents, by concatenating fragments' strings.
    pub fn contents(&self) -> String {
        self.fragments
            .iter()
            .fold("".to_string(), |acc, frg| format!("{}{}", acc, frg.text))
    }

    // TODO: doc better, make use of bounds.
    /// Calculates the width
    pub fn width(&self, context: &Context) -> u32 {
        let mut width = 0.0;
        let fonts = context.gfx_context.glyph_brush.fonts();
        for fragment in self.fragments.iter() {
            let font_id = match fragment.font_id {
                Some(font_id) => font_id,
                None => self.font_id,
            };
            let scale = match fragment.scale {
                Some(scale) => scale,
                None => self.font_scale,
            };
            let font = fonts
                .get(&font_id)
                .expect(&format!("Could not fetch {:?} from glyph brush!", font_id));
            let v_metrics = font.v_metrics(scale);
            let offset = point(0.0, v_metrics.ascent);
            let glyphs: Vec<PositionedGlyph> = font.layout(&fragment.text, scale, offset).collect();
            width += glyphs
                .iter()
                .rev()
                .filter_map(|g| {
                    g.pixel_bounding_box()
                        .map(|b| b.min.x as f32 + g.unpositioned().h_metrics().advance_width)
                })
                .next()
                .unwrap_or(0.0);
        }
        width as u32
    }

    // TODO: doc better, make use of bounds.
    /// Calculates the height
    pub fn height(&self, context: &Context) -> u32 {
        self.fragments
            .iter()
            .fold(Scale::uniform(0.0), |mut acc, frg| {
                let scale = match frg.scale {
                    Some(scale) => scale,
                    None => self.font_scale,
                };
                if scale.y.ceil() > acc.y.ceil() {
                    acc = scale
                }
                acc
            })
            .y
            .ceil() as u32
    }

    // TODO: figure out how to use font metrics to make it behave as `DrawParam::offset` does.
    /// Queues the `TextCached` to be drawn by `draw_queued()`.
    /// This is much more efficient than using `graphics::draw()` or equivalent.
    /// Note, any `TextCached` drawn via `graphics::draw()` will also draw the queue.
    pub fn queue(&self, context: &mut Context, offset: Point2, color: Option<Color>) {
        let mut sections = Vec::new();
        for fragment in self.fragments.iter() {
            let color = match fragment.color {
                Some(c) => c,
                None => match color {
                    Some(c) => c,
                    None => get_color(context),
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
        context.gfx_context.glyph_brush.queue(VariedSection {
            screen_position: (offset.x, offset.y),
            bounds: (self.bounds.x, self.bounds.x),
            //z: f32,
            layout: self.layout,
            text: sections,
            ..Default::default()
        });
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

    /// Draws all of queued `TextCached`; `DrawParam` apply to everything in the queue.
    /// Offset and color are ignored - specify them when queueing instead.
    /// This is much more efficient than using `graphics::draw()` or equivalent.
    pub fn draw_queued<D>(context: &mut Context, param: D) -> GameResult<()>
    where
        D: Into<DrawParam>,
    {
        let param: DrawParam = param.into();
        type Mat4 = na::Matrix4<f32>;
        type Vec3 = na::Vector3<f32>;

        let (offset_x, offset_y) = (-1.0, 1.0);
        let (screen_w, screen_h) = (
            context.gfx_context.screen_rect.w,
            context.gfx_context.screen_rect.h,
        );
        let (aspect, aspect_inv) = (screen_h / screen_w, screen_w / screen_h);
        let m_aspect = Mat4::new_nonuniform_scaling(&Vec3::new(1.0, aspect_inv, 1.0));
        let m_aspect_inv = Mat4::new_nonuniform_scaling(&Vec3::new(1.0, aspect, 1.0));
        let m_scale = Mat4::new_nonuniform_scaling(&Vec3::new(param.scale.x, param.scale.y, 1.0));
        let m_shear = Mat4::new(
            1.0,
            param.shear.x,
            0.0,
            0.0,
            param.shear.y,
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
        let m_rotation = Mat4::new_rotation(param.rotation * Vec3::z());
        let m_offset = Mat4::new_translation(&Vec3::new(offset_x, offset_y, 0.0));
        let m_offset_inv = Mat4::new_translation(&Vec3::new(-offset_x, -offset_y, 0.0));
        let m_translate = Mat4::new_translation(&Vec3::new(
            param.dest.x / screen_w,
            -param.dest.y / screen_h,
            0.0,
        ));

        let m_transform = m_translate * m_offset * m_aspect * m_rotation * m_scale * m_shear
            * m_aspect_inv * m_offset_inv;

        let (encoder, render_tgt, depth_view) = (
            &mut context.gfx_context.encoder,
            &context.gfx_context.screen_render_target,
            &context.gfx_context.depth_view,
        );

        Ok(context.gfx_context.glyph_brush.draw_queued_with_transform(
            m_transform.into(),
            encoder,
            render_tgt,
            depth_view,
        )?)
    }
}

impl Drawable for TextCached {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        self.queue(ctx, param.offset, param.color);
        TextCached::draw_queued(ctx, param)
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
