use super::arc::ArcBuffer;

#[derive(Debug)]
pub struct GrowingBufferArena {
    buffers: Vec<(ArcBuffer, u64)>,
    alignment: u64,
    desc: wgpu::BufferDescriptor<'static>,
}

impl GrowingBufferArena {
    pub fn new(
        device: &wgpu::Device,
        alignment: u64,
        desc: wgpu::BufferDescriptor<'static>,
    ) -> Self {
        GrowingBufferArena {
            buffers: vec![(ArcBuffer::new(device.create_buffer(&desc)), 0)],
            alignment,
            desc,
        }
    }

    pub fn allocate(&mut self, device: &wgpu::Device, size: u64) -> ArenaAllocation {
        let size = align(self.alignment, size);
        assert!(size <= self.desc.size);

        for (_, (buffer, cursor)) in self.buffers.iter_mut().enumerate() {
            if size <= self.desc.size - *cursor {
                let offset = *cursor;
                *cursor += size;
                return ArenaAllocation {
                    buffer: buffer.clone(),
                    offset,
                };
            }
        }

        self.grow(device);
        self.allocate(device, size)
    }

    pub fn free(&mut self) {
        for (_, cursor) in &mut self.buffers {
            *cursor = 0;
        }
    }

    fn grow(&mut self, device: &wgpu::Device) {
        self.buffers
            .push((ArcBuffer::new(device.create_buffer(&self.desc)), 0));
    }
}

#[derive(Debug, Clone)]
pub struct ArenaAllocation {
    pub buffer: ArcBuffer,
    pub offset: u64,
}

fn align(alignment: u64, size: u64) -> u64 {
    (size + alignment - 1) & !(alignment - 1)
}
