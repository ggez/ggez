//!

use super::{
    context::GraphicsContext,
    draw::{DrawParam, DrawUniforms},
    gpu::arc::ArcBuffer,
    Image, WgpuContext,
};
use crevice::std140::AsStd140;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering::SeqCst},
        Arc,
    },
};

/// Array of instances for fast rendering of many meshes.
///
/// Traditionally known as a "batch".
#[derive(Debug, Clone)]
pub struct InstanceArray {
    pub(crate) buffer: ArcBuffer,
    pub(crate) indices: ArcBuffer,
    pub(crate) image: Image,
    dirty: Arc<AtomicBool>,
    instances: Vec<DrawParam>,
    capacity: u32,
    len: Arc<AtomicU32>,
}

impl InstanceArray {
    /// Creates a new [InstanceArray] capable of storing up to n-`capacity` instances (this can be changed and is resized automatically when needed).
    ///
    /// If `image` is `None`, a 1x1 white image will be used which can be used to draw solid rectangles.
    pub fn new(gfx: &GraphicsContext, image: impl Into<Option<Image>>, capacity: u32) -> Self {
        InstanceArray::new_wgpu(
            &gfx.wgpu,
            image.into().unwrap_or_else(|| gfx.white_image.clone()),
            capacity,
        )
    }

    fn new_wgpu(wgpu: &WgpuContext, image: Image, capacity: u32) -> Self {
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
            size: std::mem::size_of::<u32>() as u64 * capacity as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let instances = Vec::with_capacity(capacity as usize);

        InstanceArray {
            buffer,
            indices,
            image,
            dirty: Arc::new(AtomicBool::new(false)),
            instances,
            capacity,
            len: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Resets all the instance data to a set of `DrawParam`.
    pub fn set(&mut self, instances: impl IntoIterator<Item = DrawParam>) {
        self.dirty.store(true, SeqCst);
        self.instances = instances.into_iter().collect();
    }

    /// Pushes a new instance onto the end.
    pub fn push(&mut self, instance: DrawParam) {
        self.dirty.store(true, SeqCst);
        self.instances.push(instance);
    }

    /// Updates an existing instance at a given index, and returns the previous value at the index.
    pub fn update(&mut self, index: u32, instance: DrawParam) -> Option<DrawParam> {
        self.dirty.store(true, SeqCst);
        Some(std::mem::replace(
            self.instances.get_mut(index as usize)?,
            instance,
        ))
    }

    /// Returns an immutable reference to all the instance data.
    #[inline]
    pub fn instances(&self) -> &[DrawParam] {
        &self.instances
    }

    /// Returns a mutable reference to all the instance data.
    #[inline]
    pub fn instances_mut(&mut self) -> &mut [DrawParam] {
        self.dirty.store(true, SeqCst);
        &mut self.instances
    }

    /// Clears all instance data.
    pub fn clear(&mut self) {
        self.len.store(0, SeqCst);
        self.instances.clear();
    }

    /// Returns whether the instance data has been changed without [`InstanceArray::flush()`] being called.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(SeqCst)
    }

    /// Uploads the instance data to the GPU.
    ///
    /// You do not usually need to call this yourself as it is done automatically during drawing.
    pub fn flush(&mut self, gfx: &GraphicsContext) {
        self.flush_wgpu(&gfx.wgpu)
    }

    #[allow(unsafe_code)]
    pub(crate) fn flush_wgpu(&mut self, wgpu: &WgpuContext) {
        if !self.is_dirty() {
            return;
        } else {
            self.dirty.store(false, SeqCst);
        }

        let mut layers = BTreeMap::<_, Vec<_>>::new();
        let instances = self
            .instances
            .iter()
            .enumerate()
            .map(|(i, param)| {
                layers.entry(param.z).or_default().push(i as u32);
                DrawUniforms::from_param(
                    *param,
                    [self.image.width() as f32, self.image.height() as f32].into(),
                )
                .as_std140()
            })
            .collect::<Vec<_>>();

        let len = instances.len() as u32;
        if len > self.capacity {
            self.resize_impl(wgpu, len, false);
        }

        self.len.store(instances.len() as u32, SeqCst);
        wgpu.queue.write_buffer(&self.buffer, 0, unsafe {
            std::slice::from_raw_parts(
                instances.as_ptr() as *const u8,
                instances.len() * DrawUniforms::std140_size_static(),
            )
        });

        let indices = layers.into_iter().flat_map(|(_, x)| x).collect::<Vec<_>>();
        wgpu.queue.write_buffer(&self.indices, 0, unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                indices.len() * std::mem::size_of::<u32>(),
            )
        });
    }

    /// Changes the capacity of this `InstanceArray` while preserving instances.
    ///
    /// If `new_capacity` is less than the `len`, the instances will be truncated.
    pub fn resize(&mut self, gfx: &GraphicsContext, new_capacity: u32) {
        self.resize_impl(&gfx.wgpu, new_capacity, true)
    }

    fn resize_impl(&mut self, wgpu: &WgpuContext, new_capacity: u32, copy: bool) {
        let len = self.len.load(SeqCst);
        let resized = InstanceArray::new_wgpu(wgpu, self.image.clone(), new_capacity);
        resized.len.store(new_capacity.min(len), SeqCst);

        if copy {
            let cmd = {
                let mut cmd = wgpu.device.create_command_encoder(&Default::default());
                cmd.copy_buffer_to_buffer(
                    &self.buffer,
                    0,
                    &resized.buffer,
                    0,
                    new_capacity.min(len) as u64 * DrawUniforms::std140_size_static() as u64,
                );
                cmd.finish()
            };
            wgpu.queue.submit([cmd]);
        }

        *self = resized;
    }

    /// Returns this `InstanceArray`'s associated `image`.
    #[inline]
    pub fn image(&self) -> Image {
        self.image.clone()
    }

    /// Returns the number of instances this [InstanceArray] is capable of holding.
    /// This number was specified when creating the [InstanceArray].
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity as usize
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.len.load(SeqCst) as usize
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len.load(SeqCst) == 0
    }
}
