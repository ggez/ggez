//! I guess these docs will never appear since we re-export the canvas
//! module from graphics...

use gfx::format::Swizzle;
use gfx::handle::RawRenderTargetView;
use gfx::memory::{Bind, Usage};
use gfx::texture::{AaMode, Kind};
use gfx::Factory;

use conf;
use context::DebugId;
use error::*;
use graphics::*;
use Context;

/// A generic canvas independent of graphics backend. This type should probably
/// never be used directly; use `ggez::graphics::Canvas` instead.
#[derive(Debug)]
pub struct CanvasGeneric<Spec>
where
    Spec: BackendSpec,
{
    target: RawRenderTargetView<Spec::Resources>,
    image: Image,
    debug_id: DebugId,
}

/// A canvas that can be rendered to instead of the screen (sometimes referred
/// to as "render target" or "render to texture"). Set the canvas with the
/// `ggez::graphics::set_canvas()` function, and then anything you
/// draw will be drawn to the canvas instead of the screen.
/// Resume drawing to the screen by calling `ggez::graphics::set_canvas(None)`.
///
/// **This is not an optimization tool.**  You may be tempted to say "I'll draw a scene
/// to a `Canvas` and then just draw the single `Canvas` each frame."  This is technically
/// possible but makes life much harder than it needs to be (especially since the current
/// implementation of `Canvas` has a number of [hard-to-squash bugs](https://github.com/ggez/ggez/issues?utf8=%E2%9C%93&q=is%3Aissue+canvas+).)
/// If you want to draw things maximally efficiently, use `SpriteBatch`.
///
/// A `Canvas` allows creating render targets to be used instead of
/// the screen.  This allows graphics to be rendered to images off-screen
/// in order to do things like saving to an image file or creating cool effects
/// by using shaders that render to an image.
/// If you just want to draw multiple things efficiently, look at `SpriteBatch`.
pub type Canvas = CanvasGeneric<GlBackendSpec>;

impl Canvas {
    /// Create a new canvas with the given size and number of samples.
    pub fn new(
        ctx: &mut Context,
        width: u32,
        height: u32,
        samples: conf::NumSamples,
    ) -> GameResult<Canvas> {
        let debug_id = DebugId::get(ctx);
        let (w, h) = (width as u16, height as u16);
        let aa = match samples {
            conf::NumSamples::One => AaMode::Single,
            s => AaMode::Multi(s as u8),
        };
        let kind = Kind::D2(w, h, aa);
        let levels = 1;
        let factory = &mut ctx.gfx_context.factory;
        let texture_create_info = gfx::texture::Info {
            kind: kind,
            levels: levels,
            format: ctx.gfx_context.color_format.0,
            bind: Bind::SHADER_RESOURCE | Bind::RENDER_TARGET | Bind::TRANSFER_SRC,
            usage: Usage::Data,
        };
        let tex = factory.create_texture_raw(
            texture_create_info,
            Some(ctx.gfx_context.color_format.1),
            None,
        )?;
        let resource_desc = gfx::texture::ResourceDesc {
            channel: ctx.gfx_context.color_format.1,
            layer: None,
            min: 0,
            max: levels - 1,
            swizzle: Swizzle::new(),
        };
        let resource = factory.view_texture_as_shader_resource_raw(&tex, resource_desc)?;
        let render_desc = gfx::texture::RenderDesc {
            channel: ctx.gfx_context.color_format.1,
            level: 0,
            layer: None,
        };
        let target = factory.view_texture_as_render_target_raw(&tex, render_desc)?;
        Ok(Canvas {
            target,
            image: Image {
                texture: resource,
                texture_handle: tex,
                sampler_info: ctx.gfx_context.default_sampler_info,
                blend_mode: None,
                width,
                height,
                debug_id,
            },
            debug_id,
        })
    }

    /// Create a new canvas with the current window dimensions.
    pub fn with_window_size(ctx: &mut Context) -> GameResult<Canvas> {
        use graphics;
        let (w, h) = graphics::get_drawable_size(ctx);
        // Default to no multisampling
        Canvas::new(ctx, w, h, conf::NumSamples::One)
    }

    /// Gets the backend `Image` that is being rendered to.
    pub fn get_image(&self) -> &Image {
        &self.image
    }

    /// Destroys the Canvas and returns the `Image` it contains.
    pub fn into_inner(self) -> Image {
        // This texture is created with different settings
        // than the default; does that matter?
        self.image
    }
}

impl Drawable for Canvas {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        self.debug_id.assert(ctx);
        // Gotta flip the image on the Y axis here
        // to account for OpenGL's origin being at the bottom-left.
        let mut flipped_param = param;
        flipped_param.scale.y *= -1.0;
        flipped_param.dest.y += self.image.height() as f32 * param.scale.y;
        self.image.draw_ex(ctx, flipped_param)?;
        Ok(())
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.image.blend_mode = mode;
    }
    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.image.blend_mode
    }
}

/// Set the canvas to render to. Specifying `Option::None` will cause all
/// rendering to be done directly to the screen.
pub fn set_canvas(ctx: &mut Context, target: Option<&Canvas>) {
    match target {
        Some(surface) => {
            surface.debug_id.assert(ctx);
            ctx.gfx_context.data.out = surface.target.clone();
        }
        None => {
            ctx.gfx_context.data.out = ctx.gfx_context.screen_render_target.clone();
        }
    };
}
