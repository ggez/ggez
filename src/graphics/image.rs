use super::{
    context::GraphicsContext,
    gpu::arc::{ArcTexture, ArcTextureView},
    Canvas, Color, Draw, DrawParam, Drawable, Rect, WgpuContext,
};
use crate::{context::Has, Context, GameError, GameResult};
use image::ImageEncoder;
use std::path::Path;
use std::{io::Read, num::NonZeroU32};

// maintaing a massive enum of all possible texture formats?
// screw that.
/// Describes the pixel format of an image.
pub type ImageFormat = wgpu::TextureFormat;

/// Describes the format of an encoded image.
pub type ImageEncodingFormat = ::image::ImageFormat;

/// Handle to an image stored in GPU memory.
#[derive(Debug, Clone)]
pub struct Image {
    pub(crate) texture: ArcTexture,
    pub(crate) view: ArcTextureView,
    pub(crate) format: ImageFormat,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) samples: u32,
}

impl Image {
    /// Creates a new image specifically for use with a [Canvas](crate::graphics::Canvas).
    pub fn new_canvas_image(
        gfx: &impl Has<GraphicsContext>,
        format: ImageFormat,
        width: u32,
        height: u32,
        samples: u32,
    ) -> Self {
        let gfx = gfx.retrieve();
        Self::new(
            &gfx.wgpu,
            format,
            width,
            height,
            samples,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        )
    }

    /// Creates a new image initialized with given pixel data.
    pub fn from_pixels(
        gfx: &impl Has<GraphicsContext>,
        pixels: &[u8],
        format: ImageFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let gfx = gfx.retrieve();
        Self::from_pixels_wgpu(&gfx.wgpu, pixels, format, width, height)
    }

    pub(crate) fn from_pixels_wgpu(
        wgpu: &WgpuContext,
        pixels: &[u8],
        format: ImageFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let image = Self::new(
            wgpu,
            format,
            width,
            height,
            1,
            wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );

        wgpu.queue.write_texture(
            image.texture.as_image_copy(),
            pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    NonZeroU32::new(u32::from(format.describe().block_size) * width).unwrap(),
                ),
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

    /// A little helper function that creates a new `Image` that is just a solid square of the given size and color. Mainly useful for debugging.
    pub fn from_solid(gfx: &impl Has<GraphicsContext>, size: u32, color: Color) -> Self {
        let pixels = (0..(size * size))
            .flat_map(|_| {
                let (r, g, b, a) = color.to_rgba();
                [r, g, b, a]
            })
            .collect::<Vec<_>>();
        Self::from_pixels(
            gfx,
            &pixels,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            size,
            size,
        )
    }

    /// Creates a new image initialized with pixel data loaded from a given path as an
    /// encoded image `Read` (e.g. PNG or JPEG).
    #[allow(unused_results)]
    pub fn from_path(gfx: &impl Has<GraphicsContext>, path: impl AsRef<Path>) -> GameResult<Self> {
        let gfx = gfx.retrieve();

        let mut encoded = Vec::new();
        gfx.fs.open(path)?.read_to_end(&mut encoded)?;

        Self::from_bytes(gfx, encoded.as_slice())
    }

    /// Creates a new image initialized with pixel data from a given encoded image (e.g. PNG or JPEG)
    pub fn from_bytes(gfx: &impl Has<GraphicsContext>, encoded: &[u8]) -> Result<Image, GameError> {
        let decoded = image::load_from_memory(encoded)
            .map_err(|_| GameError::ResourceLoadError(String::from("failed to load image")))?;
        let rgba8 = decoded.to_rgba8();
        let (width, height) = (rgba8.width(), rgba8.height());

        Ok(Self::from_pixels(
            gfx,
            rgba8.as_ref(),
            ImageFormat::Rgba8UnormSrgb,
            width,
            height,
        ))
    }

    fn new(
        wgpu: &WgpuContext,
        format: ImageFormat,
        width: u32,
        height: u32,
        samples: u32,
        usage: wgpu::TextureUsages,
    ) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert!(samples > 0);

        let texture = ArcTexture::new(wgpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        }));

        let view =
            ArcTextureView::new(texture.as_ref().create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(format),
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

    /// Returns the underlying [`wgpu::Texture`] and [`wgpu::TextureView`] for this [`Image`].
    #[inline]
    pub fn wgpu(&self) -> (&wgpu::Texture, &wgpu::TextureView) {
        (&self.texture, &self.view)
    }

    /// Reads the pixels of this `ImageView` and returns as `Vec<u8>`.
    /// The format matches the GPU image format.
    ///
    /// **This is a very expensive operation - call sparingly.**
    pub fn to_pixels(&self, gfx: &impl Has<GraphicsContext>) -> GameResult<Vec<u8>> {
        let gfx = gfx.retrieve();
        if self.samples > 1 {
            return Err(GameError::RenderError(String::from(
                "cannot read the pixels of a multisampled image; resolve this image with a canvas",
            )));
        }

        let block_size = u64::from(self.format.describe().block_size);

        let buffer = gfx.wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: block_size * u64::from(self.width) * u64::from(self.height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let cmd = {
            let mut encoder = gfx
                .wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                self.texture.as_image_copy(),
                wgpu::ImageCopyBuffer {
                    buffer: &buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            NonZeroU32::new(block_size as u32 * self.width).unwrap(),
                        ),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
            );
            encoder.finish()
        };

        let _ = gfx.wgpu.queue.submit([cmd]);

        // wait...
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
        let _ = gfx.wgpu.device.poll(wgpu::Maintain::Wait);
        let map_result = rx
            .recv()
            .expect("All senders dropped, this should not be possible.");
        map_result?;

        let out = buffer.slice(..).get_mapped_range().to_vec();
        Ok(out)
    }

    /// Encodes the `ImageView` to the given file format and return the encoded bytes.
    ///
    /// **This is a very expensive operation - call sparingly.**
    pub fn encode(
        &self,
        ctx: &Context,
        format: ImageEncodingFormat,
        path: impl AsRef<std::path::Path>,
    ) -> GameResult {
        let color = match self.format {
            ImageFormat::Rgba8Unorm | ImageFormat::Rgba8UnormSrgb => ::image::ColorType::Rgba8,
            ImageFormat::R8Unorm => ::image::ColorType::L8,
            ImageFormat::R16Unorm => ::image::ColorType::L16,
            format => {
                return Err(GameError::RenderError(format!(
                    "cannot ImageView::encode for the {:#?} GPU image format",
                    format
                )))
            }
        };

        let pixels = self.to_pixels(ctx)?;
        let f = ctx.fs.create(path)?;
        let writer = &mut std::io::BufWriter::new(f);

        match format {
            ImageEncodingFormat::Png => ::image::codecs::png::PngEncoder::new(writer)
                .write_image(&pixels, self.width, self.height, color)
                .map_err(Into::into),
            ImageEncodingFormat::Bmp => ::image::codecs::bmp::BmpEncoder::new(writer)
                .encode(&pixels, self.width, self.height, color)
                .map_err(Into::into),
            _ => Err(GameError::RenderError(String::from(
                "cannot ImageView::encode for formats other than Png and Bmp",
            ))),
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

impl Drawable for Image {
    fn draw(&self, canvas: &mut Canvas, param: impl Into<DrawParam>) {
        canvas.push_draw(
            Draw::Mesh {
                mesh: canvas.default_resources().mesh.clone(),
                image: self.clone(),
                scale: true,
            },
            param.into(),
        );
    }

    fn dimensions(&self, _gfx: &impl Has<GraphicsContext>) -> Option<Rect> {
        Some(Rect {
            x: 0.,
            y: 0.,
            w: self.width() as _,
            h: self.height() as _,
        })
    }
}

/// An image which is sized relative to the screen.
/// This is primarily for canvas images.
#[derive(Debug, Clone)]
pub struct ScreenImage {
    image: Image,
    format: wgpu::TextureFormat,
    size: (f32, f32),
    samples: u32,
}

impl ScreenImage {
    /// Creates a new [`ScreenImage`] with the given parameters.
    ///
    /// `width` and `height` specify the fraction of the framebuffer width and height that the [Image] will have.
    /// For example, `width = 1.0` and `height = 1.0` means the image will be the same size as the framebuffer.
    ///
    /// If `format` is `None` then the format will be inferred from the surface format.
    pub fn new(
        gfx: &impl Has<GraphicsContext>,
        format: impl Into<Option<ImageFormat>>,
        width: f32,
        height: f32,
        samples: u32,
    ) -> Self {
        let gfx = gfx.retrieve();
        assert!(width > 0.);
        assert!(height > 0.);
        assert!(samples > 0);

        let format = format.into().unwrap_or_else(|| gfx.surface_format());

        ScreenImage {
            image: Self::create(gfx, format, (width, height), samples),
            format,
            size: (width, height),
            samples,
        }
    }

    /// Returns the inner [Image], also recreating it if the framebuffer has been resized.
    pub fn image(&mut self, gfx: &impl Has<GraphicsContext>) -> Image {
        if Self::size(gfx, self.size) != (self.image.width(), self.image.height()) {
            self.image = Self::create(gfx, self.format, self.size, self.samples);
        }
        self.image.clone()
    }

    fn size(gfx: &impl Has<GraphicsContext>, (width, height): (f32, f32)) -> (u32, u32) {
        let gfx = gfx.retrieve();
        let size = gfx.window.inner_size();
        let width = (size.width as f32 * width) as u32;
        let height = (size.height as f32 * height) as u32;
        (width.max(1), height.max(1))
    }

    fn create(
        gfx: &impl Has<GraphicsContext>,
        format: wgpu::TextureFormat,
        size: (f32, f32),
        samples: u32,
    ) -> Image {
        let (width, height) = Self::size(gfx, size);
        Image::new_canvas_image(gfx, format, width, height, samples)
    }
}
