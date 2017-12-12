//! SpriteBatch type.

use context::Context;
use graphics;
use error;
use gfx;
use gfx::Factory;
use GameResult;
use super::shader::BlendMode;

// TODO:
//
// Owning the given Image is inconvenient because we might want, say,
// the same Rc<Image> shared among many SpriteBatch'es.
//
// But draw() doesn't handle that particularly well...
//
// The right way to fix this might be to have another object that implements
// Drawable that is created from a SpriteBatch and an image reference.
// BoundSpriteBatch or something.  That *feels* like the right way to go...
//
// Or just not implement Drawable and provide your own drawing functions, though
// that's squirrelly.
//
// Oh, or maybe make it take a Cow<Image> ?  that might work.
//
// For now though, let's mess around with the gfx bits rather than the ggez bits.
// We need to be able to, essentially, have an array of RectProperties.


/// A SpriteBatch draws a number of copies of the same image, using a single draw call.
///
/// This is generally faster than drawing the same sprite with many invocations of `draw()`,
/// though it has a bit of overhead to set up the batch.  This makes it run very slowly
/// in Debug mode; you need to build with optimizations enabled to really get the
/// speed boost.
#[derive(Debug)]
pub struct SpriteBatch {
    image: graphics::Image,
    sprites: Vec<graphics::InstanceProperties>,
    blend_mode: Option<BlendMode>,
}

/// An index of a particular sprite in a SpriteBatch.
pub type SpriteIdx = usize;

impl SpriteBatch {
    /// Creates a new `SpriteBatch`, drawing with the given image.
    pub fn new(image: graphics::Image) -> Self {
        Self {
            image: image,
            sprites: vec![],
            blend_mode: None,
        }
    }

    /// Adds a new sprite to the sprite batch.
    ///
    /// Returns a handle with which to modify the sprite using `set()`
    pub fn add(&mut self, param: graphics::DrawParam) -> SpriteIdx {
        let src_width = param.src.w;
        let src_height = param.src.h;
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let real_scale = graphics::Point2::new(src_width * param.scale.x * self.image.width as f32,
                                               src_height * param.scale.y *
                                               self.image.height as f32);
        let mut new_param = param;
        new_param.scale = real_scale;
        // Not entirely sure why the inversion is necessary, but oh well.
        new_param.offset.x *= -1.0 * param.scale.x;
        new_param.offset.y *= param.scale.y;
        self.sprites.push(new_param.into());
        self.sprites.len() - 1
    }

    /// Alters a sprite in the batch to use the given draw params
    pub fn set(&mut self, handle: SpriteIdx, param: graphics::DrawParam) -> GameResult<()> {
        if handle < self.sprites.len() {
            self.sprites[handle] = param.into();
            Ok(())
        } else {
            Err(error::GameError::RenderError(String::from("Provided index is out of bounds.")))
        }
    }

    /// Immediately sends all data in the batch to the graphics card.
    ///
    /// Generally just calling `graphics::draw()` on the `SpriteBatch`
    /// will do this automatically.
    pub fn flush(&self, ctx: &mut Context) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        if gfx.data.rect_instance_properties.len() < self.sprites.len() {
            gfx.data.rect_instance_properties = gfx.factory
                .create_buffer(self.sprites.len(),
                               gfx::buffer::Role::Vertex,
                               gfx::memory::Usage::Dynamic,
                               gfx::TRANSFER_DST)?;
        }
        gfx.encoder
            .update_buffer(&gfx.data.rect_instance_properties, &self.sprites[..], 0)?;
        Ok(())
    }

    /// Removes all data from the sprite batch.
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Unwraps the contained `Image`
    pub fn into_inner(self) -> graphics::Image {
        self.image
    }

    /// Replaces the contained `Image`, returning the old one.
    pub fn set_image(&mut self, image: graphics::Image) -> graphics::Image {
        use std::mem;
        mem::replace(&mut self.image, image)
    }
}

impl graphics::Drawable for SpriteBatch {
    /// Does not properly work yet, ideally the position, scale, etc. of the given
    /// DrawParam would be added to the DrawParam for each sprite.
    fn draw_ex(&self, ctx: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        self.flush(ctx)?;
        let gfx = &mut ctx.gfx_context;
        let sampler = gfx.samplers
            .get_or_insert(self.image.sampler_info, gfx.factory.as_mut());
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        gfx.data.tex = (self.image.texture.clone(), sampler);
        let mut slice = gfx.quad_slice.clone();
        slice.instances = Some((self.sprites.len() as u32, 0));
        let curr_transform = gfx.get_transform();
        gfx.push_transform(param.into_matrix() * curr_transform);
        gfx.calculate_transform_matrix();
        gfx.update_globals()?;
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
        gfx.draw(Some(&slice))?;
        if let Some(mode) = previous_mode {
            gfx.set_blend_mode(mode)?;
        }
        gfx.pop_transform();
        gfx.calculate_transform_matrix();
        gfx.update_globals()?;
        Ok(())
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }
    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
