//! I guess these docs will never appear since we re-export the canvas
//! module from graphics...

use gfx::{Factory};
use gfx::format::{ChannelTyped, Srgb, Srgba8, Swizzle};
use gfx::handle::RenderTargetView;
use gfx::memory::{Bind, Usage};
use gfx::texture::{AaMode, Kind};

use Context;
use conf::*;
use error::*;
use graphics::*;

/// A generic canvas independent of graphics backend. This type should probably
/// never be used directly; use `ggez::graphics::Canvas` instead.
#[derive(Debug)]
pub struct CanvasGeneric<Spec>
where
    Spec: BackendSpec,
{
    target: RenderTargetView<Spec::Resources, Srgba8>,
    image: Image,
}

/// The `Canvas` type allows creating render targets to be used instead of
/// the screen.  This allows graphics to be rendered to images off-screen
/// in order to do things like saving to an image file or creating cool effects.
/// If you just want to draw multiple things efficiently, look at `SpriteBatch`.
///
/// A canvas that can be rendered to instead of the screen (sometimes referred
/// to as "render target" or "render to texture"). Set the canvas with the
/// `ggez::graphics::set_canvas()` function, and then anything you
/// draw will be drawn to the canvas instead of the screen.  
///
/// Resume drawing to the screen by calling `ggez::graphics::set_canvas(None)`.
pub type Canvas = CanvasGeneric<GlBackendSpec>;

impl Canvas {
    /// Create a new canvas with the given size and number of samples.
    pub fn new(
        ctx: &mut Context,
        width: u32,
        height: u32,
        samples: NumSamples,
    ) -> GameResult<Canvas> {
        let (w, h) = (width as u16, height as u16);
        let aa = match samples {
            NumSamples::One => AaMode::Single,
            s => AaMode::Multi(s as u8),
        };
        let kind = Kind::D2(w, h, aa);
        let cty = Srgb::get_channel_type();
        let levels = 1;
        let factory = &mut ctx.gfx_context.factory;
        let tex = factory.create_texture(
            kind,
            levels,
            Bind::SHADER_RESOURCE | Bind::RENDER_TARGET,
            Usage::Data,
            Some(cty),
        )?;
        let resource = factory.view_texture_as_shader_resource::<Srgba8>(
            &tex,
            (0, levels - 1),
            Swizzle::new(),
        )?;
        let target = factory.view_texture_as_render_target(&tex, 0, None)?;
        Ok(Canvas {
            target,
            image: Image {
                texture: resource,
                sampler_info: ctx.gfx_context.default_sampler_info,
                blend_mode: None,
                width,
                height,
            },
        })
    }

    /// Create a new canvas with the current window dimensions.
    pub fn with_window_size(ctx: &mut Context) -> GameResult<Canvas> {
        use graphics;
        let (w, h) = graphics::get_drawable_size(ctx);
        // Default to no multisampling
        Canvas::new(ctx, w, h, NumSamples::One)
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
        self.image.draw_ex(ctx, param)
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
            ctx.gfx_context.data.out = surface.target.clone();
        }
        None => {
            ctx.gfx_context.data.out = ctx.gfx_context.screen_render_target.clone();
        }
    };
}
