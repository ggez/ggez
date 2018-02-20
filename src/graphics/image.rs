use std::path;
use std::io::Read;

use gfx;
use gfx_device_gl;
use image;

use graphics::*;
use graphics::shader::*;
use context::Context;
use GameResult;
use GameError;

/// Generic in-GPU-memory image data available to be drawn on the screen.
#[derive(Clone)]
pub struct ImageGeneric<R>
where
    R: gfx::Resources,
{
    pub(crate) texture: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub(crate) sampler_info: gfx::texture::SamplerInfo,
    pub(crate) blend_mode: Option<BlendMode>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// In-GPU-memory image data available to be drawn on the screen,
/// using the OpenGL backend.
pub type Image = ImageGeneric<gfx_device_gl::Resources>;

impl Image {
    /// Load a new image from the file at the given path.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Image> {
        let img = {
            let mut buf = Vec::new();
            let mut reader = context.filesystem.open(path)?;
            reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_rgba()
        };
        let (width, height) = img.dimensions();
        Image::from_rgba8(context, width as u16, height as u16, &img)
    }

    /// Creates a new `Image` from the given buffer of `u8` RGBA values.
    pub fn from_rgba8(
        context: &mut Context,
        width: u16,
        height: u16,
        rgba: &[u8],
    ) -> GameResult<Image> {
        Image::make_raw(
            &mut context.gfx_context.factory,
            &context.gfx_context.default_sampler_info,
            width,
            height,
            rgba,
        )
    }
    /// A helper function that just takes a factory directly so we can make an image
    /// without needing the full context object, so we can create an Image while still
    /// creating the GraphicsContext.
    pub(crate) fn make_raw(
        factory: &mut gfx_device_gl::Factory,
        sampler_info: &texture::SamplerInfo,
        width: u16,
        height: u16,
        rgba: &[u8],
    ) -> GameResult<Image> {
        if width == 0 || height == 0 {
            let msg = format!(
                "Tried to create a texture of size {}x{}, each dimension must
                be >0",
                width, height
            );
            return Err(GameError::ResourceLoadError(msg));
        }
        let kind = gfx::texture::Kind::D2(width, height, gfx::texture::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, gfx::texture::Mipmap::Provided, &[rgba])?;
        Ok(Image {
            texture: view,
            sampler_info: *sampler_info,
            blend_mode: None,
            width: u32::from(width),
            height: u32::from(height),
        })
    }

    /// A little helper function that creates a new Image that is just
    /// a solid square of the given size and color.  Mainly useful for
    /// debugging.
    pub fn solid(context: &mut Context, size: u16, color: Color) -> GameResult<Image> {
        // let pixel_array: [u8; 4] = color.into();
        let (r, g, b, a) = color.into();
        let pixel_array: [u8; 4] = [r, g, b, a];
        let size_squared = size as usize * size as usize;
        let mut buffer = Vec::with_capacity(size_squared);
        for _i in 0..size_squared {
            buffer.extend(&pixel_array[..]);
        }
        Image::from_rgba8(context, size, size, &buffer)
    }

    /// Return the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Return the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the filter mode for the image.
    pub fn get_filter(&self) -> FilterMode {
        self.sampler_info.filter.into()
    }

    /// Set the filter mode for the image.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.sampler_info.filter = mode.into();
    }

    /// Returns the dimensions of the image.
    pub fn get_dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
    }

    /// Gets the `Image`'s `WrapMode` along the X and Y axes.
    pub fn get_wrap(&self) -> (WrapMode, WrapMode) {
        (self.sampler_info.wrap_mode.0, self.sampler_info.wrap_mode.1)
    }

    /// Sets the `Image`'s `WrapMode` along the X and Y axes.
    pub fn set_wrap(&mut self, wrap_x: WrapMode, wrap_y: WrapMode) {
        self.sampler_info.wrap_mode.0 = wrap_x;
        self.sampler_info.wrap_mode.1 = wrap_y;
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<Image: {}x{}, {:p}, texture address {:p}, sampler: {:?}>",
            self.width(),
            self.height(),
            self,
            &self.texture,
            &self.sampler_info
        )
    }
}

impl Drawable for Image {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        let src_width = param.src.w;
        let src_height = param.src.h;
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let real_scale = Point2::new(
            src_width * param.scale.x * self.width as f32,
            src_height * param.scale.y * self.height as f32,
        );
        let mut new_param = param;
        new_param.scale = real_scale;
        gfx.update_instance_properties(new_param)?;
        let sampler = gfx.samplers
            .get_or_insert(self.sampler_info, gfx.factory.as_mut());
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        gfx.data.tex = (self.texture.clone(), sampler);
        let previous_mode: Option<BlendMode> = if let Some(mode) = self.blend_mode {
            let current_mode = gfx.get_blend_mode();
            if current_mode != mode {
                gfx.set_blend_mode(mode)?;
                Some(current_mode)
            } else {
                None
            }
        } else {
            None
        };
        gfx.draw(None)?;
        if let Some(mode) = previous_mode {
            gfx.set_blend_mode(mode)?;
        }
        Ok(())
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }
    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}