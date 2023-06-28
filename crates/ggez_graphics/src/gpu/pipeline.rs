use super::arc::{ArcBindGroupLayout, ArcPipelineLayout, ArcRenderPipeline, ArcShaderModule};
use std::collections::{hash_map::DefaultHasher, HashMap};

/// Hashable representation of a render pipeline, used as a key in the HashMap cache.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPipelineInfo {
    pub vs: ArcShaderModule,
    pub fs: ArcShaderModule,
    pub vs_entry: String,
    pub fs_entry: String,
    pub samples: u32,
    pub format: wgpu::TextureFormat,
    pub blend: Option<wgpu::BlendState>,
    pub depth: bool,
    pub vertices: bool,
    pub topology: wgpu::PrimitiveTopology,
    pub vertex_layout: wgpu::VertexBufferLayout<'static>,
}

/// Caches both the pipeline *and* the pipeline layout.
#[derive(Debug)]
pub struct PipelineCache {
    pipelines: HashMap<RenderPipelineInfo, ArcRenderPipeline>,
    layouts: HashMap<u64, ArcPipelineLayout>,
}

impl PipelineCache {
    pub fn new() -> Self {
        PipelineCache {
            pipelines: HashMap::new(),
            layouts: HashMap::new(),
        }
    }

    pub fn render_pipeline(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        info: RenderPipelineInfo,
    ) -> ArcRenderPipeline {
        let vertex_buffers = [info.vertex_layout.clone()];

        self.pipelines
            .entry(info.clone())
            .or_insert_with(|| {
                ArcRenderPipeline::new(device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(layout),
                        vertex: wgpu::VertexState {
                            module: &info.vs,
                            entry_point: &info.vs_entry,
                            buffers: if info.vertices { &vertex_buffers } else { &[] },
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: info.topology,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        },
                        depth_stencil: if info.depth {
                            Some(wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: true,
                                depth_compare: wgpu::CompareFunction::Always,
                                stencil: Default::default(),
                                bias: Default::default(),
                            })
                        } else {
                            None
                        },
                        multisample: wgpu::MultisampleState {
                            count: info.samples,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &info.fs,
                            entry_point: &info.fs_entry,
                            targets: &[Some(wgpu::ColorTargetState {
                                format: info.format,
                                blend: info.blend,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    },
                ))
            })
            .clone()
    }

    pub fn layout(
        &mut self,
        device: &wgpu::Device,
        bind_groups: &[ArcBindGroupLayout],
    ) -> ArcPipelineLayout {
        let key = {
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            for bg in bind_groups {
                bg.id().hash(&mut h);
            }
            h.finish()
        };
        self.layouts
            .entry(key)
            .or_insert_with(|| {
                ArcPipelineLayout::new(
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &bind_groups
                            .iter()
                            .map(|bg| bg.handle.as_ref())
                            .collect::<Vec<_>>(),
                        push_constant_ranges: &[],
                    }),
                )
            })
            .clone()
    }
}
