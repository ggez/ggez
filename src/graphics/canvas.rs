//! The `canvas` module enables creating render targets to be used instead of
//! the screen allowing graphics to be rendered off-screen in order to do things
//! like saving to an image file or creating cool effects

use gfx::*;
use gfx::format::*;
use gfx::handle::*;

use Context;
use error::*;
use graphics::*;

/// A generic canvas independant of graphics backend. This type should probably
/// never be used; use `ggez::graphics::Canvas` instead.
#[derive(Debug)]
pub struct CanvasGeneric<Spec>
where
    Spec: BackendSpec,
{
    target: RenderTargetView<Spec::Resources, Srgba8>,
    image: Image,
}

/// A canvas that can be rendered to instead of the screen (sometimes referred
/// to as "render target" or "render to texture"). Set the cavas with the
/// `ggez::graphics::set_canvas()` function.
pub type Canvas = CanvasGeneric<GlBackendSpec>;

impl Canvas {
    /// Create a new canvas with the given size.
    pub fn new(ctx: &mut Context, width: u32, height: u32) -> GameResult<Canvas> {
        let (w, h) = (width as u16, height as u16);
        let (_, texture, target) = ctx.gfx_context.factory.create_render_target(w, h)?;
        Ok(Canvas {
            target,
            image: Image {
                texture,
                sampler_info: ctx.gfx_context.default_sampler_info,
                width,
                height,
            },
        })
    }

    /// Create a new canvas with the current window dimentions.
    pub fn with_window_size(ctx: &mut Context) -> GameResult<Canvas> {
        let (w, h) = (ctx.conf.window_width, ctx.conf.window_height);
        Canvas::new(ctx, w, h)
    }

    /// Gets the backend Image that is being rendered to.
    pub fn get_image(&self) -> &Image {
        &self.image
    }
}

impl Drawable for Canvas {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        // We need to make sure we correct for the different coordinate systems
        let mut param = param;
        if ctx.gfx_context.screen_rect.h < 0.0 {
            param.scale.y = -param.scale.y;
        }
        self.image.draw_ex(ctx, param)
    }
}

/// Set the canvas to render to. Specifying `Option::None` will cause all
/// rendering to be done directly to the screen.
pub fn set_canvas(ctx: &mut Context, target: Option<&Canvas>) {
    let out = match target {
        Some(ref surface) => &surface.target,
        None => &ctx.gfx_context.color_view,
    };
    ctx.gfx_context.data.out = out.clone();
}
