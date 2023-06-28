use crate::{
    glam::*,
    graphics::{
        self, Aabb, Camera3dBundle, CameraUniform, Color, Instance3d, Mesh3d, Shader, Transform3d,
        Vertex3d, WgpuContext,
    },
    Context, GameError, GameResult,
};
use std::sync::Arc;

use wgpu::util::DeviceExt;

/// A 3d version of `DrawParam` used for transformation of 3d meshes
#[derive(Clone, Copy, Debug)]
pub struct DrawParam3d {
    /// The transform of the mesh to draw see `Transform3d`
    pub transform: Transform3d,
    /// The alpha component is used for intensity of blending instead of actual alpha
    pub color: Color,
}

impl DrawParam3d {
    /// Change the scale of the `DrawParam3d`
    pub fn scale<V>(mut self, scale_: V) -> Self
    where
        V: Into<mint::Vector3<f32>>,
    {
        let p: mint::Vector3<f32> = scale_.into();
        self.transform.scale = p;
        self
    }

    /// Change the position of the `DrawParam3d`
    pub fn position<P>(mut self, position_: P) -> Self
    where
        P: Into<mint::Vector3<f32>>,
    {
        let p: mint::Vector3<f32> = position_.into();
        self.transform.position = p;
        self
    }

    /// Change the rotation of the `DrawParam3d`
    pub fn rotation<R>(mut self, rotation_: R) -> Self
    where
        R: Into<mint::Quaternion<f32>>,
    {
        let p: mint::Quaternion<f32> = rotation_.into();
        self.transform.rotation = p;
        self
    }

    /// Change the color of the `DrawParam3d`
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Change the transform of the `DrawParam3d`
    pub fn transform(mut self, transform: Transform3d) -> Self {
        self.transform = transform;
        self
    }
}

impl Default for DrawParam3d {
    fn default() -> Self {
        Self {
            transform: Transform3d::default(),
            color: Color::new(1.0, 1.0, 1.0, 0.0),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DrawState3d {
    pub(crate) shader: Shader,
}

#[derive(Clone, Debug)]
pub(crate) struct DrawCommand3d {
    pub(crate) mesh: Mesh3d, // Maybe take a reference instead
    pub(crate) state: DrawState3d,
    pub(crate) param: DrawParam3d,
}

/// A 3d Canvas for rendering 3d objects
#[derive(Debug)]
pub struct Canvas3d {
    pub(crate) wgpu: Arc<WgpuContext>,
    pub(crate) default_shader: Shader,
    pub(crate) default_image: graphics::Image,
    pub(crate) draws: Vec<DrawCommand3d>,
    pub(crate) dirty_pipeline: bool,
    pub(crate) state: DrawState3d,
    pub(crate) original_state: DrawState3d,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) depth: graphics::ScreenImage,
    pub(crate) camera_uniform: CameraUniform,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) camera_buffer: wgpu::Buffer,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) target: graphics::Image,
}

impl Canvas3d {
    /// Create a `Canvas3d` from a frame. This will fill the whole window
    pub fn from_frame(ctx: &mut Context, camera: &mut Camera3dBundle) -> Self {
        Self::new(ctx, camera, ctx.gfx.frame().clone())
    }

    /// Createa a `Canvas3d` from an image to render to
    pub fn from_image(
        ctx: &mut Context,
        camera: &mut Camera3dBundle,
        image: graphics::Image,
    ) -> Self {
        Self::new(ctx, camera, image)
    }

    pub(crate) fn new(
        ctx: &mut Context,
        camera: &mut Camera3dBundle,
        target: graphics::Image,
    ) -> Self {
        let cube_code = include_str!("shader/draw3d.wgsl");
        let shader = graphics::ShaderBuilder::from_code(cube_code)
            .build(&ctx.gfx)
            .unwrap(); // Should never fail since draw3d.wgsl is unchanging

        camera.projection.aspect = ctx.gfx.size().0 / ctx.gfx.size().1;
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);

        let camera_buffer =
            ctx.gfx
                .wgpu()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let instance_buffer =
            ctx.gfx
                .wgpu()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&[Instance3d::default(), Instance3d::default()]),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let texture_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let camera_bind_group =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                    label: Some("camera_bind_group"),
                });

        let render_pipeline_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let depth = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Depth32Float, 1., 1., 1);

        let pipeline3d = Canvas3d {
            depth,
            dirty_pipeline: false,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            state: DrawState3d {
                shader: shader.clone(),
            },
            original_state: DrawState3d {
                shader: shader.clone(),
            },
            draws: Vec::default(),
            pipeline: ctx.gfx.wgpu().device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline 3d"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: shader.vs_module().unwrap(), // Should never fail since it's already built
                        entry_point: "vs_main",
                        buffers: &[Vertex3d::desc(), Instance3d::desc()],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: shader.fs_module().unwrap(), // Should never fail since already built
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: ctx.gfx.surface_format(),
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                },
            ),
            instance_buffer,
            target,
            wgpu: ctx.gfx.wgpu.clone(),
            default_shader: shader,
            default_image: graphics::Image::from_color(ctx, 1, 1, Some(Color::WHITE)),
        };

        pipeline3d
    }

    /// Set the `Shader` back to the default shader
    pub fn set_default_shader(&mut self) {
        self.state.shader = self.default_shader.clone();
        self.dirty_pipeline = true;
    }

    /// Set a custom `Shader`
    pub fn set_shader(&mut self, shader: Shader) {
        self.state.shader = shader;
        self.dirty_pipeline = true;
    }

    pub(crate) fn update_pipeline(&mut self, ctx: &mut Context) {
        let camera_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let texture_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });
        let render_pipeline_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        self.pipeline =
            ctx.gfx
                .wgpu()
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: self.state.shader.vs_module().as_ref().unwrap_or(
                            self.original_state.shader.vs_module().as_ref().unwrap_or(
                                self.original_state.shader.vs_module().as_ref().unwrap(),
                            ), // Should always exist
                        ),
                        entry_point: "vs_main",
                        buffers: &[Vertex3d::desc(), Instance3d::desc()],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: self
                            .state
                            .shader
                            .fs_module()
                            .as_ref()
                            .unwrap_or(self.original_state.shader.fs_module().as_ref().unwrap()), // Should always exist since we use original
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: ctx.gfx.surface_format(),
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });
    }

    /// Finish rendering this `Canvas3d`
    pub fn finish(&mut self, ctx: &mut Context, clear_color: Color) -> GameResult {
        if self.dirty_pipeline {
            self.update_pipeline(ctx);
        }

        self.update_instance_data(ctx);

        let draws: Vec<DrawCommand3d> = self.draws.drain(..).collect();

        {
            let depth = self.depth.image(ctx);
            let mut pass =
                ctx.gfx
                    .commands()
                    .unwrap()
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: self.target.wgpu().1,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    graphics::LinearColor::from(clear_color).into(),
                                ),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: depth.wgpu().1,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });
            for (i, draw) in draws.iter().enumerate() {
                let i = i as u32;
                if draw.state.shader != self.state.shader {
                    // self.set_shader(draw.state.shader.clone());
                    // self.update_pipeline(ctx);
                }

                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                pass.set_bind_group(
                    0,
                    draw.mesh.bind_group.as_ref().ok_or(GameError::CustomError(
                        "Bind Group not generated for mesh".to_string(),
                    ))?,
                    &[],
                );
                pass.set_bind_group(1, &self.camera_bind_group, &[]);
                pass.set_vertex_buffer(
                    0,
                    draw.mesh
                        .vert_buffer
                        .as_ref()
                        .ok_or(GameError::CustomError(
                            "Vert Buffer not generated for mesh".to_string(),
                        ))?
                        .slice(..),
                );
                pass.set_index_buffer(
                    draw.mesh
                        .ind_buffer
                        .as_ref()
                        .ok_or(GameError::CustomError(
                            "Ind Buffer not generated for mesh".to_string(),
                        ))?
                        .slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                pass.draw_indexed(0..draw.mesh.indices.len() as u32, 0, i..i + 1);
            }
            std::mem::drop(pass);
        }
        self.draws.clear();
        Ok(())
    }

    pub(crate) fn update_instance_data(&mut self, ctx: &mut Context) {
        let instance_data = self
            .draws
            .iter()
            .map(|x| {
                Instance3d::from_param(&x.param, x.mesh.to_aabb().unwrap_or(Aabb::default()).center)
            })
            .collect::<Vec<_>>();
        ctx.gfx.wgpu().queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
    }

    /// Draw the given `Mesh3d` to the `Canvas3d`
    pub fn draw(&mut self, mesh: Mesh3d, param: DrawParam3d) {
        let mut mesh = mesh;
        mesh.gen_bind_group(self);
        self.draws.push(DrawCommand3d {
            mesh,
            state: self.state.clone(),
            param,
        });
    }

    /// Resize this `Canvas3d` and the `Camera3d` `Projection`
    pub fn resize(
        &mut self,
        width: f32,
        height: f32,
        ctx: &mut Context,
        camera: &mut Camera3dBundle,
    ) {
        camera.projection.resize(width as u32, height as u32);
        self.camera_uniform.update_view_proj(camera);
        ctx.gfx.wgpu().queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Force an `Camera3d` update
    pub fn update_camera(&mut self, ctx: &mut Context, camera: &mut Camera3dBundle) {
        self.camera_uniform.update_view_proj(camera);
        ctx.gfx.wgpu().queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
