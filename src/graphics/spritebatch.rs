//! A `SpriteBatch` is a way to efficiently draw a large
//! number of copies of the same image, or part of the same image.  It's
//! useful for implementing tiled maps, spritesheets, particles, and
//! other such things.
//!
//! Essentially this uses a technique called "instancing" to queue up
//! a large amount of location/position data in a buffer, then feed it
//! to the graphics card all in one go.

use context::Context;
use graphics;
use error;
use gfx;
use gfx::Factory;
use GameResult;
use super::shader::BlendMode;

/// A `SpriteBatch` draws a number of copies of the same image, using a single draw call.
///
/// This is generally faster than drawing the same sprite with many invocations of `draw()`,
/// though it has a bit of overhead to set up the batch.  This makes it run very slowly
/// in Debug mode because it spends a lot of time on array bounds checking and
/// un-optimized math; you need to build with optimizations enabled to really get the
/// speed boost.
#[derive(Debug, Clone)]
pub struct SpriteBatch {
    image: graphics::Image,
    sprites: Vec<graphics::DrawParam>,
    blend_mode: Option<BlendMode>,
}

/// A drawable combination of a `SpriteBatch` and a specific `Image`.
/// It is not always convenient for a `SpriteBatch` to own the image
/// it is drawing, so this structure lets you override the image with
/// a borrowed one.
///
/// This is now deprecated; an `Image` is cheap to clone and
/// this was never terribly useful to begin with.
#[deprecated]
#[derive(Debug)]
pub struct BoundSpriteBatch<'a> {
    image: &'a graphics::Image,
    batch: &'a mut SpriteBatch,
}

/// An index of a particular sprite in a `SpriteBatch`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteIdx(usize);

impl SpriteBatch {
    /// Creates a new `SpriteBatch`, drawing with the given image.
    pub fn new(image: graphics::Image) -> Self {
        Self {
            image,
            sprites: vec![],
            blend_mode: None,
        }
    }

    /// Adds a new sprite to the sprite batch.
    ///
    /// Returns a handle with which type to modify the sprite using `set()`
    pub fn add(&mut self, param: graphics::DrawParam) -> SpriteIdx {
        self.sprites.push(param);
        SpriteIdx(self.sprites.len() - 1)
    }

    /// Alters a sprite in the batch to use the given draw params
    pub fn set(&mut self, handle: SpriteIdx, param: graphics::DrawParam) -> GameResult<()> {
        if handle.0 < self.sprites.len() {
            self.sprites[handle.0] = param;
            Ok(())
        } else {
            Err(error::GameError::RenderError(String::from(
                "Provided index is out of bounds.",
            )))
        }
    }

    /// Immediately sends all data in the batch to the graphics card.
    ///
    /// Generally just calling `graphics::draw()` on the `SpriteBatch`
    /// will do this automaticassertally.
    fn flush(
        &self,
        ctx: &mut Context,
        image: &graphics::Image,
        draw_color: Option<graphics::Color>,
    ) -> GameResult<()> {
        // This is a little awkward but this is the right place
        // to do whatever transformations need to happen to DrawParam's.
        // We have a Context, and *everything* must pass through this
        // function to be drawn, so.
        // Though we do awkwardly have to allocate a new vector.
        assert!(draw_color.is_some());
        let new_sprites = self.sprites
            .iter()
            .map(|param| {
                // Copy old params
                let mut new_param = *param;
                let src_width = param.src.w;
                let src_height = param.src.h;
                let real_scale = graphics::Point2::new(
                    src_width * param.scale.x * image.width as f32,
                    src_height * param.scale.y * image.height as f32,
                );
                new_param.scale = real_scale;
                // If we have no color, our color is white.
                // This is fine because coloring the whole spritebatch is possible
                // with graphics::set_color(); this just inherits from that.
                new_param.color = new_param.color.or(draw_color);
                graphics::InstanceProperties::from(new_param)
            })
            .collect::<Vec<_>>();

        let gfx = &mut ctx.gfx_context;
        if gfx.data.rect_instance_properties.len() < self.sprites.len() {
            gfx.data.rect_instance_properties = gfx.factory.create_buffer(
                self.sprites.len(),
                gfx::buffer::Role::Vertex,
                gfx::memory::Usage::Dynamic,
                gfx::memory::Bind::TRANSFER_DST,
            )?;
        }
        gfx.encoder
            .update_buffer(&gfx.data.rect_instance_properties, &new_sprites[..], 0)?;
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

    /// Create an object which draws the current sprite batch with a different image.
    #[deprecated]
    #[allow(deprecated)]
    pub fn with_image<'a>(&'a mut self, image: &'a graphics::Image) -> BoundSpriteBatch<'a> {
        BoundSpriteBatch {
            image,
            batch: self,
        }
    }
}

#[deprecated]
#[allow(deprecated)]
impl<'a> graphics::Drawable for BoundSpriteBatch<'a> {
    fn draw_ex(&self, ctx: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        // Awkwardly we must update values on all sprites and such.
        // Also awkwardly we have this chain of colors with differing priorities.
        let fg = Some(ctx.gfx_context.foreground_color);
        let draw_color = param.color.or(fg);
        self.batch.flush(ctx, self.image, draw_color)?;
        let gfx = &mut ctx.gfx_context;
        let sampler = gfx.samplers
            .get_or_insert(self.image.sampler_info, gfx.factory.as_mut());
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        gfx.data.tex = (self.image.texture.clone(), sampler);
        let mut slice = gfx.quad_slice.clone();
        slice.instances = Some((self.batch.sprites.len() as u32, 0));
        let curr_transform = gfx.get_transform();
        gfx.push_transform(param.into_matrix() * curr_transform);
        gfx.calculate_transform_matrix();
        gfx.update_globals()?;
        let previous_mode: Option<BlendMode> = if let Some(mode) = self.batch.blend_mode {
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
        self.batch.blend_mode = mode;
    }

    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.batch.blend_mode
    }
}

impl graphics::Drawable for SpriteBatch {
    fn draw_ex(&self, ctx: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        // Awkwardly we must update values on all sprites and such.
        // Also awkwardly we have this chain of colors with differing priorities.
        let fg = Some(ctx.gfx_context.foreground_color);
        let draw_color = param.color.or(fg);
        self.flush(ctx, &self.image, draw_color)?;
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
