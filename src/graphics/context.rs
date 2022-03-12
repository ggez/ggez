//!

use super::{
    draw::DrawUniforms,
    gpu::{
        arc::{ArcBindGroup, ArcBuffer, ArcRenderPipeline, ArcShaderModule},
        bind_group::{BindGroupBuilder, BindGroupCache},
        growing::GrowingBufferArena,
        pipeline::PipelineCache,
        text::TextRenderer,
    },
    image::{Image, ImageFormat},
    mesh::{Mesh, Vertex},
    sampler::{Sampler, SamplerCache},
    shader::Shader,
    text::FontData,
};
use crate::{
    conf::{Backend, Conf, FullscreenType, WindowMode},
    error::GameResult,
    filesystem::Filesystem,
    GameError,
};
use ::image as imgcrate;
use crevice::std430::AsStd430;
use glyph_brush::FontId;
use std::{collections::HashMap, path::Path, sync::Arc};
use typed_arena::Arena as TypedArena;
use winit::{self, dpi};

pub(crate) struct FrameContext {
    pub cmd: wgpu::CommandEncoder,
    pub present: Option<Image>,
    pub arenas: FrameArenas,
    pub frame: wgpu::SurfaceTexture,
    pub frame_view: wgpu::TextureView,
}

#[derive(Default)]
pub(crate) struct FrameArenas {
    pub buffers: TypedArena<ArcBuffer>,
    pub render_pipelines: TypedArena<ArcRenderPipeline>,
    pub bind_groups: TypedArena<ArcBindGroup>,
}

/// A concrete graphics context for WGPU rendering.
#[allow(missing_debug_implementations)]
pub struct GraphicsContext {
    pub(crate) window: winit::window::Window,

    #[allow(unused)]
    pub(crate) instance: wgpu::Instance,
    pub(crate) surface: wgpu::Surface,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    pub(crate) bind_group_cache: BindGroupCache,
    pub(crate) pipeline_cache: PipelineCache,
    pub(crate) sampler_cache: SamplerCache,

    pub(crate) vsync: bool,
    pub(crate) fcx: Option<FrameContext>,
    pub(crate) text: TextRenderer,
    pub(crate) fonts: HashMap<String, FontId>,
    pub(crate) staging_belt: wgpu::util::StagingBelt,
    pub(crate) uniform_arena: GrowingBufferArena,
    pub(crate) local_pool: futures::executor::LocalPool,
    pub(crate) local_spawner: futures::executor::LocalSpawner,

    pub(crate) draw_shader: ArcShaderModule,
    pub(crate) instance_shader: ArcShaderModule,
    pub(crate) text_shader: ArcShaderModule,
    pub(crate) copy_shader: ArcShaderModule,
    pub(crate) rect_mesh: Option<Arc<Mesh>>,
    pub(crate) white_image: Option<Image>,
}

impl GraphicsContext {
    #[allow(unsafe_code)]
    pub(crate) fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        conf: &Conf,
        filesystem: &mut Filesystem,
    ) -> GameResult<Self> {
        let instance = wgpu::Instance::new(match conf.backend {
            Backend::Primary => wgpu::Backends::PRIMARY,
            Backend::Secondary => wgpu::Backends::SECONDARY,
            Backend::Vulkan => wgpu::Backends::VULKAN,
            Backend::Metal => wgpu::Backends::METAL,
            Backend::Dx12 => wgpu::Backends::DX12,
            Backend::Dx11 => wgpu::Backends::DX11,
            Backend::Gl => wgpu::Backends::GL,
            Backend::BrowserWebGpu => wgpu::Backends::BROWSER_WEBGPU,
        });

        let mut window_builder = winit::window::WindowBuilder::new()
            .with_title(conf.window_setup.title.clone())
            .with_inner_size(dpi::PhysicalSize::<f64>::from((
                conf.window_mode.width,
                conf.window_mode.height,
            )))
            .with_resizable(conf.window_mode.resizable)
            .with_visible(conf.window_mode.visible);

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            window_builder = window_builder.with_drag_and_drop(false);
        }

        window_builder = if !conf.window_setup.icon.is_empty() {
            let icon = load_icon(conf.window_setup.icon.as_ref(), filesystem)?;
            window_builder.with_window_icon(Some(icon))
        } else {
            window_builder
        };

        let window = window_builder.build(&event_loop)?;
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .ok_or_else(|| {
            GameError::RenderError(String::from("failed to find suitable graphics adapter"))
        })?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))?;

        let surface_format = surface.get_preferred_format(&adapter).unwrap(/* invariant */);
        let size = window.inner_size();
        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: if conf.window_setup.vsync {
                    wgpu::PresentMode::Fifo
                } else {
                    wgpu::PresentMode::Mailbox
                },
            },
        );

        let bind_group_cache = BindGroupCache::new();
        let pipeline_cache = PipelineCache::new();
        let sampler_cache = SamplerCache::new();

        let text = TextRenderer::new(&device);

        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let uniform_arena = GrowingBufferArena::new(
            &device,
            device.limits().min_uniform_buffer_offset_alignment as u64,
            wgpu::BufferDescriptor {
                label: None,
                size: 4096 * DrawUniforms::std430_size_static() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let draw_shader =
            ArcShaderModule::new(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader/draw.wgsl").into()),
            }));

        let instance_shader =
            ArcShaderModule::new(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader/instance.wgsl").into()),
            }));

        let text_shader =
            ArcShaderModule::new(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader/text.wgsl").into()),
            }));

        let copy_shader =
            ArcShaderModule::new(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader/copy.wgsl").into()),
            }));

        let mut this = GraphicsContext {
            window,

            instance,
            surface,
            surface_format,
            device,
            queue,

            bind_group_cache,
            pipeline_cache,
            sampler_cache,

            vsync: conf.window_setup.vsync,
            fcx: None,
            text,
            fonts: HashMap::new(),
            staging_belt,
            uniform_arena,
            local_pool,
            local_spawner,

            draw_shader,
            instance_shader,
            text_shader,
            copy_shader,
            rect_mesh: None,
            white_image: None,
        };

        this.set_window_mode(&conf.window_mode)?;

        this.rect_mesh = Some(Arc::new(Mesh::new(
            &this,
            &[
                Vertex {
                    position: [0., 0.],
                    uv: [0., 0.],
                    color: [1.; 4],
                },
                Vertex {
                    position: [1., 0.],
                    uv: [1., 0.],
                    color: [1.; 4],
                },
                Vertex {
                    position: [0., 1.],
                    uv: [0., 1.],
                    color: [1.; 4],
                },
                Vertex {
                    position: [1., 1.],
                    uv: [1., 1.],
                    color: [1.; 4],
                },
            ],
            &[0, 2, 1, 2, 3, 1],
        )));

        this.white_image = Some(Image::from_pixels(
            &this,
            &[255, 255, 255, 255],
            ImageFormat::Rgba8Unorm,
            1,
            1,
        ));

        Ok(this)
    }

    /// Sets the image that will be presented to the screen at the end of the frame.
    pub fn present(&mut self, image: &Image) -> GameResult {
        if let Some(fcx) = &mut self.fcx {
            fcx.present = Some(image.clone());
            Ok(())
        } else {
            Err(GameError::RenderError(String::from(
                "cannot present outside of a frame",
            )))
        }
    }

    /// Adds a new `font` with a given `name`.
    #[allow(unused_results)]
    pub fn add_font(&mut self, name: &str, font: FontData) {
        let id = self.text.glyph_brush.add_font(font.font);
        self.fonts.insert(name.to_string(), id);
    }

    pub(crate) fn begin_frame(&mut self) -> GameResult {
        if self.fcx.is_some() {
            return Err(GameError::RenderError(String::from(
                "cannot begin a new frame while another frame is still in progress; call end_frame first",
            )));
        }

        let frame = self.surface.get_current_texture().map_err(|_| {
            GameError::RenderError(String::from("failed to get next swapchain image"))
        })?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.fcx = Some(FrameContext {
            cmd: self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default()),
            present: None,
            arenas: FrameArenas::default(),
            frame,
            frame_view,
        });

        self.uniform_arena.free();
        self.text.free();

        Ok(())
    }

    pub(crate) fn end_frame(&mut self) -> GameResult {
        if let Some(mut fcx) = self.fcx.take() {
            let present = fcx.present.take();
            if let Some(present) = present {
                let mut present_pass = fcx.cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &fcx.frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                let sampler = self
                    .sampler_cache
                    .get(&self.device, Sampler::linear_clamp());

                let (bind, layout) = BindGroupBuilder::new()
                    .image(&present.view, wgpu::ShaderStages::FRAGMENT)
                    .sampler(&sampler, wgpu::ShaderStages::FRAGMENT)
                    .create(&self.device, &mut self.bind_group_cache);

                let layout = self.pipeline_cache.layout(&self.device, &[layout]);
                let copy = self.pipeline_cache.render_pipeline(
                    &self.device,
                    &layout,
                    Shader {
                        fragment: self.copy_shader.clone(),
                        fs_entry: "fs_main".into(),
                    }
                    .info(
                        self.copy_shader.clone(),
                        1,
                        self.surface_format,
                        None,
                        false,
                        false,
                        wgpu::PrimitiveTopology::TriangleList,
                        Vertex::layout(),
                    ),
                );

                let copy = fcx.arenas.render_pipelines.alloc(copy);
                let bind = fcx.arenas.bind_groups.alloc(bind);

                present_pass.set_pipeline(copy);
                present_pass.set_bind_group(0, bind, &[]);
                present_pass.draw(0..3, 0..1);
            }

            self.staging_belt.finish();
            self.queue.submit([fcx.cmd.finish()]);
            fcx.frame.present();

            use futures::task::SpawnExt;
            self.local_spawner.spawn(self.staging_belt.recall())?;
            self.local_pool.run_until_stalled();

            Ok(())
        } else {
            Err(GameError::RenderError(String::from(
                "cannot end a frame as there was never one in progress; call begin_frame first",
            )))
        }
    }

    pub(crate) fn resize(&mut self, _new_size: dpi::PhysicalSize<u32>) {
        let size = self.window.inner_size();
        self.device.poll(wgpu::Maintain::Wait);
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );
    }

    pub(crate) fn set_window_mode(&mut self, mode: &WindowMode) -> GameResult {
        let window = &mut self.window;

        // TODO LATER: find out if single-dimension constraints are possible?
        let min_dimensions = if mode.min_width > 0.0 && mode.min_height > 0.0 {
            Some(dpi::PhysicalSize {
                width: f64::from(mode.min_width),
                height: f64::from(mode.min_height),
            })
        } else {
            None
        };
        window.set_min_inner_size(min_dimensions);

        let max_dimensions = if mode.max_width > 0.0 && mode.max_height > 0.0 {
            Some(dpi::PhysicalSize {
                width: f64::from(mode.max_width),
                height: f64::from(mode.max_height),
            })
        } else {
            None
        };
        window.set_max_inner_size(max_dimensions);
        window.set_visible(mode.visible);

        match mode.fullscreen_type {
            FullscreenType::Windowed => {
                window.set_fullscreen(None);
                window.set_decorations(!mode.borderless);
                window.set_inner_size(dpi::PhysicalSize {
                    width: f64::from(mode.width),
                    height: f64::from(mode.height),
                });
                window.set_resizable(mode.resizable);
                window.set_maximized(mode.maximized);
            }
            FullscreenType::True => {
                if let Some(monitor) = window.current_monitor() {
                    let v_modes = monitor.video_modes();
                    // try to find a video mode with a matching resolution
                    let mut match_found = false;
                    for v_mode in v_modes {
                        let size = v_mode.size();
                        if (size.width, size.height) == (mode.width as u32, mode.height as u32) {
                            window
                                .set_fullscreen(Some(winit::window::Fullscreen::Exclusive(v_mode)));
                            match_found = true;
                            break;
                        }
                    }
                    if !match_found {
                        return Err(GameError::WindowError(format!(
                            "resolution {}x{} is not supported by this monitor",
                            mode.width, mode.height
                        )));
                    }
                }
            }
            FullscreenType::Desktop => {
                window.set_fullscreen(None);
                window.set_decorations(false);
                if let Some(monitor) = window.current_monitor() {
                    window.set_inner_size(monitor.size());
                    window.set_outer_position(monitor.position());
                }
            }
        }

        let size = window.inner_size();
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: size.width,
                height: size.height,
                present_mode: if self.vsync {
                    wgpu::PresentMode::Fifo
                } else {
                    wgpu::PresentMode::Mailbox
                },
            },
        );

        Ok(())
    }
}

// This is kinda awful 'cause it copies a couple times,
// but still better than
// having `winit` try to do the image loading for us.
// see https://github.com/tomaka/winit/issues/661
pub(crate) fn load_icon(
    icon_file: &Path,
    filesystem: &mut Filesystem,
) -> GameResult<winit::window::Icon> {
    use imgcrate::GenericImageView;
    use std::io::Read;
    use winit::window::Icon;

    let mut buf = Vec::new();
    let mut reader = filesystem.open(icon_file)?;
    let _ = reader.read_to_end(&mut buf)?;
    let i = imgcrate::load_from_memory(&buf)?;
    let image_data = i.to_rgba8();
    Icon::from_rgba(image_data.to_vec(), i.width(), i.height()).map_err(|e| {
        let msg = format!("Could not load icon: {:?}", e);
        GameError::ResourceLoadError(msg)
    })
}
