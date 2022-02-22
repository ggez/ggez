//!

use super::{
    context::GraphicsContext,
    image::{Image, ImageFormat},
    mesh::Vertex,
    sampler::Sampler,
};
use crate::Context;

/// A vertex and fragment shader set that can be used to render primitives with various effects.
///
/// The `Params` type parameter can be used to pass data, textures, and samplers to the shader program.
#[derive(Debug)]
pub struct Shader {
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl Shader {
    /// Creates a shader from a WGSL string.
    pub fn from_wgsl(
        ctx: &Context,
        wgsl: &str,
        vs_entry: &str,
        fs_entry: &str,
        format: ImageFormat,
        samples: u32,
    ) -> Self {
        let module = ctx
            .gfx_context
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl.into()),
            });

        let pipeline =
            ctx.gfx_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: vs_entry,
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: memoffset::offset_of!(Vertex, position) as u64,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: memoffset::offset_of!(Vertex, uv) as u64,
                                    shader_location: 1,
                                },
                            ],
                        }],
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: samples,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &module,
                        entry_point: fs_entry,
                        targets: &[wgpu::ColorTargetState {
                            format: format.into(),
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    multiview: None,
                });

        Shader { pipeline }
    }
}

pub use crevice::std430::{AsStd430, Std430};

/// Contains shader parameters passed to the shader program.
///
/// The uniforms, textures, and samplers are all bound to **bind group 1**.
#[derive(Debug)]
pub struct ShaderParams<Uniforms: AsStd430> {
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) uniforms: wgpu::Buffer,
    pub(crate) mapped: bool,
    _phantom: std::marker::PhantomData<Uniforms>,
}

impl<Uniforms: AsStd430> ShaderParams<Uniforms> {
    /// Creates new shader parameters from the texture and sampler list.
    ///
    /// Set `mapped` to true if you expect to call `set_uniforms` mid-frame, otherwise leave as false.
    pub fn new(
        gfx: &mut GraphicsContext,
        shader: &Shader,
        textures: &[&Image],
        samplers: &[Sampler],
        mapped: bool,
    ) -> Self {
        let samplers = samplers
            .iter()
            .map(|sampler| gfx.sampler_cache.get(&gfx.device, *sampler))
            .collect::<Vec<_>>();

        let bindings = textures
            .iter()
            .map(|image| wgpu::BindingResource::TextureView(image.view.as_ref()))
            .chain(
                samplers
                    .iter()
                    .map(|sampler| wgpu::BindingResource::Sampler(sampler.as_ref())),
            )
            .enumerate()
            .map(|(i, resource)| wgpu::BindGroupEntry {
                binding: i as _,
                resource,
            })
            .collect::<Vec<_>>();

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &shader.pipeline.get_bind_group_layout(1),
            entries: &bindings,
        });

        let uniforms = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: <Uniforms as AsStd430>::std430_size_static() as u64,
            usage: if mapped {
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE
            } else {
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            },
            mapped_at_creation: mapped,
        });

        ShaderParams {
            bind_group,
            uniforms,
            mapped,
            _phantom: Default::default(),
        }
    }

    /// Updates the uniforms.
    ///
    /// If `mapped` was set when creating [ShaderParams], the uniforms will be updated immediately.
    ///
    /// If `mapped` was not set, the uniforms will be updated next frame.
    pub fn set_uniforms(&self, gfx: &GraphicsContext, uniforms: &Uniforms) {
        if self.mapped {
            self.uniforms
                .slice(..)
                .get_mapped_range_mut()
                .copy_from_slice(uniforms.as_std430().as_bytes());
        } else {
            gfx.queue
                .write_buffer(&self.uniforms, 0, uniforms.as_std430().as_bytes())
        }
    }
}
