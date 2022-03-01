//!

use crate::{GameError, GameResult};

use super::{
    context::{FrameArenas, GraphicsContext},
    draw::{DrawParam, DrawUniforms},
    gpu::{
        arc::{ArcBindGroup, ArcBindGroupLayout, ArcBuffer, ArcPipelineLayout},
        bind_group::{BindGroupBuilder, BindGroupCache, BindGroupLayoutBuilder},
        pipeline::PipelineCache,
    },
    image::Image,
    instance::InstanceArray,
    mesh::Mesh,
    sampler::{Sampler, SamplerCache},
    shader::{Shader, ShaderParams},
    text::{Text, TextLayout},
    Color, Rect,
};
use crevice::std430::{AsStd430, Std430};
use std::{collections::HashMap, sync::Arc};

pub(crate) const Z_STEP: f32 = 0.001;

/// A canvas represents a render pass and is how you render primitives onto images.
#[allow(missing_debug_implementations)]
pub struct Canvas<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    arenas: &'a FrameArenas,
    bind_group_cache: &'a mut BindGroupCache,
    pipeline_cache: &'a mut PipelineCache,
    sampler_cache: &'a mut SamplerCache,
    glyph_brush: &'a mut wgpu_glyph::GlyphBrush<wgpu::DepthStencilState>,
    fonts: &'a HashMap<String, wgpu_glyph::FontId>,
    staging_belt: &'a mut wgpu::util::StagingBelt,
    encoder: *mut wgpu::CommandEncoder,

    pass: wgpu::RenderPass<'a>,
    samples: u32,
    format: wgpu::TextureFormat,
    target: &'a Image,
    depth: Option<&'a Image>,

    uniform_arena: ArcBuffer,
    uniform_arena_cursor: u64,
    uniform_bind_group: &'a ArcBindGroup,
    uniform_layout: ArcBindGroupLayout,

    instance_uniform_arena: ArcBuffer,
    instance_uniform_arena_cursor: u64,
    instance_uniform_bind_group: &'a ArcBindGroup,
    instance_uniform_layout: ArcBindGroupLayout,

    draw_shader: Shader,
    instance_shader: Shader,
    rect_mesh: Arc<Mesh>,
    white_image: Image,

    transform: glam::Mat4,
    color: Color,
    image_id: Option<u64>,
    z_pos: f32,
    shader_type: ShaderType,
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
        depth: Option<&'a Image>,
    ) -> Self {
        assert!(image.samples() == 1);
        assert!(depth
            .map(|x| x.format().describe().sample_type == wgpu::TextureSampleType::Depth)
            .unwrap_or(true));

        Self::new(gfx, 1, image.format().into(), image, depth, |cmd| {
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
                depth_stencil_attachment: depth.map(|depth| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.),
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                }),
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
        depth: Option<&'a Image>,
    ) -> Self {
        assert!(msaa_image.samples() > 1);
        assert_eq!(resolve_image.samples(), 1);
        assert!(depth
            .map(|x| x.format().describe().sample_type == wgpu::TextureSampleType::Depth)
            .unwrap_or(true));

        Self::new(
            gfx,
            msaa_image.samples(),
            msaa_image.format().into(),
            msaa_image,
            depth,
            |cmd| {
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
                    depth_stencil_attachment: depth.map(|depth| {
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &depth.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0.),
                                store: true,
                            }),
                            stencil_ops: None,
                        }
                    }),
                })
            },
        )
    }

    fn new(
        gfx: &'a mut GraphicsContext,
        samples: u32,
        format: wgpu::TextureFormat,
        target: &'a Image,
        depth: Option<&'a Image>,
        create_pass: impl FnOnce(&'a mut wgpu::CommandEncoder) -> wgpu::RenderPass<'a>,
    ) -> Self {
        let device = &gfx.device;
        let queue = &gfx.queue;
        let bind_group_cache = &mut gfx.bind_group_cache;
        let pipeline_cache = &mut gfx.pipeline_cache;
        let sampler_cache = &mut gfx.sampler_cache;
        let glyph_brush = &mut gfx.glyph_brush;
        let fonts = &gfx.fonts;
        let staging_belt = &mut gfx.staging_belt;

        let (arenas, pass, encoder) = {
            let fcx = gfx.fcx.as_mut().expect("creating canvas when not in frame");

            let encoder = (&mut fcx.cmd) as *mut _;
            let arenas = &fcx.arenas;
            let pass = create_pass(&mut fcx.cmd);

            (arenas, pass, encoder)
        };

        let size = gfx
            .window
            .inner_size()
            .to_logical(gfx.window.scale_factor());
        let transform = glam::Mat4::orthographic_rh(0., size.width, size.height, 0., 0., 1000.)
            * glam::Mat4::from_scale(glam::vec3(1., 1., -1.)); // idk

        let uniform_arena = ArcBuffer::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: DrawUniforms::std430_size_static() as u64 * Self::MAX_DRAWS_PER_FRAME,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        let (uniform_bind_group, uniform_layout) = BindGroupBuilder::new()
            .buffer(
                &uniform_arena,
                0,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                wgpu::BufferBindingType::Uniform,
                true,
                Some(DrawUniforms::std430_size_static() as _),
            )
            .create(device, bind_group_cache);
        let uniform_bind_group = arenas.bind_groups.alloc(uniform_bind_group);

        let instance_uniform_arena =
            ArcBuffer::new(device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: InstanceUniforms::std430_size_static() as u64 * Self::MAX_DRAWS_PER_FRAME,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        let (instance_uniform_bind_group, instance_uniform_layout) = BindGroupBuilder::new()
            .buffer(
                &instance_uniform_arena,
                0,
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
                true,
                Some(InstanceUniforms::std430_size_static() as _),
            )
            .create(device, bind_group_cache);
        let instance_uniform_bind_group = arenas.bind_groups.alloc(instance_uniform_bind_group);

        let mut this = Canvas {
            device,
            queue,
            arenas,
            bind_group_cache,
            pipeline_cache,
            sampler_cache,
            glyph_brush,
            fonts,
            staging_belt,
            encoder,

            pass,
            samples,
            format,
            target,
            depth,

            uniform_arena,
            uniform_arena_cursor: 0,
            uniform_bind_group,
            uniform_layout,

            instance_uniform_arena,
            instance_uniform_arena_cursor: 0,
            instance_uniform_bind_group,
            instance_uniform_layout,

            draw_shader: gfx.draw_shader.clone().unwrap(),
            instance_shader: gfx.instance_shader.clone().unwrap(),
            rect_mesh: gfx.rect_mesh.clone().unwrap(),
            white_image: gfx.white_image.clone().unwrap(),

            transform,
            color: Color::WHITE,
            image_id: None,
            z_pos: 2.,
            shader_type: ShaderType::Draw,
        };

        this.set_default_shader(ShaderType::Draw);
        this.set_sampler(Sampler::linear_clamp());

        this
    }

    /// Sets the shader to use when drawing, along with the provided parameters, **bound to bind group 3**.
    pub fn set_shader_with_params<Uniforms: AsStd430 + 'static>(
        &mut self,
        shader: &Shader,
        params: ShaderParams<Uniforms>,
        ty: ShaderType,
    ) {
        let texture_layout = BindGroupLayoutBuilder::new()
            .image(wgpu::ShaderStages::FRAGMENT)
            .create(self.device, self.bind_group_cache);

        let sampler_layout = BindGroupLayoutBuilder::new()
            .sampler(wgpu::ShaderStages::FRAGMENT)
            .create(self.device, self.bind_group_cache);

        let instance_layout = BindGroupLayoutBuilder::new()
            .buffer(
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Storage { read_only: true },
                false,
            )
            .create(self.device, self.bind_group_cache);

        let layout = match ty {
            ShaderType::Draw => self.pipeline_cache.layout(
                self.device,
                &[
                    self.uniform_layout.clone(),
                    texture_layout,
                    sampler_layout,
                    params.layout.clone(),
                ],
            ),
            ShaderType::Instance => self.pipeline_cache.layout(
                self.device,
                &[
                    self.instance_uniform_layout.clone(),
                    texture_layout,
                    sampler_layout,
                    instance_layout,
                    params.layout.clone(),
                ],
            ),
        };

        self.set_shader_impl(shader, &layout, ty);

        let bind_group = self.arenas.bind_groups.alloc(params.bind_group.clone());
        self.pass
            .set_bind_group(if ty == ShaderType::Draw { 3 } else { 4 }, bind_group, &[]);
    }

    /// Sets the shader to use when drawing.
    pub fn set_shader(&mut self, shader: &Shader, ty: ShaderType) {
        let texture_layout = BindGroupLayoutBuilder::new()
            .image(wgpu::ShaderStages::FRAGMENT)
            .create(self.device, self.bind_group_cache);

        let sampler_layout = BindGroupLayoutBuilder::new()
            .sampler(wgpu::ShaderStages::FRAGMENT)
            .create(self.device, self.bind_group_cache);

        let instance_layout = BindGroupLayoutBuilder::new()
            .buffer(
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Storage { read_only: true },
                false,
            )
            .create(self.device, self.bind_group_cache);

        let layout = match ty {
            ShaderType::Draw => self.pipeline_cache.layout(
                self.device,
                &[self.uniform_layout.clone(), texture_layout, sampler_layout],
            ),
            ShaderType::Instance => self.pipeline_cache.layout(
                self.device,
                &[
                    self.instance_uniform_layout.clone(),
                    texture_layout,
                    sampler_layout,
                    instance_layout,
                ],
            ),
        };

        self.set_shader_impl(shader, &layout, ty);
    }

    fn set_shader_impl(&mut self, shader: &Shader, layout: &ArcPipelineLayout, ty: ShaderType) {
        self.shader_type = ty;
        self.pass.set_pipeline(self.arenas.render_pipelines.alloc(
            self.pipeline_cache.render_pipeline(
                self.device,
                layout.as_ref(),
                shader.info(
                    self.samples,
                    self.format,
                    Some(wgpu::BlendState::ALPHA_BLENDING),
                    self.depth.is_some(),
                    true,
                ),
            ),
        ));
    }

    /// Resets the active shader to the default.
    pub fn set_default_shader(&mut self, ty: ShaderType) {
        self.set_shader(
            &match ty {
                ShaderType::Draw => self.draw_shader.clone(),
                ShaderType::Instance => self.instance_shader.clone(),
            },
            ty,
        );
    }

    /// Sets the active sampler used to sample images.
    pub fn set_sampler(&mut self, sampler: Sampler) {
        let sampler = self.sampler_cache.get(self.device, sampler);

        let (bind_group, _) = BindGroupBuilder::new()
            .sampler(&sampler, wgpu::ShaderStages::FRAGMENT)
            .create(self.device, self.bind_group_cache);

        let bind_group = self.arenas.bind_groups.alloc(bind_group);

        self.pass.set_bind_group(2, bind_group, &[]);
    }

    /// Sets the color that is multiplied against the image color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Draws a mesh.
    pub fn draw_mesh<'b>(
        &mut self,
        mesh: &Mesh,
        image: impl Into<Option<&'b Image>>,
        mut param: DrawParam,
    ) {
        let cursor = self.uniform_arena_cursor;
        self.uniform_arena_cursor += 1;
        let uniforms_size = DrawUniforms::std430_size_static() as u64;
        let byte_cursor = cursor * self.device.limits().min_uniform_buffer_offset_alignment as u64;

        param.src_rect = if let Some(image) = image.into() {
            self.set_image(image);
            param.src_rect
        } else {
            self.set_image(&self.white_image.clone());
            Rect::one()
        };

        param.z = Some(param.z.unwrap_or_else(|| {
            self.z_pos += Z_STEP;
            self.z_pos
        }));

        let mut uniforms = DrawUniforms::from(param);
        uniforms.transform = (self.transform * glam::Mat4::from(uniforms.transform)).into();

        self.queue.write_buffer(
            &self.uniform_arena,
            byte_cursor,
            uniforms.as_std430().as_bytes(),
        );

        self.pass
            .set_bind_group(0, self.uniform_bind_group, &[byte_cursor as _]);

        let verts = self.arenas.buffers.alloc(mesh.verts.clone());
        let inds = self.arenas.buffers.alloc(mesh.inds.clone());

        self.pass.set_vertex_buffer(0, verts.slice(..));
        self.pass
            .set_index_buffer(inds.slice(..), wgpu::IndexFormat::Uint32);

        if self.shader_type != ShaderType::Draw {
            self.set_default_shader(ShaderType::Draw);
        }

        self.pass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
    }

    /// Draws a rectangle.
    pub fn draw<'b>(&mut self, image: impl Into<Option<&'b Image>>, param: DrawParam) {
        self.draw_mesh(&self.rect_mesh.clone(), image, param)
    }

    /// Draws a mesh instanced many times, using the [DrawParam]s found in `instances`.
    ///
    /// `z` specifies the base Z position of the instance array. Set to `None` to use the current Z cursor.
    ///
    /// `skip_z` specifies whether the Z span of the instance array should affect the Z cursor.
    pub fn draw_mesh_instances<'b>(
        &mut self,
        mesh: &Mesh,
        image: impl Into<Option<&'b Image>>,
        instances: &InstanceArray,
        z: Option<f32>,
        skip_z: bool,
    ) {
        if instances.len() == 0 {
            return;
        }

        let cursor = self.instance_uniform_arena_cursor;
        self.instance_uniform_arena_cursor += 1;
        let uniforms_size = InstanceUniforms::std430_size_static() as u64;
        let byte_cursor = cursor * uniforms_size;

        let z_pos = if let Some(z) = z {
            z
        } else {
            self.z_pos + Z_STEP
        };

        if !skip_z {
            self.z_pos = z_pos - 2. * instances.z_min + instances.z_max;
        }

        if let Some(image) = image.into() {
            self.set_image(image);
        } else {
            self.set_image(&self.white_image.clone());
        };

        let transform = self.transform
            * glam::Mat4::from_translation(glam::vec3(0., 0., z_pos - instances.z_min));
        let uniforms = InstanceUniforms {
            transform: transform.into(),
        };

        self.queue.write_buffer(
            &self.instance_uniform_arena,
            byte_cursor,
            uniforms.as_std430().as_bytes(),
        );

        let (bind_group, _) = BindGroupBuilder::new()
            .buffer(
                &instances.buffer,
                0,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                wgpu::BufferBindingType::Storage { read_only: true },
                false,
                None,
            )
            .create(self.device, self.bind_group_cache);

        let bind_group = self.arenas.bind_groups.alloc(bind_group);

        self.pass
            .set_bind_group(0, self.instance_uniform_bind_group, &[byte_cursor as _]);
        self.pass.set_bind_group(3, bind_group, &[]);

        let verts = self.arenas.buffers.alloc(mesh.verts.clone());
        let inds = self.arenas.buffers.alloc(mesh.inds.clone());

        self.pass.set_vertex_buffer(0, verts.slice(..));
        self.pass
            .set_index_buffer(inds.slice(..), wgpu::IndexFormat::Uint32);

        if self.shader_type != ShaderType::Instance {
            self.set_default_shader(ShaderType::Instance);
        }

        self.pass
            .draw_indexed(0..mesh.index_count as u32, 0, 0..instances.len() as u32);
    }

    /// Equivalent of `draw` (as is to `draw_mesh`) for instanced rendering (i.e. as is to `draw_mesh_instances`).
    pub fn draw_instances<'b>(
        &mut self,
        image: impl Into<Option<&'b Image>>,
        instances: &InstanceArray,
        z: Option<f32>,
        skip_z: bool,
    ) {
        self.draw_mesh_instances(&self.rect_mesh.clone(), image, instances, z, skip_z)
    }

    /// Draws a section text that is fit and aligned into a given `rect` bounds.
    ///
    /// The section can be made up of multiple [Text], letting the user have complex formatting
    /// in the same section of text (e.g. bolding, highlighting, headers, etc).
    ///
    /// [TextLayout] determines how the text is aligned in `rect` and whether the text wraps or not.
    ///
    /// Depth must be enabled to draw text.
    pub fn draw_bounded_text(&mut self, text: &[Text], rect: Rect, layout: TextLayout) {
        assert!(self.depth.is_some());

        let mut section = wgpu_glyph::Section::default()
            .with_screen_position((rect.x, rect.y))
            .with_bounds((rect.w, rect.h))
            .with_layout(match layout {
                TextLayout::SingleLine { h_align, v_align } => {
                    wgpu_glyph::Layout::default_single_line()
                        .h_align(h_align.into())
                        .v_align(v_align.into())
                }
                TextLayout::Wrap { h_align, v_align } => wgpu_glyph::Layout::default_wrap()
                    .h_align(h_align.into())
                    .v_align(v_align.into()),
            });

        for text in text {
            let z = text.z.unwrap_or_else(|| {
                self.z_pos += Z_STEP;
                self.z_pos
            });

            section = section.add_text(wgpu_glyph::Text {
                text: text.text,
                scale: text.size.into(),
                font_id: *self.fonts.get(text.font).expect("invalid font name"),
                extra: wgpu_glyph::Extra {
                    color: text.color.into(),
                    z,
                },
            });
        }

        self.glyph_brush.queue(section);
    }

    /// Unbounded version of `draw_bounded_text`.
    pub fn draw_text(
        &mut self,
        text: &[Text],
        pos: impl Into<mint::Vector2<f32>>,
        layout: TextLayout,
    ) {
        let pos = pos.into();
        self.draw_bounded_text(
            text,
            Rect::new(pos.x, pos.y, f32::INFINITY, f32::INFINITY),
            layout,
        )
    }

    /// Finish drawing with this canvas.
    #[allow(unsafe_code)]
    pub fn finish(self) -> GameResult<()> {
        if self.depth.is_some() {
            let Canvas {
                device,
                staging_belt,
                glyph_brush,
                encoder,
                target,
                depth,
                transform,
                pass,
                ..
            } = self;

            std::mem::drop(pass);

            glyph_brush
                .draw_queued_with_transform(
                    device,
                    staging_belt,
                    unsafe { encoder.as_mut().unwrap() },
                    target.view.as_ref(),
                    wgpu::RenderPassDepthStencilAttachment {
                        view: depth.unwrap().view.as_ref(),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }),
                        stencil_ops: None,
                    },
                    transform.to_cols_array(),
                )
                .map_err(|s| GameError::RenderError(s))?;
        }

        Ok(())
    }

    fn set_image(&mut self, image: &Image) {
        if self
            .image_id
            .map(|id| id != image.view.id())
            .unwrap_or(true)
        {
            self.image_id = Some(image.view.id());

            let (bind_group, _) = BindGroupBuilder::new()
                .image(&image.view, wgpu::ShaderStages::FRAGMENT)
                .create(self.device, self.bind_group_cache);

            let bind_group = self.arenas.bind_groups.alloc(bind_group);

            self.pass.set_bind_group(1, bind_group, &[]);
        }
    }
}

/// Describes what part of the drawing pipeline the shader handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderType {
    /// The shader handles non-instanced draws.
    Draw,
    /// The shader handles instanced draws (i.e. using [InstanceArray]).
    Instance,
}

/// Describes the image load operation when starting a new canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanvasLoadOp {
    /// Keep the existing contents of the image.
    DontClear,
    /// Clear the image contents to a solid color.
    Clear(Color),
}

#[derive(crevice::std430::AsStd430)]
struct InstanceUniforms {
    pub transform: mint::ColumnMatrix4<f32>,
}
