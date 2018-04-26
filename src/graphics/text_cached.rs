use super::*;

use graphics::text::Font;
use gfx_glyph::{GlyphBrush, GlyphBrushBuilder, Section};
pub use gfx_glyph::FontId;
use rusttype::Font as TTFFont;

/// TODO: doc me
#[derive(Clone, Debug)]
pub struct TextCached {
    font_id: FontId,
    contents: String,
    blend_mode: Option<BlendMode>,
}

/// TODO: doc me
fn unpack_font_enum(font: &Font) -> GameResult<&TTFFont<'static>> {
    match *font {
        Font::TTFFont { ref font, .. } => Ok(font),
        Font::BitmapFontVariant(_) => {
            return Err(GameError::FontError(
                "Only TTF fonts can be used with TextCached!".into(),
            ))
        }
    }
}

impl TextCached {
    /// TODO: doc me
    pub fn load_fonts(context: &mut Context, fonts: &[Font]) -> GameResult<()> {
        let mut fonts = fonts.iter();
        let first = unpack_font_enum(fonts.next().unwrap())?;
        let mut brush_builder = GlyphBrushBuilder::using_font(first.clone());
        for font in fonts {
            let font = unpack_font_enum(font)?;
            brush_builder.add_font(font.clone());
        }
        let factory = *context.gfx_context.factory.clone();
        context.gfx_context.glyph_brush = brush_builder.build(factory);
        Ok(())
    }

    /// TODO: doc me
    pub fn new(context: &mut Context, text: &str, font_id: FontId) -> GameResult<TextCached> {
        Ok(TextCached {
            font_id: font_id.clone(),
            contents: text.to_string(),
            blend_mode: None,
        })
    }
}

impl Drawable for TextCached {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let coords = param.dest.coords;
        ctx.gfx_context.glyph_brush.queue(Section {
            text: &self.contents,
            screen_position: (coords[0], coords[1]),
            //bounds: (f32, f32),
            //scale: Scale,
            //color: [f32; 4],
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
