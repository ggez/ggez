//!

use super::{
    context::GraphicsContext,
    draw::{DrawParam, DrawUniforms, Std140DrawUniforms},
    gpu::arc::ArcBuffer,
    Canvas, Draw, Drawable, Image, Rect, WgpuContext,
};
use crevice::std140::AsStd140;
use std::collections::BTreeMap;

/// Array of instances for fast rendering of many meshes.
///
/// Traditionally known as a "batch".
#[derive(Debug)]
pub struct InstanceArray {
    pub(crate) buffer: ArcBuffer,
    pub(crate) indices: ArcBuffer,
    pub(crate) image: Image,
    pub(crate) ordered: bool,
    dirty: bool,
    capacity: u32,
    uniforms: Vec<Std140DrawUniforms>,
    params: Vec<DrawParam>,
}

impl InstanceArray {
    /// Creates a new [InstanceArray] capable of storing up to n-`capacity` instances (this can be changed and is resized automatically when needed).
    ///
    /// If `image` is `None`, a 1x1 white image will be used which can be used to draw solid rectangles.
    pub fn new(
        gfx: &GraphicsContext,
        image: impl Into<Option<Image>>,
        capacity: u32,
        ordered: bool,
    ) -> Self {
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
            buffer,
            indices,
            image,
            ordered,
            dirty: false,
            capacity,
            uniforms,
            params,
        }
    }

    /// Resets all the instance data to a set of `DrawParam`.
    pub fn set(&mut self, instances: impl IntoIterator<Item = DrawParam>) {
        self.dirty = true;
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
        self.dirty = true;
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
            self.dirty = true;
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

    /// Returns whether the instance data has been changed without [`InstanceArray::flush()`] being called.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Returns an immutable slice of all the instance data in this [`InstanceArray`].
    #[inline]
    pub fn instances(&self) -> &[DrawParam] {
        &self.params
    }

    /// Uploads the instance data to the GPU.
    ///
    /// You do not usually need to call this yourself as it is done automatically during drawing.
    pub fn flush(&mut self, gfx: &GraphicsContext) {
        self.flush_wgpu(&gfx.wgpu)
    }

    #[allow(unsafe_code)]
    pub(crate) fn flush_wgpu(&mut self, wgpu: &WgpuContext) {
        if !self.dirty {
            return;
        } else {
            self.dirty = false;
        }

        let len = self.uniforms.len() as u32;
        if len > self.capacity {
            let resized = InstanceArray::new_wgpu(wgpu, self.image.clone(), len, self.ordered);
            self.buffer = resized.buffer;
            self.indices = resized.indices;
            self.capacity = len;
        }

        wgpu.queue.write_buffer(&self.buffer, 0, unsafe {
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
            wgpu.queue.write_buffer(&self.indices, 0, unsafe {
                std::slice::from_raw_parts(
                    indices.as_ptr() as *const u8,
                    indices.len() * std::mem::size_of::<u32>(),
                )
            });
        }
    }

    /// Changes the capacity of this `InstanceArray` while preserving instances.
    ///
    /// If `new_capacity` is less than the `len`, the instances will be truncated.
    pub fn resize(&mut self, gfx: &GraphicsContext, new_capacity: u32) {
        let resized = InstanceArray::new(gfx, self.image.clone(), new_capacity, self.ordered);
        self.buffer = resized.buffer;
        self.indices = resized.indices;
        self.capacity = new_capacity;
        self.dirty = true;
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
        self.capacity as usize
    }
}

impl<'a> Drawable for &'a mut InstanceArray {
    fn draw(self, canvas: &mut Canvas, param: DrawParam) {
        self.flush_wgpu(&canvas.wgpu);
        canvas.push_draw(
            Draw::MeshInstances {
                mesh: canvas.default_resources().mesh.clone(),
                instances: (&*self).into(),
            },
            param,
        );
    }

    fn dimensions(self, _gfx: &mut GraphicsContext) -> Option<Rect> {
        None
    }
}
