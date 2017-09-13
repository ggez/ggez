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
use error;
use gfx;
use gfx::Factory;
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

/// A Batch draws a number of copies of the same object, using a single draw call.
pub trait Batch<I> {
    /// Adds a new instance to the batch.
    ///
    /// Returns a handle with which to modify the instance using `set()`
    fn add(&mut self, param: graphics::DrawParam) -> InstanceIdx;

    /// Alters an instance in the batch to use the given draw params
    fn set(&mut self, handle: InstanceIdx, param: graphics::DrawParam) -> GameResult<()>;

    /// Immediately sends all data in the batch to the graphics card.
    ///
    /// Generally just calling `graphics::draw()` on the batch
    /// will do this automatically.
    fn flush(&self, ctx: &mut Context) -> GameResult<()>;

    /// Removes all data from the instances batch.
    fn clear(&mut self);

    /// Unwraps the inner source of the batch.
    fn into_inner(self) -> I;

    /// Replaces batch source, returning the old one.
    fn set_inner(&mut self, inner: I) -> I;
}

/// A handle to reference an instance inside a Batch
pub type InstanceIdx = usize;

/// A instances batch draws a number of copies of an image using the same draw call.
#[derive(Debug)]
pub struct SpriteBatch {
    image: graphics::Image,
    instances: Vec<graphics::RectInstanceProperties>,
}

impl SpriteBatch {
    /// Creates a new `SpriteBatch`, drawing with the given image.
    pub fn new(image: graphics::Image) -> Self {
        Self {
            image: image,
            instances: vec![],
        }
    }
}

impl Batch<graphics::Image> for SpriteBatch {
    fn add(&mut self, param: graphics::DrawParam) -> InstanceIdx {
        let src_width = param.src.w;
        let src_height = param.src.h;
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        // We also invert the Y scale if our screen coordinates
        // are "upside down", because by default we present the
        // illusion that the screen is addressed in pixels.
        // BUGGO: Which I rather regret now.
        // let invert_y = if gfx.screen_rect.h < 0.0 { 1.0 } else { -1.0 };
        // TODO: Figure out whether implementing this is needed/how to do it cleanly
        let real_scale = graphics::Point {
            x: src_width * param.scale.x * self.image.width as f32,
            y: src_height * param.scale.y * self.image.height as f32,
        };
        let mut new_param = param;
        new_param.scale = real_scale;
        // Not entirely sure why the inversion is necessary, but oh well.
        new_param.offset.x *= -1.0 * param.scale.x;
        new_param.offset.y *= param.scale.y;
        self.instances.push(new_param.into());
        self.instances.len() - 1
    }

    fn set(&mut self, handle: InstanceIdx, param: graphics::DrawParam) -> GameResult<()> {
        if handle < self.instances.len() - 1 {
            let src_width = param.src.w;
            let src_height = param.src.h;
            // We have to mess with the scale to make everything
            // be its-unit-size-in-pixels.
            // We also invert the Y scale if our screen coordinates
            // are "upside down", because by default we present the
            // illusion that the screen is addressed in pixels.
            // BUGGO: Which I rather regret now.
            // let invert_y = if gfx.screen_rect.h < 0.0 { 1.0 } else { -1.0 };
            // TODO: Figure out whether implementing this is needed/how to do it cleanly
            let real_scale = graphics::Point {
                x: src_width * param.scale.x * self.image.width as f32,
                y: src_height * param.scale.y * self.image.height as f32,
            };
            let mut new_param = param;
            new_param.scale = real_scale;
            // Not entirely sure why the inversion is necessary, but oh well.
            new_param.offset.x *= -1.0 * param.scale.x;
            new_param.offset.y *= param.scale.y;
            self.instances[handle] = new_param.into();
            Ok(())
        } else {
            Err(error::GameError::RenderError(String::from("Provided index is out of bounds.")))
        }
    }

    fn flush(&self, ctx: &mut Context) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        if gfx.data.rect_instance_properties.len() < self.instances.len() {
            gfx.data.rect_instance_properties = gfx.factory
                .create_buffer(self.instances.len(),
                               gfx::buffer::Role::Vertex,
                               gfx::memory::Usage::Dynamic,
                               gfx::TRANSFER_DST)?;
        }
        gfx.encoder
            .update_buffer(&gfx.data.rect_instance_properties, &self.instances[..], 0)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.instances.clear();
    }

    fn into_inner(self) -> graphics::Image {
        self.image
    }

    fn set_inner(&mut self, image: graphics::Image) -> graphics::Image {
        use std::mem;
        mem::replace(&mut self.image, image)
    }
}

impl graphics::Drawable for SpriteBatch {
    fn draw_ex(&self, ctx: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        self.flush(ctx)?;
        let gfx = &mut ctx.gfx_context;
        let sampler = gfx.samplers
            .get_or_insert(self.image.sampler_info, gfx.factory.as_mut());
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        gfx.data.tex = (self.image.texture.clone(), sampler);
        let mut slice = gfx.quad_slice.clone();
        slice.instances = Some((self.instances.len() as u32, 0));
        gfx.push_transform(param.into());
        gfx.update_transform()?;
        gfx.encoder.draw(&slice, &gfx.pso, &gfx.data);
        gfx.pop_transform();
        gfx.update_transform()?;
        Ok(())
    }
}

/// A instances batch draws a number of copies of an image using the same draw call.
#[derive(Debug)]
pub struct MeshBatch {
    mesh: graphics::Mesh,
    instances: Vec<graphics::RectInstanceProperties>,
}

impl MeshBatch {
    /// Creates a new `MeshBatch`, drawing with the given mesh.
    pub fn new(mesh: graphics::Mesh) -> Self {
        Self {
            mesh,
            instances: vec![]
        }
    }

    /// Creates a new `MeshBatch` drawing a rect of the Context's point size at each point
    pub fn from_points(ctx: &mut Context, points: &[graphics::Point]) -> GameResult<Self> {
        let instances = points.into_iter().map(|p| {
            graphics::DrawParam {
                dest: *p,
                .. Default::default()
            }.into()
        }).collect::<Vec<graphics::RectInstanceProperties>>();
        let pt_size = ctx.gfx_context.point_size;
        let hw = pt_size / 2.0;
        let verts = [
            [-hw, -hw].into(),
            [hw, -hw].into(),
            [hw, hw].into(),
            [-hw, hw].into(),
        ];
        let w = ctx.gfx_context.line_width;
        let mesh = graphics::Mesh::new_polygon(ctx, graphics::DrawMode::Fill, &verts, w)?;
        Ok(Self {
            mesh,
            instances
        })
    }

    /// Creates a new `MeshBatch` from the given rectangle and draw mode.
    pub fn from_rect(
        ctx: &mut Context,
        mode: graphics::DrawMode,
        rect: graphics::Rect
    ) -> GameResult<Self> {
        let x = rect.x;
        let y = rect.y;
        let w = rect.w;
        let h = rect.h;
        let x1 = x - (w / 2.0);
        let x2 = x + (w / 2.0);
        let y1 = y - (h / 2.0);
        let y2 = y + (h / 2.0);
        let pts = [[x1, y1].into(),
                [x2, y1].into(),
                [x2, y2].into(),
                [x1, y2].into()];
        let w = ctx.gfx_context.line_width;
        let mesh = graphics::Mesh::new_polygon(ctx, mode, &pts, w)?;
        Ok(Self {
            mesh,
            instances: vec![]
        })
    }

    /// Creates a new `MeshBatch` from one or more line segments
    pub fn from_line(ctx: &mut Context, points: &[graphics::Point]) -> GameResult<Self> {
        let w = ctx.gfx_context.line_width;
        let mesh = graphics::Mesh::new_line(ctx, points, w)?;
        Ok(Self {
            mesh,
            instances: vec![]
        })
    }

    /// Creates a new `MeshBatch` from the given polygon and `DrawMode`
    pub fn from_polygon(ctx: &mut Context, mode: graphics::DrawMode, verticies: &[graphics::Point]) -> GameResult<Self> {
        let w = ctx.gfx_context.line_width;
        let mesh = graphics::Mesh::new_polygon(ctx, mode, verticies, w)?;
        Ok(Self {
            mesh,
            instances: vec![]
        })
    }

    /// Creates a new `MeshBatch` from an ellipse built from the given parameters
    pub fn from_ellipse(
        ctx: &mut Context,
        mode: graphics::DrawMode,
        point: graphics::Point,
        radius1: f32,
        radius2: f32,
        tolerance: f32
    ) -> GameResult<(Self)> {
        let mesh = graphics::Mesh::new_ellipse(ctx, mode, point, radius1, radius2, tolerance)?;
        Ok(Self {
            mesh,
            instances: vec![]
        })
    }

    /// Creates a new `MeshBatch` from a circle built from the given parameters
    pub fn from_circle(
        ctx: &mut Context,
        mode: graphics::DrawMode,
        point: graphics::Point,
        radius: f32,
        tolerance: f32
    ) -> GameResult<(Self)> {
        let mesh = graphics::Mesh::new_circle(ctx, mode, point, radius, tolerance)?;
        Ok(Self {
            mesh,
            instances: vec![]
        })
    }
}

impl Batch<graphics::Mesh> for MeshBatch {
    fn add(&mut self, param: graphics::DrawParam) -> InstanceIdx {
        self.instances.push(param.into());
        self.instances.len() - 1
    }

    fn set(&mut self, handle: InstanceIdx, param: graphics::DrawParam) -> GameResult<()> {
        if handle < self.instances.len() - 1 {
            self.instances[handle] = param.into();
            Ok(())
        } else {
            Err(error::GameError::RenderError(String::from("Provided index is out of bounds.")))
        }
    }

    fn flush(&self, ctx: &mut Context) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        if gfx.data.rect_instance_properties.len() < self.instances.len() {
            gfx.data.rect_instance_properties = gfx.factory
                .create_buffer(self.instances.len(),
                               gfx::buffer::Role::Vertex,
                               gfx::memory::Usage::Dynamic,
                               gfx::TRANSFER_DST)?;
        }
        gfx.encoder
            .update_buffer(&gfx.data.rect_instance_properties, &self.instances[..], 0)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.instances.clear();
    }

    fn into_inner(self) -> graphics::Mesh {
        self.mesh
    }

    fn set_inner(&mut self, mesh: graphics::Mesh) -> graphics::Mesh {
        use std::mem;
        mem::replace(&mut self.mesh, mesh)
    }
}

impl graphics::Drawable for MeshBatch {
    fn draw_ex(&self, ctx: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        self.flush(ctx)?;
        let gfx = &mut ctx.gfx_context;
        gfx.data.vbuf = self.mesh.buffer.clone();
        gfx.data.tex.0 = gfx.white_image.texture.clone();
        let mut slice = self.mesh.slice.clone();
        slice.instances = Some((self.instances.len() as u32, 0));
        gfx.push_transform(param.into());
        gfx.update_transform()?;
        gfx.encoder.draw(&slice, &gfx.pso, &gfx.data);
        gfx.pop_transform();
        gfx.update_transform()?;
        Ok(())
    }
}