//!

use super::{
    context::{FrameArenas, GraphicsContext},
    draw::{DrawParam, DrawUniforms, ZPosition},
    gpu::{
        arc::{ArcBindGroupLayout, ArcShaderModule, ArcTextureView},
        bind_group::{BindGroupBuilder, BindGroupCache, BindGroupLayoutBuilder},
        growing::GrowingBufferArena,
        pipeline::PipelineCache,
        text::{TextRenderer, TextVertex},
    },
    image::Image,
    instance::InstanceArray,
    mesh::{Mesh, Vertex},
    sampler::{Sampler, SamplerCache},
    shader::{Shader, ShaderParams},
    text::{Text, TextLayout},
    Color, Rect,
};
use crate::{GameError, GameResult};
use crevice::std430::{AsStd430, Std430};
use std::{collections::HashMap, hash::Hash, sync::Arc};

/// A canvas represents a render pass and is how you render primitives onto images.
#[allow(missing_debug_implementations)]
pub struct Canvas<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    arenas: &'a FrameArenas,
    bind_group_cache: &'a mut BindGroupCache,
    pipeline_cache: &'a mut PipelineCache,
    sampler_cache: &'a mut SamplerCache,
    text_renderer: &'a mut TextRenderer,
    fonts: &'a HashMap<String, glyph_brush::FontId>,
    uniform_arena: &'a mut GrowingBufferArena,

    shader: Shader,
    shader_bind_group: Option<(&'a wgpu::BindGroup, ArcBindGroupLayout)>,
    text_shader: Shader,
    text_shader_bind_group: Option<(&'a wgpu::BindGroup, ArcBindGroupLayout)>,

    shader_ty: Option<ShaderType>,
    dirty_pipeline: bool,
    pass: wgpu::RenderPass<'a>,
    samples: u32,
    format: wgpu::TextureFormat,
    text_uniforms: &'a wgpu::BindGroup,

    draw_sm: ArcShaderModule,
    instance_sm: ArcShaderModule,
    text_sm: ArcShaderModule,
    rect_mesh: Arc<Mesh>,
    white_image: Image,

    transform: glam::Mat4,
    image_id: Option<u64>,
}

impl<'a> Canvas<'a> {
    /// Create a new [Canvas] from an image. This will allow for drawing to a single color image.
    ///
    /// The image must be created for Canvas usage, i.e. [Image::new_canvas_image], or [ScreenImage], and must only have a sample count of 1.
    pub fn from_image(
        gfx: &'a mut GraphicsContext,
        load_op: CanvasLoadOp,
        image: &'a Image,
    ) -> Self {
        assert!(gfx.fcx.is_some(), "starting Canvas outside of frame");
        assert!(image.samples() == 1);

        Self::new(gfx, 1, image.format().into(), |cmd| {
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
        assert!(gfx.fcx.is_some(), "starting Canvas outside of frame");
        assert!(msaa_image.samples() > 1);
        assert_eq!(resolve_image.samples(), 1);

        Self::new(
            gfx,
            msaa_image.samples(),
            msaa_image.format().into(),
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
                    depth_stencil_attachment: None,
                })
            },
        )
    }

    fn new(
        gfx: &'a mut GraphicsContext,
        samples: u32,
        format: wgpu::TextureFormat,
        create_pass: impl FnOnce(&'a mut wgpu::CommandEncoder) -> wgpu::RenderPass<'a>,
    ) -> Self {
        let device = &gfx.device;
        let queue = &gfx.queue;
        let bind_group_cache = &mut gfx.bind_group_cache;
        let pipeline_cache = &mut gfx.pipeline_cache;
        let sampler_cache = &mut gfx.sampler_cache;
        let text_renderer = &mut gfx.text;
        let fonts = &gfx.fonts;
        let uniform_arena = &mut gfx.uniform_arena;

        let (arenas, pass) = {
            let fcx = gfx.fcx.as_mut().expect("creating canvas when not in frame");

            let arenas = &fcx.arenas;
            let pass = create_pass(&mut fcx.cmd);

            (arenas, pass)
        };

        let size = gfx.window.inner_size();
        let transform = glam::Mat4::orthographic_rh(
            0.,
            size.width as _,
            size.height as _,
            0.,
            ZPosition::MAX as f32,
            ZPosition::MIN as f32,
        );

        let shader = Shader {
            fragment: gfx.draw_shader.clone(),
            fs_entry: "fs_main".into(),
        };

        let text_shader = Shader {
            fragment: gfx.text_shader.clone(),
            fs_entry: "fs_main".into(),
        };

        let text_uniforms = uniform_arena.allocate(
            device,
            mint::ColumnMatrix4::<f32>::std430_size_static() as _,
        );

        queue.write_buffer(
            &text_uniforms.buffer,
            text_uniforms.offset,
            (mint::ColumnMatrix4::<f32>::from(transform))
                .as_std430()
                .as_bytes(),
        );

        let (text_uniforms, _) = BindGroupBuilder::new()
            .buffer(
                &text_uniforms.buffer,
                text_uniforms.offset,
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
                false,
                Some(mint::ColumnMatrix4::<f32>::std430_size_static() as _),
            )
            .create(device, bind_group_cache);

        let text_uniforms = arenas.bind_groups.alloc(text_uniforms);

        let mut this = Canvas {
            device,
            queue,
            arenas,
            bind_group_cache,
            pipeline_cache,
            sampler_cache,
            text_renderer,
            fonts,
            uniform_arena,

            shader,
            shader_bind_group: None,
            text_shader,
            text_shader_bind_group: None,

            shader_ty: None,
            dirty_pipeline: true,
            pass,
            samples,
            format,
            text_uniforms,

            draw_sm: gfx.draw_shader.clone(),
            instance_sm: gfx.instance_shader.clone(),
            text_sm: gfx.text_shader.clone(),
            rect_mesh: gfx.rect_mesh.clone().unwrap(),
            white_image: gfx.white_image.clone().unwrap(),

            transform,
            image_id: None,
        };

        this.set_sampler(Sampler::linear_clamp());

        this
    }

    /// Sets the shader to use when drawing meshes, along with the provided parameters, **bound to bind group 3**.
    pub fn set_shader_with_params<Uniforms: AsStd430 + 'static>(
        &mut self,
        shader: Shader,
        params: ShaderParams<Uniforms>,
    ) {
        self.dirty_pipeline = true;
        self.shader = shader;
        self.shader_bind_group = Some((
            self.arenas.bind_groups.alloc(params.bind_group.clone()),
            params.layout.clone(),
        ));
    }

    /// Sets the shader to use when drawing meshes.
    pub fn set_shader(&mut self, shader: Shader) {
        self.dirty_pipeline = true;
        self.shader = shader;
    }

    /// Sets the shader to use when drawing meshes, along with the provided parameters, **bound to bind group 3**.
    pub fn set_text_shader_with_params<Uniforms: AsStd430 + 'static>(
        &mut self,
        shader: Shader,
        params: ShaderParams<Uniforms>,
    ) {
        self.dirty_pipeline = true;
        self.text_shader = shader;
        self.text_shader_bind_group = Some((
            self.arenas.bind_groups.alloc(params.bind_group.clone()),
            params.layout.clone(),
        ));
    }

    /// Sets the shader to use when drawing meshes.
    pub fn set_text_shader(&mut self, shader: Shader) {
        self.dirty_pipeline = true;
        self.text_shader = shader;
    }

    /// Resets the active mesh shader to the default.
    pub fn set_default_shader(&mut self) {
        self.set_shader(Shader {
            fragment: self.draw_sm.clone(),
            fs_entry: "fs_main".into(),
        });
    }

    /// Resets the active mesh shader to the default.
    pub fn set_default_text_shader(&mut self) {
        self.set_text_shader(Shader {
            fragment: self.draw_sm.clone(),
            fs_entry: "fs_main".into(),
        });
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

    /// Draws a mesh.
    pub fn draw_mesh<'b>(
        &mut self,
        mesh: &Mesh,
        image: impl Into<Option<&'b Image>>,
        mut param: DrawParam,
    ) {
        self.update_pipeline(ShaderType::Draw);

        let alloc_size = self.device.limits().min_uniform_buffer_offset_alignment as u64;
        let uniform_alloc = self.uniform_arena.allocate(self.device, alloc_size);

        let (uniform_bind_group, _) = BindGroupBuilder::new()
            .buffer(
                &uniform_alloc.buffer,
                0,
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
                true,
                Some(alloc_size),
            )
            .create(self.device, self.bind_group_cache);

        let (src_rect, image_scale) = if let Some(image) = image.into() {
            self.set_image(&image.view);
            (
                param.src_rect,
                glam::Vec2::new(image.width() as _, image.height() as _),
            )
        } else {
            self.set_image(&self.white_image.view.clone());
            (Rect::one(), glam::Vec2::ONE)
        };

        param.src_rect = src_rect;

        let mut uniforms = DrawUniforms::from_param(param, image_scale.into());
        uniforms.transform = (self.transform * glam::Mat4::from(uniforms.transform)).into();

        self.queue.write_buffer(
            &uniform_alloc.buffer,
            uniform_alloc.offset,
            uniforms.as_std430().as_bytes(),
        );

        self.pass.set_bind_group(
            0,
            self.arenas.bind_groups.alloc(uniform_bind_group),
            &[uniform_alloc.offset as u32],
        );

        let verts = self.arenas.buffers.alloc(mesh.verts.clone());
        let inds = self.arenas.buffers.alloc(mesh.inds.clone());

        self.pass.set_vertex_buffer(0, verts.slice(..));
        self.pass
            .set_index_buffer(inds.slice(..), wgpu::IndexFormat::Uint32);

        self.pass.draw_indexed(0..mesh.index_count as _, 0, 0..1);
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
    ) {
        if instances.len() == 0 {
            return;
        }

        self.update_pipeline(ShaderType::Instance);

        let alloc_size = self.device.limits().min_uniform_buffer_offset_alignment as u64;
        let uniform_alloc = self.uniform_arena.allocate(self.device, alloc_size);

        let (uniform_bind_group, _) = BindGroupBuilder::new()
            .buffer(
                &uniform_alloc.buffer,
                0,
                wgpu::ShaderStages::VERTEX,
                wgpu::BufferBindingType::Uniform,
                true,
                Some(alloc_size),
            )
            .create(self.device, self.bind_group_cache);

        let image_scale = if let Some(image) = image.into() {
            self.set_image(&image.view);
            glam::Vec2::new(image.width() as _, image.height() as _)
        } else {
            self.set_image(&self.white_image.view.clone());
            glam::Vec2::ONE
        };

        let uniforms = InstanceUniforms {
            transform: self.transform.into(),
            pre_transform: glam::Mat4::from_scale(glam::vec3(image_scale.x, image_scale.y, 0.))
                .into(),
        };

        self.queue.write_buffer(
            &uniform_alloc.buffer,
            uniform_alloc.offset,
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

        self.pass.set_bind_group(
            0,
            self.arenas.bind_groups.alloc(uniform_bind_group),
            &[uniform_alloc.offset as u32],
        );
        self.pass.set_bind_group(3, bind_group, &[]);

        let verts = self.arenas.buffers.alloc(mesh.verts.clone());
        let inds = self.arenas.buffers.alloc(mesh.inds.clone());

        self.pass.set_vertex_buffer(0, verts.slice(..));
        self.pass
            .set_index_buffer(inds.slice(..), wgpu::IndexFormat::Uint32);

        self.pass
            .draw_indexed(0..mesh.index_count as _, 0, 0..instances.len() as _);
    }

    /// Equivalent of `draw` (as is to `draw_mesh`) for instanced rendering (i.e. as is to `draw_mesh_instances`).
    pub fn draw_instances<'b>(
        &mut self,
        image: impl Into<Option<&'b Image>>,
        instances: &InstanceArray,
    ) {
        self.draw_mesh_instances(&self.rect_mesh.clone(), image, instances)
    }

    /// Draws a section text that is fit and aligned into a given `rect` bounds.
    ///
    /// The section can be made up of multiple [Text], letting the user have complex formatting
    /// in the same section of text (e.g. bolding, highlighting, headers, etc).
    ///
    /// [TextLayout] determines how the text is aligned in `rect` and whether the text wraps or not.
    pub fn draw_bounded_text(
        &mut self,
        text: &[Text],
        rect: Rect,
        layout: TextLayout,
    ) -> GameResult<()> {
        self.update_pipeline(ShaderType::Text);

        self.text_renderer.queue(glyph_brush::Section {
            screen_position: (rect.x, rect.y),
            bounds: (rect.w, rect.h),
            layout: match layout {
                TextLayout::SingleLine { h_align, v_align } => {
                    glyph_brush::Layout::default_single_line()
                        .h_align(h_align.into())
                        .v_align(v_align.into())
                }
                TextLayout::Wrap { h_align, v_align } => glyph_brush::Layout::default_wrap()
                    .h_align(h_align.into())
                    .v_align(v_align.into()),
            },
            text: text
                .iter()
                .map(|text| {
                    Ok(glyph_brush::Text {
                        text: &text.text,
                        scale: text.size.into(),
                        font_id: *self
                            .fonts
                            .get(text.font.as_ref())
                            .ok_or_else(|| GameError::FontSelectError(text.font.to_string()))?,
                        extra: glyph_brush::Extra {
                            color: text.color.into(),
                            z: 0.,
                        },
                    })
                })
                .collect::<GameResult<Vec<_>>>()?,
        });

        self.set_image(&self.text_renderer.cache_view.clone());
        self.pass.set_bind_group(0, self.text_uniforms, &[]);

        self.text_renderer
            .draw_queued(self.device, self.queue, self.arenas, &mut self.pass)?;

        Ok(())
    }

    /// Unbounded version of `draw_bounded_text`.
    pub fn draw_text(
        &mut self,
        text: &[Text],
        pos: impl Into<mint::Vector2<f32>>,
        layout: TextLayout,
    ) -> GameResult<()> {
        let pos = pos.into();
        self.draw_bounded_text(
            text,
            Rect::new(pos.x, pos.y, f32::INFINITY, f32::INFINITY),
            layout,
        )
    }

    /// Finish drawing with this canvas.
    pub fn finish(self) {}

    fn update_pipeline(&mut self, ty: ShaderType) {
        if self.dirty_pipeline || self.shader_ty != Some(ty) {
            self.dirty_pipeline = false;
            self.shader_ty = Some(ty);

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

            let uniform_layout = BindGroupLayoutBuilder::new()
                .buffer(
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Uniform,
                    ty != ShaderType::Text,
                )
                .create(self.device, self.bind_group_cache);

            let mut groups = vec![uniform_layout, texture_layout, sampler_layout];

            if ty == ShaderType::Instance {
                groups.push(instance_layout);
            }

            let shader = match ty {
                ShaderType::Draw | ShaderType::Instance => {
                    if let Some((bind_group, bind_group_layout)) = &self.shader_bind_group {
                        self.pass.set_bind_group(
                            if ty == ShaderType::Draw { 3 } else { 4 },
                            bind_group,
                            &[],
                        );

                        groups.push(bind_group_layout.clone());
                    }

                    &self.shader
                }
                ShaderType::Text => {
                    if let Some((bind_group, bind_group_layout)) = &self.text_shader_bind_group {
                        self.pass.set_bind_group(3, bind_group, &[]);
                        groups.push(bind_group_layout.clone());
                    }

                    &self.text_shader
                }
            };

            let layout = self.pipeline_cache.layout(self.device, &groups);
            let pipeline = self
                .arenas
                .render_pipelines
                .alloc(self.pipeline_cache.render_pipeline(
                    self.device,
                    layout.as_ref(),
                    shader.info(
                        match ty {
                            ShaderType::Draw => self.draw_sm.clone(),
                            ShaderType::Instance => self.instance_sm.clone(),
                            ShaderType::Text => self.text_sm.clone(),
                        },
                        self.samples,
                        self.format,
                        Some(wgpu::BlendState::ALPHA_BLENDING),
                        false,
                        true,
                        match ty {
                            ShaderType::Text => wgpu::PrimitiveTopology::TriangleStrip,
                            _ => wgpu::PrimitiveTopology::TriangleList,
                        },
                        match ty {
                            ShaderType::Text => TextVertex::layout(),
                            _ => Vertex::layout(),
                        },
                    ),
                ));

            self.pass.set_pipeline(pipeline);
        }
    }

    fn set_image(&mut self, view: &ArcTextureView) {
        if self.image_id.map(|id| id != view.id()).unwrap_or(true) {
            self.image_id = Some(view.id());

            let (bind_group, _) = BindGroupBuilder::new()
                .image(&view, wgpu::ShaderStages::FRAGMENT)
                .create(self.device, self.bind_group_cache);

            let bind_group = self.arenas.bind_groups.alloc(bind_group);

            self.pass.set_bind_group(1, bind_group, &[]);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ShaderType {
    Draw,
    Instance,
    Text,
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
    pub pre_transform: mint::ColumnMatrix4<f32>,
}
