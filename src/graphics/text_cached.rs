use super::*;

use graphics::text::Font;
use gfx_glyph::{FontId, Scale, Section};

/// Efficient drawable text using a `Font::GlyphFont`.
#[derive(Clone, Debug)]
pub struct TextCached {
    font_id: FontId,
    font_scale: Scale,
    contents: String,
    blend_mode: Option<BlendMode>,
}

impl TextCached {
    /// Creates a `TextCached` with given `Font::GlyphFont`.
    /// Can be relatively efficiently re-created every frame.
    pub fn new(context: &mut Context, text: &str, font: &Font) -> GameResult<TextCached> {
        if let &Font::GlyphFont { font_id, scale } = font {
            return Ok(TextCached {
                font_id,
                font_scale: scale,
                contents: text.to_string(),
                blend_mode: None,
            });
        }
        Err(GameError::FontError(
            "`TextCached` can only be used with a `Font::GlyphFont`!".into(),
        ))
    }
}

impl Drawable for TextCached {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let color = match param.color {
            Some(color) => color,
            None => get_color(ctx),
        };
        let (font_id, font_scale) = (self.font_id, self.font_scale);
        ctx.gfx_context.glyph_brush.queue(Section {
            text: &self.contents,
            //screen_position: (dest.x, dest.y),
            //bounds: (f32, f32),
            scale: font_scale,
            color: <[f32; 4]>::from(color),
            //z: f32,
            //layout: Layout<BuiltInLineBreaker>,
            font_id,
            ..Section::default()
        });

        type Mat4 = na::Matrix4<f32>;
        type Vec3 = na::Vector3<f32>;

        let (offset_x, offset_y) = (-1.0, 1.0);
        let (screen_w, screen_h) = (ctx.gfx_context.screen_rect.w, ctx.gfx_context.screen_rect.h);
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
            &mut ctx.gfx_context.encoder,
            &ctx.gfx_context.screen_render_target,
            &ctx.gfx_context.depth_view,
        );

        ctx.gfx_context.glyph_brush.draw_queued_with_transform(
            m_transform.into(),
            encoder,
            render_tgt,
            depth_view,
        )?;
        Ok(())
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
