//! I guess these docs will never appear since we re-export the canvas
//! module from graphics...
use std::path;

use gfx::format::{Format, Swizzle};
use gfx::handle::RawRenderTargetView;
use gfx::memory::{Bind, Usage};
use gfx::texture::{AaMode, Kind};
use gfx::Factory;
use glam::Quat;

use crate::context::DebugId;
use crate::error::*;
use crate::graphics::*;
use crate::Context;
use crate::{conf, filesystem};

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
///
/// Note that if the canvas is not of the same size as the screen, and you want
/// to render using coordinates relative to the canvas' coordinate system, you
/// need to call [`graphics::set_screen_coordinates`](fn.set_screen_coordinates.html)
/// and pass in a rectangle with position (0, 0) and a size equal to that of the
/// canvas.
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

    /// Gets the backend `Image` that is being rendered to. Note that this will be flipped but otherwise the same, use the [`to_image`](#method.to_image) function for the unflipped version.
    pub fn raw_image(&self) -> &Image {
        &self.image
    }

    /// Creates an `Image` with the content of its raw counterpart but transformed to behave like a normal `Image`.
    pub fn to_image(&self, ctx: &mut Context) -> GameResult<Image> {
        let pixel_data = self.to_rgba8(ctx)?;
        Image::from_rgba8(ctx, self.image.width, self.image.height, &pixel_data)
    }

    /// Gets the backend `Target` that is being rendered to.
    pub fn target(&self) -> &RawRenderTargetView<S::Resources> {
        &self.target
    }

    /// Dumps the flipped `Canvas`'s data to a `Vec` of `u8` RBGA8 values.
    pub fn to_rgba8(&self, ctx: &mut Context) -> GameResult<Vec<u8>> {
        let mut pixel_data = self.image.to_rgba8(ctx)?;
        flip_pixel_data(
            &mut pixel_data,
            self.image.width as usize,
            self.image.height as usize,
        );
        Ok(pixel_data)
    }

    /// Encode the `Canvas`'s content to the given file format and
    /// write it out to the given path.
    ///
    /// See the [`filesystem`](../filesystem/index.html) module docs for where exactly
    /// the file will end up.
    pub fn encode<P: AsRef<path::Path>>(
        &self,
        ctx: &mut Context,
        format: ImageFormat,
        path: P,
    ) -> GameResult {
        use std::io;
        let data = self.to_rgba8(ctx)?;
        let f = filesystem::create(ctx, path)?;
        let writer = &mut io::BufWriter::new(f);
        let color_format = ::image::ColorType::Rgba8;
        match format {
            ImageFormat::Png => ::image::png::PngEncoder::new(writer)
                .encode(
                    &data,
                    u32::from(self.width()),
                    u32::from(self.height()),
                    color_format,
                )
                .map_err(Into::into),
        }
    }

    /// Return the width of the canvas.
    pub fn width(&self) -> u16 {
        self.image.width
    }

    /// Return the height of the canvas.
    pub fn height(&self) -> u16 {
        self.image.height
    }

    /// Returns the dimensions of the canvas.
    pub fn dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, f32::from(self.width()), f32::from(self.height()))
    }

    /// Get the filter mode for the canvas.
    pub fn filter(&self) -> FilterMode {
        self.image.filter()
    }

    /// Set the filter mode for the canvas.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.image.set_filter(mode)
    }
}

impl Drawable for Canvas {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        self.debug_id.assert(ctx);

        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let scale_x = param.src.w * f32::from(self.width());
        let scale_y = param.src.h * f32::from(self.height());

        let mat = glam::Mat4::from(param.trans.to_bare_matrix());
        let new_param = param.transform(
            mat * Matrix4::from_scale(glam::vec3(scale_x, scale_y, 1.0))
                * glam::Mat4::from_scale_rotation_translation(
                    glam::vec3(1.0, -1.0, 1.0),
                    Quat::identity(),
                    glam::vec3(0.0, 1.0, 0.0),
                ),
        );
        let new_src = Rect {
            x: param.src.x,
            y: (1.0 - param.src.h) - param.src.y,
            w: param.src.w,
            h: param.src.h,
        };
        let new_param = new_param.src(new_src);

        image::draw_image_raw(&self.image, ctx, new_param)
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

/// Fast non-allocating function for flipping pixel data in an image vertically
fn flip_pixel_data(rgba: &mut Vec<u8>, width: usize, height: usize) {
    // cast the buffer into u32 so we can easily access the pixels themselves
    // splits the pixel buffer into an upper (first) and a lower (second) half
    let pixels: (&mut [u32], &mut [u32]) =
        bytemuck::cast_slice_mut(rgba.as_mut_slice()).split_at_mut(width * height / 2);

    // When the image has an uneven height, it will split the buffer in the middle of a row.
    // This will decrease pixel count so that the x,y in the loop will never enter the split row since
    // for uneven height images the middle row will stay the same anyway
    let pixel_count = if height % 2 == 0 {
        width * height / 2
    } else {
        width * height / 2 - width / 2
    };
    // Even though we removed uwidth / 2 from pixel_count,
    // the second half of the buffer's size will still contain that data so
    // we need to offset the index on that by the size of said data
    let second_set_offset = if height % 2 == 0 {
        // even height (evenness on width doesn't matter)
        0
    } else if width % 2 == 0 {
        // uneven height but even width
        width / 2
    } else {
        // uneven height and uneven width
        width / 2 + 1
    };
    for i in 0..pixel_count {
        let x = i % width;
        let y = i / width;
        let reverse_y = height / 2 - y - 1;

        let idx = (y * width) + x;
        let second_idx = (reverse_y * width) + x + second_set_offset;

        std::mem::swap(&mut pixels.0[idx], &mut pixels.1[second_idx]);
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
