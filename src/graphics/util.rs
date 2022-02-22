use super::context::GraphicsContext;

pub struct BufferInitDeferDescriptor<'a> {
    pub data: &'a [u8],
    pub usage: wgpu::BufferUsages,
}

pub fn create_buffer_init_defer(
    gfx: &GraphicsContext,
    desc: &BufferInitDeferDescriptor,
) -> wgpu::Buffer {
    let buf = gfx.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: desc.data.len() as u64,
        usage: desc.usage,
        mapped_at_creation: false,
    });
    gfx.queue.write_buffer(&buf, 0, desc.data);
    buf
}

#[allow(unsafe_code)]
pub unsafe fn slice_to_bytes<T>(slice: &[T]) -> &[u8] {
    std::slice::from_raw_parts(
        slice.as_ptr() as *const u8,
        slice.len() * std::mem::size_of::<T>(),
    )
}

#[allow(unsafe_code)]
pub unsafe fn val_to_bytes<T>(val: &T) -> &[u8] {
    std::slice::from_raw_parts(val as *const T as *const u8, std::mem::size_of::<T>())
}
