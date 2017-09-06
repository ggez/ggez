/*
use sdl2;
use image;
use gfx;
use gfx::texture;
use gfx::traits::Device;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx::Factory;
*/

use context::Context;
use graphics;
use GameResult;

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
#[derive(Debug)]
pub struct SpriteBatch {
    image: graphics::Image,
    sprites: Vec<SpriteInfo>,
    quads: Vec<graphics::Rect>
}

pub type SpriteIdx = usize;
pub type QuadIdx = usize;

#[derive(Debug)]
pub struct SpriteInfo {
    param: graphics::DrawParam,
    quad_handle: Option<QuadIdx>
}

impl SpriteBatch {
    /// Creates a new `SpriteBatch`, drawing with the given image.
    pub fn new(image: graphics::Image) -> Self {
        Self {
            image: image,
            sprites: vec![],
            quads: vec![]
        }
    }

    /// Adds a new sprite to the sprite batch.
    ///
    /// Returns a handle with which to modify the sprite using `set()`
    pub fn add(&mut self, param: graphics::DrawParam) -> SpriteIdx {
        self.sprites.push(
            SpriteInfo{
                param,
                quad_handle: None
            }
        );
        self.sprites.len() - 1
    }

    /// Adds a new quad defining a region of the source image to use
    /// when drawing a sprite. Allows use of a texture atlas to batch
    /// multiple types of tiles etc at once.
    ///
    /// Returns a handle with which to modify the quad or reference it
    /// when adding a new sprite.
    pub fn add_quad(&mut self, quad: graphics::Rect) -> QuadIdx {
        self.quads.push(quad);
        self.quads.len() - 1
    }

    /// Adds a new sprite to the batch using the given quad handle.
    ///
    /// Returns a handle with which to modify the sprite using `set()`
    pub fn add_with_quad(
        &mut self,
        param: graphics::DrawParam,
        quad_handle: QuadIdx
    ) -> SpriteIdx {
        self.sprites.push(
            SpriteInfo {
                param,
                quad_handle: Some(quad_handle)
            }
        );
        self.sprites.len() - 1
    }

    /// Alters a sprite in the batch to use the given draw params
    pub fn set(&mut self, handle: SpriteIdx, param: graphics::DrawParam) {
        self.sprites[handle].param = param;
    }

    /// Alters a sprite in the batch to use the given `SpriteInfo`
    pub fn set_ex(&mut self, handle: SpriteIdx, info: SpriteInfo) {
        self.sprites[handle] = info;
    }

    /// Alters a quad in the batch to use the given Rect instead
    pub fn set_quad(&mut self, handle: QuadIdx, quad: graphics::Rect) {
        self.quads[handle] = quad;
    }

    /// Immediately sends all data in the batch to the graphics card.
    ///
    /// Generally just calling `graphics::draw()` on the `SpriteBatch`
    /// will do this automatically.
    pub fn flush(&self, ctx: &mut Context) {
        for s in &self.sprites {
            graphics::draw_ex(ctx, &self.image, s.param);
        }
    }

    /// Removes all data from the sprite batch.
    pub fn clear(&mut self) {
        self.sprites.clear()
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
    fn draw_ex(&self, ctx: &mut Context, _param: graphics::DrawParam) -> GameResult<()> {
        self.flush(ctx);
        Ok(())
    }
}
