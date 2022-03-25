//!

use crate::{GameError, GameResult};

use super::{
    context::GraphicsContext,
    draw::{DrawParam, DrawUniforms},
    gpu::arc::ArcBuffer,
    Image, WgpuContext,
};
use crevice::std140::AsStd140;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub(crate) struct InstanceArrayInner {
    pub(crate) buffer: ArcBuffer,
    pub(crate) indices: ArcBuffer,
    dirty: bool,
    capacity: u32,
    len: u32,
}

/// Array of instances for fast rendering of many meshes.
///
/// Traditionally known as a "batch".
#[derive(Debug, Clone)]
pub struct InstanceArray {
    pub(crate) inner: Arc<RwLock<InstanceArrayInner>>,
    pub(crate) image: Image,
    pub(crate) ordered: bool,
    instances: Vec<DrawParam>,
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

        let instances = Vec::with_capacity(capacity as usize);

        InstanceArray {
            inner: Arc::new(RwLock::new(InstanceArrayInner {
                buffer,
                indices,
                dirty: false,
                capacity,
                len: 0,
            })),
            image,
            ordered,
            instances,
        }
    }

    /// Resets all the instance data to a set of `DrawParam`.
    pub fn set(&mut self, instances: impl IntoIterator<Item = DrawParam>) -> GameResult {
        let mut inner = self.inner.write().map_err(|_| GameError::LockError)?;
        inner.dirty = true;
        self.instances = instances.into_iter().collect();
        Ok(())
    }

    /// Pushes a new instance onto the end.
    pub fn push(&mut self, instance: DrawParam) -> GameResult {
        let mut inner = self.inner.write().map_err(|_| GameError::LockError)?;
        inner.dirty = true;
        self.instances.push(instance);
        Ok(())
    }

    /// Updates an existing instance at a given index, and returns the previous value at the index.
    pub fn update(&mut self, index: u32, instance: DrawParam) -> GameResult<Option<DrawParam>> {
        let mut inner = self.inner.write().map_err(|_| GameError::LockError)?;
        inner.dirty = true;
        if let Some(r) = self.instances.get_mut(index as usize) {
            Ok(Some(std::mem::replace(r, instance)))
        } else {
            Ok(None)
        }
    }

    /// Returns an immutable reference to all the instance data.
    #[inline]
    pub fn instances(&self) -> &[DrawParam] {
        &self.instances
    }

    /// Returns a mutable reference to all the instance data.
    #[inline]
    pub fn instances_mut(&mut self) -> GameResult<&mut [DrawParam]> {
        let mut inner = self.inner.write().map_err(|_| GameError::LockError)?;
        inner.dirty = true;
        Ok(&mut self.instances)
    }

    /// Clears all instance data.
    pub fn clear(&mut self) -> GameResult {
        let mut inner = self.inner.write().map_err(|_| GameError::LockError)?;
        inner.len = 0;
        self.instances.clear();
        Ok(())
    }

    /// Returns whether the instance data has been changed without [`InstanceArray::flush()`] being called.
    #[inline]
    pub fn is_dirty(&self) -> GameResult<bool> {
        let inner = self.inner.read().map_err(|_| GameError::LockError)?;
        Ok(inner.dirty)
    }

    /// Uploads the instance data to the GPU.
    ///
    /// You do not usually need to call this yourself as it is done automatically during drawing.
    pub fn flush(&mut self, gfx: &GraphicsContext) -> GameResult {
        self.flush_wgpu(&gfx.wgpu)
    }

    #[allow(unsafe_code)]
    pub(crate) fn flush_wgpu(&mut self, wgpu: &WgpuContext) -> GameResult {
        let mut inner = self.inner.write().map_err(|_| GameError::LockError)?;
        if !inner.dirty {
            return Ok(());
        } else {
            inner.dirty = false;
        }

        let mut layers = BTreeMap::<_, Vec<_>>::new();
        let instances = self
            .instances
            .iter()
            .enumerate()
            .map(|(i, param)| {
                if self.ordered {
                    layers.entry(param.z).or_default().push(i as u32);
                }
                DrawUniforms::from_param(
                    *param,
                    [self.image.width() as f32, self.image.height() as f32].into(),
                )
                .as_std140()
            })
            .collect::<Vec<_>>();

        let len = instances.len() as u32;
        if len > inner.capacity {
            *inner = InstanceArray::new_wgpu(wgpu, self.image.clone(), len, self.ordered)
                .inner
                .read()
                .unwrap()
                .clone();
            inner.dirty = false;
        }

        inner.len = instances.len() as u32;
        wgpu.queue.write_buffer(&inner.buffer, 0, unsafe {
            std::slice::from_raw_parts(
                instances.as_ptr() as *const u8,
                instances.len() * DrawUniforms::std140_size_static(),
            )
        });

        if self.ordered {
            let indices = layers.into_iter().flat_map(|(_, x)| x).collect::<Vec<_>>();
            wgpu.queue.write_buffer(&inner.indices, 0, unsafe {
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
    pub fn resize(&mut self, gfx: &GraphicsContext, new_capacity: u32) -> GameResult {
        let mut inner = self.inner.write().map_err(|_| GameError::LockError)?;
        *inner = InstanceArray::new(gfx, self.image.clone(), new_capacity, self.ordered)
            .inner
            .read()
            .unwrap()
            .clone();
        Ok(())
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
    pub fn capacity(&self) -> GameResult<usize> {
        let inner = self.inner.read().map_err(|_| GameError::LockError)?;
        Ok(inner.capacity as usize)
    }

    #[inline]
    pub(crate) fn len(&self) -> GameResult<usize> {
        let inner = self.inner.read().map_err(|_| GameError::LockError)?;
        Ok(inner.len as usize)
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> GameResult<bool> {
        let inner = self.inner.read().map_err(|_| GameError::LockError)?;
        Ok(inner.len == 0)
    }
}
