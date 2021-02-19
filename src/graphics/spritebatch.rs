//! A [`SpriteBatch`](struct.SpriteBatch.html) is a way to
//! efficiently draw a large number of copies of the same image, or part
//! of the same image.  It's useful for implementing tiled maps,
//! spritesheets, particles, and other such things.
//!
//! Essentially this uses a technique called "instancing" to queue up
//! a large amount of location/position data in a buffer, then feed it
//! to the graphics card all in one go.
//!
//! Also it's super slow in `rustc`'s default debug mode, because
//! `rustc` adds a lot of checking to the vector accesses and math.
//! If you use it, it's recommended to crank up the `opt-level` for
//! debug mode in your game's `Cargo.toml`.

use crate::context::Context;
use crate::error;
use crate::error::GameResult;
use crate::graphics::shader::BlendMode;
use crate::graphics::types::FilterMode;
use crate::graphics::{self, transform_rect, BackendSpec, DrawParam, Matrix4, Rect, Transform};
use gfx::Factory;

/// A `SpriteBatch` draws a number of copies of the same image, using a single draw call.
///
/// This is generally faster than drawing the same sprite with many
/// invocations of [`draw()`](../fn.draw.html), though it has a bit of
/// overhead to set up the batch.  This overhead makes it run very
/// slowly in `debug` mode because it spends a lot of time on array
/// bounds checking and un-optimized math; you need to build with
/// optimizations enabled to really get the speed boost.
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteBatch {
    image: graphics::Image,
    sprites: Vec<graphics::DrawParam>,
    blend_mode: Option<BlendMode>,
}

/// An index of a particular sprite in a `SpriteBatch`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteIdx(pub usize);

impl SpriteBatch {
    /// Creates a new `SpriteBatch`, drawing with the given image.
    ///
    /// Takes ownership of the `Image`, but cloning an `Image` is
    /// cheap since they have an internal `Arc` containing the actual
    /// image data.
    pub fn new(image: graphics::Image) -> Self {
        Self {
            image,
            sprites: vec![],
            blend_mode: None,
        }
    }

    /// Adds a new sprite to the sprite batch.
    ///
    /// Returns a handle with which to modify the sprite using
    /// [`set()`](#method.set)
    pub fn add<P>(&mut self, param: P) -> SpriteIdx
    where
        P: Into<graphics::DrawParam>,
    {
        self.sprites.push(param.into());
        SpriteIdx(self.sprites.len() - 1)
    }

    /// Alters a sprite in the batch to use the given draw params
    pub fn set<P>(&mut self, handle: SpriteIdx, param: P) -> GameResult
    where
        P: Into<graphics::DrawParam>,
    {
        if handle.0 < self.sprites.len() {
            self.sprites[handle.0] = param.into();
            Ok(())
        } else {
            Err(error::GameError::RenderError(String::from(
                "Provided index is out of bounds.",
            )))
        }
    }

    /// Immediately sends all data in the batch to the graphics card.
    ///
    /// Generally just calling [`graphics::draw()`](../fn.draw.html) on the `SpriteBatch`
    /// will do this automatically.
    fn flush(&self, ctx: &mut Context, image: &graphics::Image) -> GameResult {
        // This is a little awkward but this is the right place
        // to do whatever transformations need to happen to DrawParam's.
        // We have a Context, and *everything* must pass through this
        // function to be drawn, so.
        // Though we do awkwardly have to allocate a new vector.
        // ...though upon benchmarking, the actual allocation is basically nothing,
        // the cost in debug mode is alllll math.
        let new_sprites = self
            .sprites
            .iter()
            .map(|param| {
                // Copy old params
                let src_width = param.src.w;
                let src_height = param.src.h;
                let new_param = match param.trans {
                    Transform::Values { scale, .. } => {
                        let new_scale = mint::Vector2 {
                            x: scale.x * src_width * f32::from(image.width),
                            y: scale.y * src_height * f32::from(image.height),
                        };
                        param.scale(new_scale)
                    }
                    Transform::Matrix(_) => *param,
                };
                let primitive_param = new_param;
                primitive_param.to_instance_properties(ctx.gfx_context.is_srgb())
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

    /// Unwraps and returns the contained `Image`
    pub fn into_inner(self) -> graphics::Image {
        self.image
    }

    /// Replaces the contained `Image`, returning the old one.
    pub fn set_image(&mut self, image: graphics::Image) -> graphics::Image {
        use std::mem;
        mem::replace(&mut self.image, image)
    }

    /// Get the filter mode for the SpriteBatch.
    pub fn filter(&self) -> FilterMode {
        self.image.filter()
    }

    /// Set the filter mode for the SpriteBatch.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.image.set_filter(mode);
    }
}

impl graphics::Drawable for SpriteBatch {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        // scale the offset according to the dimensions of the spritebatch
        // but only if there is an offset (it's too expensive to calculate the dimensions to always to this)
        let mut new_param = param;
        if let Transform::Values { offset, .. } = param.trans {
            if offset != [0.0, 0.0].into() {
                if let Some(dim) = self.dimensions(ctx) {
                    let new_offset = mint::Vector2 {
                        x: offset.x * dim.w + dim.x,
                        y: offset.y * dim.h + dim.y,
                    };
                    new_param = param.offset(new_offset);
                }
            }
        }
        // Awkwardly we must update values on all sprites and such.
        // Also awkwardly we have this chain of colors with differing priorities.
        self.flush(ctx, &self.image)?;
        let gfx = &mut ctx.gfx_context;
        let sampler = gfx
            .samplers
            .get_or_insert(self.image.sampler_info, gfx.factory.as_mut());
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        let typed_thingy = gfx
            .backend_spec
            .raw_to_typed_shader_resource(self.image.texture.clone());
        gfx.data.tex = (typed_thingy, sampler);

        let mut slice = gfx.quad_slice.clone();
        slice.instances = Some((self.sprites.len() as u32, 0));
        // It's a little silly converting this mint matrix back to a glam
        // one but it's simpler than the alternative.
        let m = Matrix4::from(new_param.trans.to_bare_matrix());
        gfx.set_global_mvp(m)?;
        let previous_mode: Option<BlendMode> = if let Some(mode) = self.blend_mode {
            let current_mode = gfx.blend_mode();
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
        gfx.set_global_mvp(graphics::Matrix4::identity())?;
        Ok(())
    }
    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        if self.sprites.is_empty() {
            return None;
        }
        let dimensions = self.image.dimensions();
        self.sprites
            .iter()
            .map(|&param| transform_rect(dimensions, param))
            .fold(None, |acc: Option<Rect>, rect| {
                Some(if let Some(acc) = acc {
                    acc.combine_with(rect)
                } else {
                    rect
                })
            })
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }
    fn blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
