use std::fmt;
use std::path;
use std::convert::From;
use std::collections::HashMap;
use std::io::Read;
use std::u16;

use sdl2;
use image;
use gfx;
use gfx::texture;
use gfx::traits::Device;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_sdl;
use gfx::Factory;


use context::Context;
use graphics;
use GameError;
use GameResult;

// Owning the given Image is inconvenient because we might want, say,
// the same Rc<Image> shared among many SpriteBatch'es.
//
// But draw() doesn't
//
// The right way to fix this might be to have another object that implements
// Drawable that is created from a SpriteBatch and an image reference.
// BoundSpriteBatch or something.  That *feels* like the right way to go...
//
// Or just not implement Drawable and provide your own drawing functions, though
// that's squirrelly.
//
// Oh, or maybe make it take a Cow<Image> ?  that might work.

/// A SpriteBatch draws a number of copies of the same image, using a single draw call.
pub struct SpriteBatch {
    image: graphics::Image,
    sprites: Vec<graphics::DrawParam>,
}

pub type SpriteIdx = usize;

impl SpriteBatch {
    /// Creates a new `SpriteBatch`, drawing with the given image.
    pub fn new(image: graphics::Image) -> Self {
        Self {
            image: image,
            sprites: vec![],
        }
    }
    
    /// Adds a new sprite to the sprite batch.
    ///
    /// Returns a handle with which to modify the sprite using `set()`
    pub fn add(&mut self, param: graphics::DrawParam) -> SpriteIdx {
        self.sprites.push(param);
        self.sprites.len() - 1
    }

    /// Alters a sprite in the batch to use the given draw params.
    pub fn set(&mut self, handle: SpriteIdx, param: graphics::DrawParam) {
        self.sprites[handle] = param;
    }

    /// Immediately sends all data in the batch to the graphics card.
    ///
    /// Generally just calling `graphics::draw()` on the `SpriteBatch`
    /// will do this automatically.
    pub fn flush(&self, ctx: &mut Context) {
        for s in &self.sprites {
            graphics::draw_ex(ctx, &self.image, *s);
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
    fn draw_ex(&self, ctx: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        self.flush(ctx);
        Ok(())
    }
}
