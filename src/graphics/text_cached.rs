use super::*;

use graphics::text::Font;
use gfx_glyph::{FontId, Scale, Section};
use rusttype::Font as TTFFont;

/// TODO: doc me
#[derive(Clone, Debug)]
pub struct TextCached {
    font_id: FontId,
    contents: String,
    blend_mode: Option<BlendMode>,
}

impl TextCached {
    /// TODO: doc me
    pub fn new(context: &mut Context, text: &str, font: Font) -> GameResult<TextCached> {
        if let Font::GlyphFont { font_id } = font {
            return Ok(TextCached {
                font_id,
                contents: text.to_string(),
                blend_mode: None,
            })
        }
        Err(GameError::FontError(
            "`TextCached` can only be used with a `Font::GlyphFont`!".into(),
        ))
    }
}

impl Drawable for TextCached {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let (coords, color) = (
            param.dest.coords,
            match param.color{
                Some(color) => color,
                None => get_color(ctx),
            },
        );
        ctx.gfx_context.glyph_brush.queue(Section {
            text: &self.contents,
            screen_position: (coords[0], coords[1]),
            //bounds: (f32, f32),
            //scale: Scale,
            color: <[f32; 4]>::from(color),
            //z: f32,
            //layout: Layout<BuiltInLineBreaker>,
            font_id: self.font_id,
            ..Section::default()
        });
        let (encoder, render_tgt, depth_view) = (
            &mut ctx.gfx_context.encoder,
            &ctx.gfx_context.screen_render_target,
            &ctx.gfx_context.depth_view,
        );
        ctx.gfx_context
            .glyph_brush
            .draw_queued(encoder, render_tgt, depth_view)?;
        Ok(())
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
