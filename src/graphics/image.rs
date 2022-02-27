//!

use super::{
    context::GraphicsContext,
    gpu::arc::{ArcTexture, ArcTextureView},
    Rect,
};
use crate::{GameError, GameResult};
use std::{io::Read, num::NonZeroU32};

/// Pixel format of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageFormat {
    /// Single 8-bit channel (red). Most appropriate for grayscale images that do not need float precision.
    R8,
    /// Four 8-bit channels (RGBA), non-SRGB. Most appropriate for non-SRGB color images, e.g. texture maps.
    Rgba8,
    /// Four 8-bit channels (RGBA), SRGB. Most appropriate for SRGB color images, such as sprites.
    Rgba8Srgb,
    /// Single 32-bit float channel. Most appropriate for depth targets.
    Depth32,
    /// 32-bit channel split into 24 bits for depth and 8 bits for stencil.
    /// Appropriate for both depth and stencil targets.
    ///
    /// Prefer `Depth32` if stencil is not needed.
    Depth24Stencil8,
}

impl ImageFormat {
    pub(crate) fn supports_depth(&self) -> bool {
        matches!(self, ImageFormat::Depth32 | ImageFormat::Depth24Stencil8)
    }

    pub(crate) fn bytes_per_pixel(&self) -> u32 {
        match self {
            ImageFormat::R8 => 1,
            ImageFormat::Rgba8 | ImageFormat::Rgba8Srgb => 4,
            ImageFormat::Depth32 | ImageFormat::Depth24Stencil8 => 4,
        }
    }
}

impl From<ImageFormat> for wgpu::TextureFormat {
    fn from(f: ImageFormat) -> Self {
        match f {
            ImageFormat::R8 => wgpu::TextureFormat::R8Unorm,
            ImageFormat::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
            ImageFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            ImageFormat::Depth32 => wgpu::TextureFormat::Depth32Float,
            ImageFormat::Depth24Stencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
        }
    }
}

/// Handle to an image stored in GPU memory.
#[derive(Debug, Clone)]
pub struct Image {
    pub(crate) texture: ArcTexture,
    pub(crate) view: ArcTextureView,
    format: ImageFormat,
    width: u32,
    height: u32,
    samples: u32,
}

impl Image {
    /// Creates a new image specifically for use with a [Canvas].
    pub fn new_canvas_image(
        gfx: &GraphicsContext,
        format: ImageFormat,
        width: u32,
        height: u32,
        samples: u32,
    ) -> Self {
        Self::new(
            gfx,
            format,
            width,
            height,
            samples,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }

    /// Creates a new image initialized with given pixel data.
    pub fn from_pixels(
        gfx: &GraphicsContext,
        pixels: &[u8],
        format: ImageFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let image = Self::new(
            gfx,
            format,
            width,
            height,
            1,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        gfx.queue.write_texture(
            image.texture.as_image_copy(),
            pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(format.bytes_per_pixel() * width).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        image
    }

    /// Creates a new image initialized with pixel data loaded from an encoded image `Read` (e.g. PNG or JPEG).
    #[allow(unused_results)]
    pub fn from_read<R: Read>(gfx: &GraphicsContext, read: &mut R, srgb: bool) -> GameResult<Self> {
        let mut encoded = Vec::new();
        read.read_to_end(&mut encoded)?;
        let decoded = image::load_from_memory(&encoded[..])
            .map_err(|_| GameError::ResourceLoadError(String::from("failed to load image")))?;
        let rgba8 = decoded.to_rgba8();
        let (width, height) = (rgba8.width(), rgba8.height());

        Ok(Self::from_pixels(
            gfx,
            rgba8.as_ref(),
            if srgb {
                ImageFormat::Rgba8Srgb
            } else {
                ImageFormat::Rgba8
            },
            width,
            height,
        ))
    }

    fn new(
        gfx: &GraphicsContext,
        format: ImageFormat,
        width: u32,
        height: u32,
        samples: u32,
        usage: wgpu::TextureUsages,
    ) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert!(samples > 0);

        let texture = ArcTexture::new(gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format: format.into(),
            usage,
        }));

        let view =
            ArcTextureView::new(texture.as_ref().create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(format.into()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                base_array_layer: 0,
                array_layer_count: Some(NonZeroU32::new(1).unwrap()),
            }));

        Image {
            texture,
            view,
            format,
            width,
            height,
            samples,
        }
    }

    /// Returns the image format of this image.
    #[inline]
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    /// Returns the number of MSAA samples this image has.
    #[inline]
    pub fn samples(&self) -> u32 {
        self.samples
    }

    /// Returns the width (in pixels) of the image.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height (in pixels) of the image.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Helper function that calculates a sub-rectangle of this image in UV coordinates, given pixel coordinates.
    pub fn uv_rect(&self, x: u32, y: u32, w: u32, h: u32) -> Rect {
        Rect {
            x: x as f32 / self.width as f32,
            y: y as f32 / self.height as f32,
            w: w as f32 / self.width as f32,
            h: h as f32 / self.height as f32,
        }
    }
}

/// An image which is sized relative to the screen.
/// This is primarily for canvas images.
#[derive(Debug, Clone)]
pub struct ScreenImage {
    image: Image,
    format: ImageFormat,
    size: (f32, f32),
    samples: u32,
}

impl ScreenImage {
    /// Creates a new [ScreenImage] with the given parameters.
    ///
    /// `width` and `height` specify the fraction of the framebuffer width and height that the [Image] will have.
    /// For example, `width = 1.0` and `height = 1.0` means the image will be the same size as the framebuffer.
    pub fn new(
        gfx: &GraphicsContext,
        format: ImageFormat,
        width: f32,
        height: f32,
        samples: u32,
    ) -> Self {
        assert!(width > 0.);
        assert!(height > 0.);
        assert!(samples > 0);

        ScreenImage {
            image: Self::create(gfx, format, (width, height), samples),
            format,
            size: (width, height),
            samples,
        }
    }

    /// Returns the inner [Image], also recreating it if the framebuffer has been resized.
    pub fn image(&mut self, gfx: &GraphicsContext) -> Image {
        if Self::size(gfx, self.size) != (self.image.width(), self.image.height()) {
            self.image = Self::create(gfx, self.format, self.size, self.samples);
        }
        self.image.clone()
    }

    fn size(gfx: &GraphicsContext, (width, height): (f32, f32)) -> (u32, u32) {
        let size = gfx.window.inner_size();
        let width = (size.width as f32 * width) as u32;
        let height = (size.height as f32 * height) as u32;
        (width, height)
    }

    fn create(gfx: &GraphicsContext, format: ImageFormat, size: (f32, f32), samples: u32) -> Image {
        let (width, height) = Self::size(gfx, size);
        Image::new_canvas_image(gfx, format, width, height, samples)
    }
}
