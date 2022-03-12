//!

use super::{
    context::GraphicsContext,
    gpu::{
        arc::{ArcBindGroup, ArcBindGroupLayout, ArcBuffer, ArcShaderModule},
        bind_group::BindGroupBuilder,
        pipeline::RenderPipelineInfo,
    },
    image::Image,
    sampler::Sampler,
};
use crevice::std430::Std430;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;

/// A custom fragment shader that can be used to render with shader effects.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shader {
    pub(crate) fragment: ArcShaderModule,
    pub(crate) fs_entry: String,
}

impl Shader {
    /// Creates a shader from a WGSL string.
    pub fn from_wgsl(gfx: &GraphicsContext, wgsl: &str, fs_entry: &str) -> Self {
        let module = ArcShaderModule::new(gfx.device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl.into()),
            },
        ));

        Shader {
            fragment: module,
            fs_entry: fs_entry.into(),
        }
    }

    pub(crate) fn info(
        &self,
        vs: ArcShaderModule,
        samples: u32,
        format: wgpu::TextureFormat,
        blend: Option<wgpu::BlendState>,
        depth: bool,
        vertices: bool,
        topology: wgpu::PrimitiveTopology,
        vertex_layout: wgpu::VertexBufferLayout<'static>,
    ) -> RenderPipelineInfo {
        RenderPipelineInfo {
            vs,
            fs: self.fragment.clone(),
            vs_entry: "vs_main".into(),
            fs_entry: self.fs_entry.clone(),
            samples,
            format,
            blend,
            depth,
            vertices,
            topology,
            vertex_layout,
        }
    }
}

pub use crevice::std430::AsStd430;

/// Parameters that can be passed to a custom shader, including uniforms, images, and samplers.
#[derive(Debug)]
pub struct ShaderParams<Uniforms: AsStd430> {
    pub(crate) uniforms: ArcBuffer,
    pub(crate) layout: ArcBindGroupLayout,
    pub(crate) bind_group: ArcBindGroup,
    _marker: PhantomData<Uniforms>,
}

impl<Uniforms: AsStd430> ShaderParams<Uniforms> {
    /// Creates a new [ShaderParams], initialized with the given uniforms, images, and samplers.
    pub fn new(
        gfx: &mut GraphicsContext,
        uniforms: &Uniforms,
        images: &[&Image],
        samplers: &[Sampler],
    ) -> Self {
        let uniforms = ArcBuffer::new(gfx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: uniforms.as_std430().as_bytes(),
            },
        ));

        let samplers = samplers
            .iter()
            .map(|&sampler| gfx.sampler_cache.get(&gfx.device, sampler))
            .collect::<Vec<_>>();

        let mut builder = BindGroupBuilder::new();
        builder = builder.buffer(
            &uniforms,
            0,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            wgpu::BufferBindingType::Uniform,
            false,
            None,
        );

        for image in images {
            builder = builder.image(&image.view, wgpu::ShaderStages::FRAGMENT);
        }

        for sampler in &samplers {
            builder = builder.sampler(sampler, wgpu::ShaderStages::FRAGMENT);
        }

        let (bind_group, layout) = builder.create(&gfx.device, &mut gfx.bind_group_cache);

        ShaderParams {
            uniforms,
            layout,
            bind_group,
            _marker: PhantomData,
        }
    }

    /// Updates the uniform data.
    pub fn set_uniforms(&self, gfx: &GraphicsContext, uniforms: &Uniforms) {
        gfx.queue
            .write_buffer(&self.uniforms, 0, uniforms.as_std430().as_bytes());
    }
}

impl<Uniforms: AsStd430> Clone for ShaderParams<Uniforms> {
    fn clone(&self) -> Self {
        Self {
            uniforms: self.uniforms.clone(),
            layout: self.layout.clone(),
            bind_group: self.bind_group.clone(),
            _marker: PhantomData,
        }
    }
}
