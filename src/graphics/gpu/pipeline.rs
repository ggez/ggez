use super::arc::{ArcBindGroupLayout, ArcPipelineLayout, ArcRenderPipeline, ArcShaderModule};
use crate::graphics::mesh::Vertex;
use std::collections::{hash_map::DefaultHasher, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RenderPipelineKey {
    vs_id: u64,
    fs_id: u64,
    vs_entry: String,
    fs_entry: String,
    samples: u32,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    depth: bool,
    vertices: bool,
}

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
}

#[derive(Debug)]
pub struct PipelineCache {
    pipelines: HashMap<RenderPipelineKey, ArcRenderPipeline>,
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
        let key = RenderPipelineKey {
            vs_id: info.vs.id(),
            fs_id: info.fs.id(),
            vs_entry: info.vs_entry.clone(),
            fs_entry: info.fs_entry.clone(),
            samples: info.samples,
            format: info.format,
            blend: info.blend,
            depth: info.depth,
            vertices: info.vertices,
        };

        let vertex_buffers = [Vertex::layout()];

        self.pipelines
            .entry(key)
            .or_insert_with(|| {
                ArcRenderPipeline::new(device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(layout),
                        vertex: wgpu::VertexState {
                            module: &*info.vs,
                            entry_point: &info.vs_entry,
                            buffers: if info.vertices { &vertex_buffers } else { &[] },
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
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
                                depth_compare: wgpu::CompareFunction::Less,
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
                            module: &*info.fs,
                            entry_point: &info.fs_entry,
                            targets: &[wgpu::ColorTargetState {
                                format: info.format,
                                blend: info.blend,
                                write_mask: wgpu::ColorWrites::ALL,
                            }],
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
