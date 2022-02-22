use super::{
    image::{Image, ImageFormat},
    mesh::Vertex,
    sampler::Sampler,
};
use crate::Context;

/// A vertex and fragment shader set that can be used to render primitives with various effects.
///
/// The `Params` type parameter can be used to pass data, textures, and samplers to the shader program.
#[derive(Debug)]
pub struct Shader<Params: ShaderParams> {
    pipeline: wgpu::RenderPipeline,
    _phantom: std::marker::PhantomData<Params>,
}

impl<Params: ShaderParams> Shader<Params> {
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

        Shader {
            pipeline,
            _phantom: Default::default(),
        }
    }
}

/// A type that describes the parameters that are supplied to a shader.
pub trait ShaderParams {
    /// The uniforms data.
    /// This is a good place to put general data that needs to be passed to the shader.
    type Uniforms;
    /// The exact number of textures this shader uses.
    const TEXTURE_COUNT: usize;
    /// The exact number of samplers this shader uses.
    const SAMPLER_COUNT: usize;

    /// Return the value of the uniforms.
    fn uniforms(&self) -> &Self::Uniforms;
    /// Return a slice with all the textures (as image handles).
    ///
    /// The length of the slice **must** match [ShaderParams::TEXTURE_COUNT].
    fn textures(&self) -> &[&Image];
    /// Return a slice with all the samplers.
    ///
    /// The length of the slice **must** match [ShaderParams::SAMPLER_COUNT].
    fn samplers(&self) -> &[Sampler];
}
