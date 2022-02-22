//!

use super::{
    context::GraphicsContext, image::Image, mesh::Mesh, shader, transform::Transform, Color, Rect,
};
use crevice::std430::{AsStd430, Std430};
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// A canvas represents a render pass and is how you render primitives onto images.
#[derive(Debug)]
pub struct Canvas<'a> {
    device: &'a wgpu::Device,
    pass: wgpu::RenderPass<'a>,
    target: &'a Image,
    resolve: Option<&'a Image>,

    default_pipeline: Arc<wgpu::RenderPipeline>,
    uniform_arena: wgpu::Buffer,
    uniform_arena_cursor: u64,
    bind_group: wgpu::BindGroup,

    batch_mesh_id: Option<usize>,
}

impl<'a> Canvas<'a> {
    // this is a temporary limitation right now.
    // dont be afraid to crank this number up tho; uniforms are cheap in memory.
    // TODO(jazzfool): impl growing uniform arenas to remove this limitation
    const MAX_DRAWS_PER_FRAME: u64 = 1024;

    /// Create a new [Canvas] from an image. This will allow for drawing to a single color image.
    ///
    /// The image must be created for Canvas usage, i.e. [Image::new_canvas_image], or [ScreenImage], and must only have a sample count of 1.
    pub fn from_image(
        gfx: &'a mut GraphicsContext,
        load_op: CanvasLoadOp,
        image: &'a Image,
    ) -> Self {
        assert!(image.samples() == 1);

        Self::new(gfx, image, None, |cmd| {
            cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: image.view.as_ref(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match load_op {
                            CanvasLoadOp::DontClear => wgpu::LoadOp::Load,
                            CanvasLoadOp::Clear(color) => wgpu::LoadOp::Clear(color.into()),
                        },
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            })
        })
    }

    /// Create a new [Canvas] from an MSAA image and a resolve target. This will allow for drawing with MSAA to a color image, then resolving the samples into a secondary target.
    ///
    /// Both images must be created for Canvas usage (see [Canvas::from_image]). `msaa_image` must have a sample count > 1 and `resolve_image` must strictly have a sample count of 1.
    pub fn from_msaa(
        gfx: &'a mut GraphicsContext,
        load_op: CanvasLoadOp,
        msaa_image: &'a Image,
        resolve_image: &'a Image,
    ) -> Self {
        assert!(msaa_image.samples() > 1);
        assert!(resolve_image.samples() == 1);

        Self::new(gfx, msaa_image, Some(resolve_image), |cmd| {
            cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: msaa_image.view.as_ref(),
                    resolve_target: Some(resolve_image.view.as_ref()),
                    ops: wgpu::Operations {
                        load: match load_op {
                            CanvasLoadOp::DontClear => wgpu::LoadOp::Load,
                            CanvasLoadOp::Clear(color) => wgpu::LoadOp::Clear(color.into()),
                        },
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            })
        })
    }

    #[allow(unsafe_code)]
    fn new(
        gfx: &'a mut GraphicsContext,
        target: &'a Image,
        resolve: Option<&'a Image>,
        create_pass: impl FnOnce(&'a mut wgpu::CommandEncoder) -> wgpu::RenderPass<'a>,
    ) -> Self {
        let device = &gfx.device;
        let mut pass = create_pass(&mut gfx.fcx.as_mut().unwrap().cmd);

        let default_pipeline = gfx.pipelines.default.clone();

        let uniform_arena = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<DrawUniforms>() as u64 * Self::MAX_DRAWS_PER_FRAME,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_arena,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        pass.set_pipeline(unsafe { &*(default_pipeline.as_ref() as *const _) });

        Canvas {
            device,
            pass,
            target,
            resolve,

            default_pipeline,
            uniform_arena,
            uniform_arena_cursor: 0,
            bind_group,

            batch_mesh_id: None,
        }
    }

    /// Sets the shader to use when drawing.
    pub fn set_shader<Uniforms: shader::AsStd430>(
        &mut self,
        shader: &'a shader::Shader,
        params: &'a shader::ShaderParams<Uniforms>,
    ) {
        self.pass.set_pipeline(&shader.pipeline);
        self.pass.set_bind_group(1, &params.bind_group, &[]);
    }

    /// Draws a mesh.
    #[allow(unsafe_code)]
    pub fn draw_mesh(&mut self, mesh: &Mesh, src_rect: Rect, transform: Transform) {
        let cursor = self.uniform_arena_cursor;
        self.uniform_arena_cursor += 1;

        let uniforms_size = DrawUniforms::std430_size_static() as u64;
        let byte_cursor = cursor * uniforms_size;

        let uniforms = DrawUniforms {};

        self.uniform_arena
            .slice(byte_cursor..(byte_cursor + uniforms_size))
            .get_mapped_range_mut()
            .copy_from_slice(uniforms.as_std430().as_bytes());

        self.pass.set_bind_group(
            0,
            unsafe { &*(&self.bind_group as *const _) },
            &[byte_cursor as _],
        );

        self.pass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
    }
}

#[derive(shader::AsStd430)]
struct DrawUniforms {}

/// Describes the image load operation when starting a new canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanvasLoadOp {
    /// Keep the existing contents of the image.
    DontClear,
    /// Clear the image contents to a solid color.
    Clear(Color),
}
