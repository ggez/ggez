use crate::{
    context::HasMut,
    glam::*,
    graphics::{
        self, Aabb, CameraUniform, Color, DrawParam3d, DrawState3d, Instance3d, Mesh3d, Shader,
        Vertex3d, WgpuContext,
    },
    Context, GameError, GameResult,
};
use std::sync::Arc;

use wgpu::util::DeviceExt;

use super::{Camera3d, Drawable3d, GraphicsContext};

#[derive(Clone, Debug)]
pub(crate) struct DrawCommand3d {
    pub(crate) mesh: Mesh3d, // Maybe take a reference instead
    pub(crate) param: DrawParam3d,
    pub(crate) pipeline_id: usize,
}

/// A 3d Canvas for rendering 3d objects
#[derive(Debug)]
pub struct Canvas3d {
    pub(crate) wgpu: Arc<WgpuContext>,
    pub(crate) default_shader: Shader,
    pub(crate) default_image: graphics::Image,
    pub(crate) draws: Vec<DrawCommand3d>,
    pub(crate) state: DrawState3d,
    pub(crate) original_state: DrawState3d,
    pub(crate) pipelines: Vec<(wgpu::RenderPipeline, DrawState3d)>,
    pub(crate) depth: graphics::Image,
    pub(crate) camera_uniform: CameraUniform,
    pub(crate) instance_buffer: Option<wgpu::Buffer>,
    pub(crate) camera_buffer: wgpu::Buffer,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) target: graphics::Image,
    pub(crate) clear_color: graphics::Color,
    pub(crate) curr_sampler: graphics::Sampler,
}

impl Canvas3d {
    /// Create a `Canvas3d` from a frame. This will fill the whole window
    pub fn from_frame(
        gfx: &mut impl HasMut<GraphicsContext>,
        camera: &mut Camera3d,
        clear_color: Color,
    ) -> Self {
        let gfx = gfx.retrieve_mut();
        Self::new(gfx, camera, gfx.frame().clone(), clear_color)
    }

    /// Createa a `Canvas3d` from an image to render to
    pub fn from_image(
        gfx: &mut impl HasMut<GraphicsContext>,
        camera: &mut Camera3d,
        image: graphics::Image,
        clear_color: Color,
    ) -> Self {
        let gfx = gfx.retrieve_mut();
        Self::new(gfx, camera, image, clear_color)
    }

    pub(crate) fn new(
        gfx: &mut impl HasMut<GraphicsContext>,
        camera: &mut Camera3d,
        target: graphics::Image,
        clear_color: Color,
    ) -> Self {
        let gfx = gfx.retrieve_mut();
        let cube_code = include_str!("shader/draw3d.wgsl");
        let shader = graphics::ShaderBuilder::from_code(cube_code)
            .build(gfx)
            .unwrap(); // Should never fail since draw3d.wgsl is unchanging

        camera.projection.aspect = gfx.size().0 / gfx.size().1;
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);

        let camera_buffer =
            gfx.wgpu()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout =
            gfx.wgpu()
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
            gfx.wgpu()
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

        let camera_bind_group = gfx
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
            gfx.wgpu()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let depth = graphics::Image::new_canvas_image(
            gfx,
            graphics::ImageFormat::Depth32Float,
            target.width(),
            target.height(),
            1,
        );

        Canvas3d {
            clear_color,
            curr_sampler: graphics::Sampler::default(),
            depth,
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
            pipelines: vec![(
                gfx.wgpu()
                    .device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                                format: gfx.surface_format(),
                                blend: Some(wgpu::BlendState {
                                    color: wgpu::BlendComponent::REPLACE,
                                    alpha: wgpu::BlendComponent::REPLACE,
                                }),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    }),
                DrawState3d {
                    shader: shader.clone(),
                },
            )],
            instance_buffer: None,
            target,
            wgpu: gfx.wgpu.clone(),
            default_shader: shader,
            default_image: graphics::Image::from_color(gfx, 1, 1, Some(Color::WHITE)),
        }
    }

    /// Set the `Shader` back to the default shader
    pub fn set_default_shader(&mut self) {
        self.state.shader = self.default_shader.clone();
    }

    /// Set a custom `Shader`
    pub fn set_shader(&mut self, shader: Shader) {
        self.state.shader = shader;
    }

    pub(crate) fn update_pipeline(&mut self, gfx: &mut impl HasMut<GraphicsContext>) {
        let gfx = gfx.retrieve_mut();
        let camera_bind_group_layout =
            gfx.wgpu()
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
            gfx.wgpu()
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
            gfx.wgpu()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        self.pipelines.push((
            gfx.wgpu()
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: self.state.shader.vs_module().clone().as_ref().unwrap_or(
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
                            .clone()
                            .fs_module()
                            .as_ref()
                            .unwrap_or(self.original_state.shader.fs_module().as_ref().unwrap()), // Should always exist since we use original
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gfx.surface_format(),
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                }),
            self.state.clone(),
        ));
    }

    /// Finish rendering this `Canvas3d`
    pub fn finish(&mut self, gfx: &mut impl HasMut<GraphicsContext>) -> GameResult {
        self.update_instance_data(gfx);
        let gfx = gfx.retrieve_mut();

        let draws: Vec<DrawCommand3d> = self.draws.drain(..).collect();

        {
            let mut pass = gfx
                .commands()
                .unwrap()
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: self.target.wgpu().1,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                graphics::LinearColor::from(self.clear_color).into(),
                            ),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: self.depth.wgpu().1,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });
            for (i, draw) in draws.iter().enumerate() {
                let i = i as u32;
                pass.set_pipeline(&self.pipelines[draw.pipeline_id].0);
                pass.set_vertex_buffer(1, self.instance_buffer.as_ref().unwrap().slice(..)); // Will always exist because of update_instance_data
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
        }
        self.draws.clear();
        Ok(())
    }

    pub(crate) fn update_instance_data(&mut self, gfx: &mut impl HasMut<GraphicsContext>) {
        let gfx = gfx.retrieve_mut();
        let instance_data = self
            .draws
            .iter()
            .map(|x| {
                if let Some(offset) = x.param.offset {
                    Instance3d::from_param(&x.param, offset)
                } else {
                    Instance3d::from_param(
                        &x.param,
                        x.mesh.to_aabb().unwrap_or(Aabb::default()).center,
                    )
                }
            })
            .collect::<Vec<_>>();
        self.instance_buffer = Some(gfx.wgpu().device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            },
        ));
    }

    /// Draw any thing that implements the `Drawable3d`
    pub fn draw(
        &mut self,
        gfx: &mut impl HasMut<GraphicsContext>,
        drawable: &impl Drawable3d,
        param: impl Into<DrawParam3d>,
    ) {
        drawable.draw(gfx, self, param);
    }
    /// Draw the given `Mesh3d` to the `Canvas3d`
    pub fn draw_mesh(
        &mut self,
        gfx: &mut impl HasMut<GraphicsContext>,
        mesh: Mesh3d,
        param: DrawParam3d,
    ) {
        // This is pretty 'hacky' but I didn't have any better ideas that wouldn't require users to mess with lifetimes
        let mut id = 0;
        let states: Vec<DrawState3d> = self.pipelines.iter().map(|x| x.1.clone()).collect();
        for (i, state) in states.iter().enumerate() {
            if state.shader == self.state.shader {
                id = i;
            }

            if i == self.pipelines.len() - 1 {
                id = i + 1;
                self.update_pipeline(gfx);
            }
        }
        let mut mesh = mesh;
        mesh.gen_bind_group(self, id, self.curr_sampler);
        self.draws.push(DrawCommand3d {
            mesh,
            param,
            pipeline_id: id,
        });
    }

    /// Resize this `Canvas3d` and the `Camera3d` `Projection`
    pub fn resize(&mut self, width: f32, height: f32, ctx: &mut Context, camera: &mut Camera3d) {
        camera.projection.resize(width as u32, height as u32);
        self.camera_uniform.update_view_proj(camera);
        ctx.gfx.wgpu().queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Force an `Camera3d` update
    pub fn update_camera(&mut self, ctx: &mut Context, camera: &mut Camera3d) {
        self.camera_uniform.update_view_proj(camera);
        ctx.gfx.wgpu().queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Set the sampler used for textures
    pub fn set_sampler(&mut self, sampler: graphics::Sampler) {
        self.curr_sampler = sampler;
    }

    /// Set the sampler back to the default for textures
    pub fn set_default_sampler(&mut self) {
        self.curr_sampler = graphics::Sampler::default();
    }
}
