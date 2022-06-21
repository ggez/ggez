use crate::{
    context::{Has, HasMut},
    GameError, GameResult,
};

use super::{
    context::GraphicsContext,
    draw::{DrawParam, DrawUniforms, Std140DrawUniforms},
    gpu::arc::ArcBuffer,
    internal_canvas::InstanceArrayView,
    transform_rect, Canvas, Draw, Drawable, Image, Mesh, Rect, WgpuContext,
};
use crevice::std140::AsStd140;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering::SeqCst},
        Mutex,
    },
};

/// Array of instances for fast rendering of many meshes.
///
/// Traditionally known as a "batch".
#[derive(Debug)]
pub struct InstanceArray {
    pub(crate) buffer: Mutex<ArcBuffer>,
    pub(crate) indices: Mutex<ArcBuffer>,
    pub(crate) image: Image,
    pub(crate) ordered: bool,
    dirty: AtomicBool,
    capacity: AtomicU32,
    uniforms: Vec<Std140DrawUniforms>,
    params: Vec<DrawParam>,
}

impl InstanceArray {
    /// Creates a new [InstanceArray] capable of storing up to n-`capacity` instances
    /// (this can be changed and is resized automatically when needed).
    ///
    /// If `image` is `None`, a 1x1 white image will be used which can be used to draw solid rectangles.
    ///
    /// `ordered` controls whether the z-order of instances is simply defined by their indices
    /// (unordered), or by the actual order of their [z-values] (ordered). Intuitively, creating such
    /// order by comparing the [z-values] comes at a slight performance cost.
    ///
    /// [z-values]:crate::graphics::ZIndex
    pub fn new(
        gfx: &impl Has<GraphicsContext>,
        image: impl Into<Option<Image>>,
        capacity: u32,
        ordered: bool,
    ) -> Self {
        let gfx = gfx.retrieve();
        InstanceArray::new_wgpu(
            &gfx.wgpu,
            image.into().unwrap_or_else(|| gfx.white_image.clone()),
            capacity,
            ordered,
        )
    }

    fn new_wgpu(wgpu: &WgpuContext, image: Image, capacity: u32, ordered: bool) -> Self {
        assert!(capacity > 0);

        let buffer = ArcBuffer::new(wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: DrawUniforms::std140_size_static() as u64 * capacity as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        let indices = ArcBuffer::new(wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: if ordered {
                std::mem::size_of::<u32>() as u64 * capacity as u64
            } else {
                4 // min for layout
            },
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let uniforms = Vec::with_capacity(capacity as usize);
        let params = Vec::with_capacity(capacity as usize);

        InstanceArray {
            buffer: Mutex::new(buffer),
            indices: Mutex::new(indices),
            image,
            ordered,
            dirty: AtomicBool::new(false),
            capacity: AtomicU32::new(capacity),
            uniforms,
            params,
        }
    }

    /// Resets all the instance data to a set of `DrawParam`.
    pub fn set(&mut self, instances: impl IntoIterator<Item = DrawParam>) {
        self.dirty.store(true, SeqCst);
        self.params.clear();
        self.params.extend(instances);
        self.uniforms.clear();
        self.uniforms.extend(self.params.iter().map(|x| {
            DrawUniforms::from_param(
                x,
                [self.image.width() as f32, self.image.height() as f32].into(),
            )
            .as_std140()
        }));
    }

    /// Pushes a new instance onto the end.
    pub fn push(&mut self, instance: DrawParam) {
        self.dirty.store(true, SeqCst);
        self.uniforms.push(
            DrawUniforms::from_param(
                &instance,
                [self.image.width() as f32, self.image.height() as f32].into(),
            )
            .as_std140(),
        );
        self.params.push(instance);
    }

    /// Updates an existing instance at a given index, if it is valid.
    pub fn update(&mut self, index: u32, instance: DrawParam) {
        if let Some((uniform, param)) = self
            .uniforms
            .get_mut(index as usize)
            .and_then(|x| Some((x, self.params.get_mut(index as usize)?)))
        {
            self.dirty.store(true, SeqCst);
            *uniform = DrawUniforms::from_param(
                &instance,
                [self.image.width() as f32, self.image.height() as f32].into(),
            )
            .as_std140();
            *param = instance;
        }
    }

    /// Clears all instance data.
    pub fn clear(&mut self) {
        // don't need to set dirty here
        self.uniforms.clear();
        self.params.clear();
    }

    /// Returns whether the instance data has been changed without being flushed (i.e., uploaded to the GPU).
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(SeqCst)
    }

    /// Returns an immutable slice of all the instance data in this [`InstanceArray`].
    #[inline]
    pub fn instances(&self) -> &[DrawParam] {
        &self.params
    }

    #[allow(unsafe_code)]
    pub(crate) fn flush_wgpu(&self, wgpu: &WgpuContext) -> GameResult<()> {
        if !self.dirty.load(SeqCst) {
            return Ok(());
        } else {
            self.dirty.store(false, SeqCst);
        }

        let len = self.uniforms.len() as u32;
        if len > self.capacity.load(SeqCst) {
            let mut resized = InstanceArray::new_wgpu(wgpu, self.image.clone(), len, self.ordered);
            *self.buffer.lock().map_err(|_| GameError::LockError)? =
                resized.buffer.get_mut().unwrap().clone();
            *self.indices.lock().map_err(|_| GameError::LockError)? =
                resized.indices.get_mut().unwrap().clone();
            self.capacity.store(len, SeqCst);
        }

        wgpu.queue
            .write_buffer(&self.buffer.lock().unwrap(), 0, unsafe {
                std::slice::from_raw_parts(
                    self.uniforms.as_ptr() as *const u8,
                    self.uniforms.len() * DrawUniforms::std140_size_static(),
                )
            });

        if self.ordered {
            let mut layers = BTreeMap::<_, Vec<_>>::new();
            for (i, param) in self.params.iter().enumerate() {
                layers.entry(param.z).or_default().push(i);
            }
            let indices = layers.into_values().flatten().collect::<Vec<_>>();
            wgpu.queue
                .write_buffer(&self.indices.lock().unwrap(), 0, unsafe {
                    std::slice::from_raw_parts(
                        indices.as_ptr() as *const u8,
                        indices.len() * std::mem::size_of::<u32>(),
                    )
                });
        }

        Ok(())
    }

    /// Changes the capacity of this `InstanceArray` while preserving instances.
    ///
    /// If `new_capacity` is less than the `len`, the instances will be truncated.
    pub fn resize(&mut self, gfx: &impl Has<GraphicsContext>, new_capacity: u32) {
        let resized = InstanceArray::new(gfx, self.image.clone(), new_capacity, self.ordered);
        self.buffer = resized.buffer;
        self.indices = resized.indices;
        self.capacity.store(new_capacity, SeqCst);
        self.dirty.store(true, SeqCst);
        self.uniforms.truncate(new_capacity as usize);
        self.params.truncate(new_capacity as usize);
        self.uniforms
            .reserve((new_capacity as usize).min(self.uniforms.len()) - self.uniforms.len());
        self.params
            .reserve((new_capacity as usize).min(self.params.len()) - self.params.len());
    }

    /// Returns this `InstanceArray`'s associated `image`.
    #[inline]
    pub fn image(&self) -> Image {
        self.image.clone()
    }

    /// Returns the number of instances this [InstanceArray] is capable of holding.
    /// This number was specified when creating the [InstanceArray], or if the [InstanceArray]
    /// was automatically resized, the greatest length of instances.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.load(SeqCst) as usize
    }

    /// This is equivalent to `<InstanceArray as Drawable>::dimensions()` (see [`Drawable::dimensions()`]), but with a mesh taken into account.
    ///
    /// Essentially, consider `<InstanceArray as Drawable>::dimensions()` to be the bounds when the [`InstanceArray`] is drawn with `canvas.draw()`,
    /// and consider [`InstanceArray::dimensions_meshed()`] to be the bounds when the [`InstanceArray`] is drawn with `canvas.draw_instanced_mesh()`.
    pub fn dimensions_meshed(
        &self,
        gfx: &mut impl HasMut<GraphicsContext>,
        mesh: &Mesh,
    ) -> Option<Rect> {
        if self.params.is_empty() {
            return None;
        }
        let dimensions = mesh.dimensions(gfx)?;
        self.params
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
}

impl Drawable for InstanceArray {
    fn draw(&self, canvas: &mut Canvas, param: impl Into<DrawParam>) {
        self.flush_wgpu(&canvas.wgpu).unwrap();
        canvas.push_draw(
            Draw::MeshInstances {
                mesh: canvas.default_resources().mesh.clone(),
                instances: InstanceArrayView::from_instances(self).unwrap(),
            },
            param.into(),
        );
    }

    fn dimensions(&self, gfx: &mut impl HasMut<GraphicsContext>) -> Option<Rect> {
        let gfx = gfx.retrieve_mut();
        if self.params.is_empty() {
            return None;
        }
        let dimensions = self.image.dimensions(gfx)?;
        self.params
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
}
