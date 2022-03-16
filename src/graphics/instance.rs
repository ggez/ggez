//!

use super::{
    context::GraphicsContext,
    draw::{DrawParam, DrawUniforms},
    gpu::arc::ArcBuffer,
    Image,
};
use crevice::std140::{AsStd140, Std140};

/// Array of instances for fast rendering of many meshes.
///
/// Traditionally known as a "batch".
#[derive(Debug)]
pub struct InstanceArray {
    pub(crate) buffer: ArcBuffer,
    pub(crate) image: Image,
    capacity: u32,
    len: u32,
}

impl InstanceArray {
    /// Creates a new [InstanceArray] capable of storing up to n-`capacity` instances (this can be changed and is resized automatically when needed).
    ///
    /// If `image` is `None`, a 1x1 white image will be used which can be used to draw solid rectangles.
    pub fn new(gfx: &GraphicsContext, image: impl Into<Option<Image>>, capacity: u32) -> Self {
        assert!(capacity > 0);

        let buffer = ArcBuffer::new(gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: DrawUniforms::std140_size_static() as u64 * capacity as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        let image = image
            .into()
            .unwrap_or_else(|| gfx.white_image.clone().unwrap(/* invariant */));

        InstanceArray {
            buffer,
            image,
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
    {
        let instances = instances
            .into_iter()
            .map(|param| {
                DrawUniforms::from_param(
                    param,
                    [self.image.width() as f32, self.image.height() as f32].into(),
                )
                .as_std140()
            })
            .collect::<Vec<_>>();

        let len = instances.len() as u32;
        if len > self.capacity {
            self.resize_impl(gfx, len, false);
        }

        self.len = instances.len() as u32;
        gfx.queue.write_buffer(&self.buffer, 0, unsafe {
            std::slice::from_raw_parts(
                instances.as_ptr() as *const u8,
                instances.len() * DrawUniforms::std140_size_static(),
            )
        });
    }

    /// Pushes a new instance onto the end.
    ///
    /// Prefer `set` where bulk instances needs to be set.
    pub fn push(&mut self, gfx: &GraphicsContext, instance: DrawParam) {
        if self.len == self.capacity {
            self.resize(gfx, self.capacity + self.capacity / 2);
        }

        let instance = DrawUniforms::from_param(
            instance,
            [self.image.width() as f32, self.image.height() as f32].into(),
        );
        gfx.queue.write_buffer(
            &self.buffer,
            self.len as u64 * DrawUniforms::std140_size_static() as u64,
            instance.as_std140().as_bytes(),
        );
        self.len += 1;
    }

    /// Updates an existing instance at a given index.
    pub fn update(&mut self, gfx: &GraphicsContext, index: u32, instance: DrawParam) {
        assert!(index < self.len, "index out of range");

        let instance = DrawUniforms::from_param(
            instance,
            [self.image.width() as f32, self.image.height() as f32].into(),
        );
        gfx.queue.write_buffer(
            &self.buffer,
            index as u64 * DrawUniforms::std140_size_static() as u64,
            instance.as_std140().as_bytes(),
        );
    }

    /// Clears all instance data.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Changes the capacity of this `InstanceArray` while preserving instances.
    ///
    /// If `new_capacity` is less than the `len`, the instances will be truncated.
    pub fn resize(&mut self, gfx: &GraphicsContext, new_capacity: u32) {
        self.resize_impl(gfx, new_capacity, true)
    }

    fn resize_impl(&mut self, gfx: &GraphicsContext, new_capacity: u32, copy: bool) {
        let mut resized = InstanceArray::new(gfx, self.image.clone(), new_capacity);
        resized.len = new_capacity.min(self.len);

        if copy {
            let cmd = {
                let mut cmd = gfx.device.create_command_encoder(&Default::default());
                cmd.copy_buffer_to_buffer(
                    &self.buffer,
                    0,
                    &resized.buffer,
                    0,
                    new_capacity.min(self.len) as u64 * DrawUniforms::std140_size_static() as u64,
                );
                cmd.finish()
            };
            gfx.queue.submit([cmd]);
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

    /// Returns the number of instances.
    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }
}
