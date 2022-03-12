//!

use super::{
    context::GraphicsContext,
    draw::{DrawParam, DrawUniforms},
    gpu::arc::ArcBuffer,
};
use crevice::std430::{AsStd430, Std430};

/// Array of instances for fast rendering.
///
/// Traditionally known as a "batch".
#[derive(Debug)]
pub struct InstanceArray {
    pub(crate) buffer: ArcBuffer,
    capacity: u32,
    len: u32,
}

impl InstanceArray {
    /// Creates a new [InstanceArray] capable of storing up to n-`capacity` instances.
    pub fn new(gfx: &GraphicsContext, capacity: u32) -> Self {
        let buffer = ArcBuffer::new(gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: DrawUniforms::std430_size_static() as u64 * capacity as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        InstanceArray {
            buffer,
            capacity,
            len: 0,
        }
    }

    /// Resets all the instance data to a set of `DrawParam`.
    ///
    /// Prefer this over `push` where possible.
    #[allow(unsafe_code)]
    pub fn set<I>(&mut self, gfx: &GraphicsContext, instances: I)
    where
        I: IntoIterator<Item = DrawParam>,
        I::IntoIter: ExactSizeIterator,
    {
        let instances = instances.into_iter();

        assert!(
            instances.len() <= self.capacity as usize,
            "exceeding instance array capacity"
        );

        let instances = instances
            .map(|param| DrawUniforms::from_param(param, glam::Vec2::ONE.into()))
            .collect::<Vec<_>>();

        self.len = instances.len() as u32;
        gfx.queue.write_buffer(&self.buffer, 0, unsafe {
            std::slice::from_raw_parts(
                instances.as_ptr() as *const u8,
                instances.len() * DrawUniforms::std430_size_static(),
            )
        });
    }

    /// Pushes a new instance onto the end.
    ///
    /// Prefer `set` where bulk instances needs to be set.
    pub fn push(&mut self, gfx: &GraphicsContext, instance: DrawParam) {
        assert!(
            self.len < self.capacity,
            "exceeding instance array capacity"
        );

        let instance = DrawUniforms::from_param(instance, glam::Vec2::ONE.into());
        gfx.queue.write_buffer(
            &self.buffer,
            self.len as u64 * DrawUniforms::std430_size_static() as u64,
            instance.as_std430().as_bytes(),
        );
        self.len += 1;
    }

    /// Updates an existing instance at a given index.
    pub fn update(&mut self, gfx: &GraphicsContext, index: u32, instance: DrawParam) {
        assert!(index < self.len);

        let instance = DrawUniforms::from_param(instance, glam::Vec2::ONE.into());
        gfx.queue.write_buffer(
            &self.buffer,
            index as u64 * DrawUniforms::std430_size_static() as u64,
            instance.as_std430().as_bytes(),
        );
    }

    /// Clears all instance data.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns the number of instances this [InstanceArray] is capable of holding.
    /// This number was specified when creating the [InstanceArray].
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity as usize
    }

    /// Returns the number of instances.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len as usize
    }
}
