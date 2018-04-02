//! A `GlyphCache` is a quickly and dynamically text rendering way.

use std::f32;

use gfx_device_gl;
use gfx_glyph::{GlyphBrush, GlyphBrushBuilder, Scale, Section};

use GameResult;
use context::Context;
use graphics::{self, Color, Point2};

/// Re-exports `gfx_glyph` some enum for `TextParam.layout`.
#[doc(no_inline)]
pub use gfx_glyph::{BuiltInLineBreaker, HorizontalAlign, Layout, VerticalAlign};

/// A `GlyphCache` draw texts from gpu glyph cache.
///
/// This is faster than `graphics::Text`, and ideal for dynamic text change.
/// e.g. "typewriter like text message", "update score every frame"
#[derive(Debug)]
pub struct GlyphCache<'a> {
    glyph_brush: GlyphBrush<'a, gfx_device_gl::Resources, gfx_device_gl::Factory>,
}

impl<'a> GlyphCache<'a> {
    //
    // TODO: Build a new GlyphCache from Font struct.
    // e.g. `GlyphCache::new()`
    //
    // but graphics::Font struct is optimized for graphics::Text,
    // not suitable for this GlyphCache implement currently.
    //

    /// Build a new GlyphCache from font bytes data.
    pub fn from_bytes(ctx: &mut Context, font_bytes: &'a [u8]) -> Self {
        let builder = GlyphBrushBuilder::using_font_bytes(font_bytes);
        let factory = graphics::get_factory(ctx);

        let glyph_brush = builder.build(factory.clone());

        GlyphCache { glyph_brush }
    }

    /// Set text and display param.
    pub fn queue(&mut self, param: TextParam) {
        self.glyph_brush.queue(Section {
            text: param.text,
            screen_position: (param.position.x, param.position.y),
            bounds: (param.bounds.x, param.bounds.y),
            scale: Scale::uniform(param.font_size),
            color: [param.color.r, param.color.g, param.color.b, param.color.a],
            z: param.z,
            layout: param.layout,
            ..Section::default()
        })
    }

    /// Draw to canvas. Almost same as `graphics::Draw()`.
    ///
    /// `graphics::Draw()` and `graphics::DrawParam` struct is not compatible
    /// for `gfx_glyph::Section`.
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;

        self.glyph_brush
            .draw_queued(&mut gfx.encoder, &gfx.screen_render_target, &gfx.depth_view)?;

        Ok(())
    }

    /// Extract GlyphCache inner `gfx_glyph::GlyphBrush`.
    ///
    /// Currently, ggez not have `gfx_glyph` had full implement,
    /// probably some user will decide way to the DIY.
    ///
    /// At that time this function will be useful.
    pub fn extract_glyph_brush(
        self,
    ) -> GlyphBrush<'a, gfx_device_gl::Resources, gfx_device_gl::Factory> {
        self.glyph_brush
    }
}

/// This struct feature like a `graphics::DrawParam`,
/// but `graphics::DrawParam` is not compatible for `gfx_glyph::Section` struct.
///
/// Some doc comment quote from `gfx_glyph::Section` comment.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextParam<'a> {
    /// GlyphCache render from this text.
    pub text: &'a str,
    /// Position on screen to render text, in pixels from top-left.
    /// Defaults to Point2(0, 0).
    pub position: Point2,
    /// Max (width, height) bounds, in pixels from top-left.
    /// Defaults to unbounded.
    pub bounds: Point2,
    /// Font size. Defaults to 16.0.
    pub font_size: f32,
    /// Rgba color of rendered text. Defaults to black.
    pub color: Color,
    /// Z values for use in depth testing. Defaults to 0.0.
    ///
    /// **NOTICE: Currently no worked!**
    pub z: f32,
    /// Display layout.
    /// Defaults to "Left-align and No-wrap"
    pub layout: Layout<BuiltInLineBreaker>,
}

impl Default for TextParam<'static> {
    fn default() -> Self {
        Self {
            text: "",
            position: Point2::new(0.0, 0.0),
            bounds: Point2::new(f32::INFINITY, f32::INFINITY),
            font_size: 16.0,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            z: 0.0,
            layout: Layout::default(),
        }
    }
}
