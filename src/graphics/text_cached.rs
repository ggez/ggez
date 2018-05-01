use super::*;

use graphics::text::Font;
pub use gfx_glyph::{FontId, Scale};
use gfx_glyph::{Section, SectionText, VariedSection};

/// A piece of text with optional color, font and scale information.
/// Can be implicitly constructed from `String` and `(String, Color)`.
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
            text,
            ..TextFragment::default()
        }
    }
}

// TODO: consider even more convenience conversions.
impl From<(String, Color)> for TextFragment {
    fn from(tuple: (String, Color)) -> TextFragment {
        TextFragment {
            text: tuple.0,
            color: Some(tuple.1),
            ..TextFragment::default()
        }
    }
}

/// Optional parameters for `TextCached.queue()`.
#[derive(Clone, Debug)]
pub struct TextParam {
    // TODO: figure out how to use font metrics to make it behave as `DrawParam::offset` does.
    /// Offset for transformations, like scale or rotation, in screen coordinates.
    /// This is different from `DrawParam::offset`!
    pub offset: Point2,
    /// Dimensions of the rectangle to try and fit (by wrapping) the text into.
    pub bounds: Point2,
    /// Text's color, defaults to white (`graphics::get_color()`).
    pub color: Option<Color>,
    /// Text's font ID, defaults to 0 (`Text::default_font()`).
    pub font_id: Option<FontId>,
    /// Text's scale, defaults to uniform 16px(?).
    pub scale: Option<Scale>,
}

impl Default for TextParam {
    fn default() -> Self {
        use std::f32;
        TextParam {
            offset: Point2::origin(),
            bounds: Point2::new(f32::INFINITY, f32::INFINITY),
            color: None,
            font_id: None,
            scale: None,
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
}

impl TextCached {
    /// Creates an empty `TextCached`.
    pub fn new_empty(context: &mut Context) -> GameResult<TextCached>
    {
        Ok(TextCached {
            fragments: Vec::new(),
            blend_mode: None,
        })
    }

    /// Creates a `TextCached` from a `TextFragment`.
    pub fn new<T>(context: &mut Context, fragment: T) -> GameResult<TextCached>
    where
        T: Into<TextFragment>,
    {
        let mut text = TextCached::new_empty(context)?;
        text.add_fragment(fragment);
        Ok(text)
    }

    /// Adds another `TextFragment`; can be chained.
    pub fn add_fragment<T>(&mut self, fragment: T) -> &mut TextCached
    where
        T: Into<TextFragment>,
    {
        self.fragments.push(fragment.into());
        self
    }

    /// Queues the `TextCached` to be drawn by `draw_queued()`.
    /// This is much more efficient than using `graphics::draw()` or equivalent.
    /// Note, any `TextCached` drawn via `graphics::draw()` will also draw the queue,
    /// if it hasn't been drawn by `draw_queued()` yet.
    pub fn queue(&self, context: &mut Context, param: TextParam) {
        let mut sections = Vec::new();
        for fragment in self.fragments.iter() {
            let color = match fragment.color {
                Some(color) => color,
                None => match param.color {
                    Some(color) => color,
                    None => get_color(context),
                },
            };
            let font_id = match fragment.font_id {
                Some(font_id) => font_id,
                None => match param.font_id {
                    Some(font_id) => font_id,
                    None => FontId::default(),
                },
            };
            let scale = match fragment.scale {
                Some(scale) => scale,
                None => match param.scale {
                    Some(scale) => scale,
                    None => Scale::uniform(16.0),
                },
            };
            sections.push(SectionText {
                text: &fragment.text,
                color: <[f32; 4]>::from(color),
                font_id,
                scale,
            });
        }
        context.gfx_context.glyph_brush.queue(VariedSection {
            screen_position: (param.offset.x, param.offset.y),
            bounds: (param.bounds.x, param.bounds.x),
            //z: f32,
            //layout: Layout<BuiltInLineBreaker>,
            text: sections,
            ..VariedSection::default()
        });
    }

    /// Draws all of queued `TextCached`; `DrawParam` apply to everything in the queue.
    /// `DrawParam::offset` is ignored - specify it when queueing instead (in screen coords).
    /// This is much more efficient than using `graphics::draw()` or equivalent.
    pub fn draw_queued(context: &mut Context, param: DrawParam) -> GameResult<()> {
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
        self.queue(
            ctx,
            TextParam {
                offset: param.offset,
                ..TextParam::default()
            },
        );
        TextCached::draw_queued(ctx, param)
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
