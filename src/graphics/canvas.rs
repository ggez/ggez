//! I guess these docs will never appear since we re-export the canvas
//! module from graphics...
use std::convert::TryFrom;

use gfx::format::{Format, Swizzle};
use gfx::handle::RawRenderTargetView;
use gfx::memory::{Bind, Usage};
use gfx::texture::{AaMode, Kind};
use gfx::Factory;

use crate::conf;
use crate::context::DebugId;
use crate::error::*;
use crate::graphics::*;
use crate::Context;

/// A generic canvas independent of graphics backend. This type should
/// never need to be used directly; use [`graphics::Canvas`](type.Canvas.html)
/// instead.
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
/// [`graphics::set_canvas()`](fn.set_canvas.html) function, and then anything you
/// draw will be drawn to the canvas instead of the screen.
///
/// Resume drawing to the screen by calling `graphics::set_canvas(None)`.
///
/// A `Canvas` allows graphics to be rendered to images off-screen
/// in order to do things like saving to an image file or creating cool effects
/// by using shaders that render to an image.
/// If you just want to draw multiple things efficiently, look at
/// [`SpriteBatch`](spritebatch/struct.Spritebatch.html).
pub type Canvas = CanvasGeneric<GlBackendSpec>;

impl<S> CanvasGeneric<S>
where
    S: BackendSpec,
{
    #[allow(clippy::new_ret_no_self)]
    /// Create a new `Canvas` with the given size and number of samples.
    pub fn new(
        ctx: &mut Context,
        width: u16,
        height: u16,
        samples: conf::NumSamples,
        color_format: Format,
    ) -> GameResult<Canvas> {
        let debug_id = DebugId::get(ctx);
        let aa = match samples {
            conf::NumSamples::One => AaMode::Single,
            s => AaMode::Multi(s.into()),
        };
        let kind = Kind::D2(width, height, aa);
        let levels = 1;
        let factory = &mut ctx.gfx_context.factory;
        let texture_create_info = gfx::texture::Info {
            kind,
            levels,
            format: color_format.0,
            bind: Bind::SHADER_RESOURCE | Bind::RENDER_TARGET | Bind::TRANSFER_SRC,
            usage: Usage::Data,
        };
        let tex = factory.create_texture_raw(texture_create_info, Some(color_format.1), None)?;
        let resource_desc = gfx::texture::ResourceDesc {
            channel: color_format.1,
            layer: None,
            min: 0,
            max: levels - 1,
            swizzle: Swizzle::new(),
        };
        let resource = factory.view_texture_as_shader_resource_raw(&tex, resource_desc)?;
        let render_desc = gfx::texture::RenderDesc {
            channel: color_format.1,
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

    /// Create a new `Canvas` with the current window dimensions.
    pub fn with_window_size(ctx: &mut Context) -> GameResult<Canvas> {
        use crate::graphics;
        let (w, h) = graphics::drawable_size(ctx);
        // Default to no multisampling
        Canvas::new(
            ctx,
            w as u16,
            h as u16,
            conf::NumSamples::One,
            get_window_color_format(ctx),
        )
    }

    /// Gets the backend `Image` that is being rendered to.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// Gets the backend `Target` that is being rendered to.
    pub fn target(&self) -> &RawRenderTargetView<S::Resources> {
        &self.target
    }

    /// Get the filter mode for the image.
    pub fn filter(&self) -> FilterMode {
        self.image.filter()
    }

    /// Set the filter mode for the canvas.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.image.set_filter(mode)
    }

    /// Destroys the `Canvas` and returns the `Image` it contains.
    pub fn into_inner(self) -> Image {
        // TODO: This texture is created with different settings
        // than the default; does that matter?
        // Test; we really just need to add Bind::TRANSFER_SRC
        // and change the Usage's to match to make them identical.
        // Ask termhn maybe?
        self.image
    }
}

impl Drawable for Canvas {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        self.debug_id.assert(ctx);
        // Gotta flip the image on the Y axis here
        // to account for OpenGL's origin being at the bottom-left.
        match param.trans {
            Transform::Values { scale, dest, .. } => {
                let new_scale = mint::Vector2 {
                    x: scale.x,
                    y: -scale.y,
                };
                let new_dest = mint::Point2 {
                    x: dest.x,
                    y: dest.y + f32::from(self.image.height()) * scale.y,
                };
                let new_param = param.dest(new_dest).scale(new_scale);
                self.image.draw(ctx, new_param)
            }
            Transform::Matrix(_) => self.image.draw(ctx, param),
        }
    }
    fn dimensions(&self, _: &mut Context) -> Option<Rect> {
        Some(self.image.dimensions())
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.image.blend_mode = mode;
    }
    fn blend_mode(&self) -> Option<BlendMode> {
        self.image.blend_mode
    }
}

/// Set the `Canvas` to render to. Specifying `Option::None` will cause all
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
