use super::arc::{ArcBindGroup, ArcBindGroupLayout, ArcBuffer, ArcSampler, ArcTextureView};
use std::{collections::HashMap, num::NonZeroU64};

pub struct BindGroupLayoutBuilder {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        BindGroupLayoutBuilder { entries: vec![] }
    }

    pub fn buffer(
        mut self,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BufferBindingType,
        has_dynamic_offset: bool,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as _,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset,
                min_binding_size: None,
            },
            count: None,
        });
        self
    }

    pub fn image(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as _,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
        self
    }

    pub fn sampler(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as _,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        self
    }

    pub fn create(self, device: &wgpu::Device, cache: &mut BindGroupCache) -> ArcBindGroupLayout {
        let entries = self.entries.clone();
        cache
            .layouts
            .entry(self.entries)
            .or_insert_with(|| {
                ArcBindGroupLayout::new(device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &entries,
                    },
                ))
            })
            .clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BindGroupEntryKey {
    Buffer {
        id: u64,
        offset: u64,
        size: Option<u64>,
    },
    Image {
        id: u64,
    },
    Sampler {
        id: u64,
    },
}

pub struct BindGroupBuilder<'a> {
    layout: BindGroupLayoutBuilder,
    entries: Vec<wgpu::BindGroupEntry<'a>>,
    key: Vec<BindGroupEntryKey>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new() -> Self {
        BindGroupBuilder {
            layout: BindGroupLayoutBuilder::new(),
            entries: vec![],
            key: vec![],
        }
    }

    pub fn buffer(
        mut self,
        buffer: &'a ArcBuffer,
        offset: u64,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BufferBindingType,
        has_dynamic_offset: bool,
        size: Option<u64>,
    ) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as _,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: buffer.as_ref(),
                offset,
                size: size.map(|x| NonZeroU64::new(x).unwrap()),
            }),
        });

        self.key.push(BindGroupEntryKey::Buffer {
            id: buffer.id(),
            offset,
            size,
        });

        BindGroupBuilder {
            layout: self.layout.buffer(visibility, ty, has_dynamic_offset),
            entries: self.entries,
            key: self.key,
        }
    }

    pub fn image(mut self, view: &'a ArcTextureView, visibility: wgpu::ShaderStages) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as _,
            resource: wgpu::BindingResource::TextureView(view.as_ref()),
        });

        self.key.push(BindGroupEntryKey::Image { id: view.id() });

        BindGroupBuilder {
            layout: self.layout.image(visibility),
            entries: self.entries,
            key: self.key,
        }
    }

    pub fn sampler(mut self, sampler: &'a ArcSampler, visibility: wgpu::ShaderStages) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as _,
            resource: wgpu::BindingResource::Sampler(sampler.as_ref()),
        });

        self.key
            .push(BindGroupEntryKey::Sampler { id: sampler.id() });

        BindGroupBuilder {
            layout: self.layout.sampler(visibility),
            entries: self.entries,
            key: self.key,
        }
    }

    pub fn create(
        self,
        device: &wgpu::Device,
        cache: &mut BindGroupCache,
    ) -> (ArcBindGroup, ArcBindGroupLayout) {
        let layout = self.layout.create(device, cache);

        let group = cache
            .groups
            .entry(self.key)
            .or_insert_with(|| {
                ArcBindGroup::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: layout.as_ref(),
                    entries: &self.entries,
                }))
            })
            .clone();

        (group, layout)
    }
}

#[derive(Debug)]
pub struct BindGroupCache {
    layouts: HashMap<Vec<wgpu::BindGroupLayoutEntry>, ArcBindGroupLayout>,
    groups: HashMap<Vec<BindGroupEntryKey>, ArcBindGroup>,
}

impl BindGroupCache {
    pub fn new() -> Self {
        BindGroupCache {
            layouts: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}
