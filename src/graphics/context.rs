//!

use super::sampler::SamplerCache;
use crate::{
    conf::{Backend, Conf, FullscreenType, WindowMode},
    error::GameResult,
    filesystem::Filesystem,
    graphics::*,
    GameError,
};
use ::image as imgcrate;
use std::{path::Path, sync::Arc};
use winit::{self, dpi};

#[derive(Debug)]
pub(crate) struct FrameContext {
    pub cmd: wgpu::CommandEncoder,
    pub present: Option<image::Image>,
}

#[derive(Debug)]
pub(crate) struct Pipelines {
    pub copy: wgpu::RenderPipeline,
    pub default: Arc<wgpu::RenderPipeline>,
}

impl Pipelines {
    fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let copy_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../../resources/copy.wgsl").into()),
        });

        Pipelines {
            copy: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &copy_shader,
                    entry_point: "vs_main",
                    buffers: &[],
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
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &copy_shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    }],
                }),
                multiview: None,
            }),
            default: todo!(),
        }
    }
}

/// A concrete graphics context for WGPU rendering.
#[derive(Debug)]
pub struct GraphicsContext {
    pub(crate) window: winit::window::Window,

    pub(crate) instance: wgpu::Instance,
    pub(crate) surface: wgpu::Surface,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    pub(crate) pipelines: Pipelines,
    pub(crate) fcx: Option<FrameContext>,
    pub(crate) sampler_cache: SamplerCache,
}

impl GraphicsContext {
    #[allow(unsafe_code)]
    pub(crate) fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        conf: &Conf,
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

        let window = winit::window::WindowBuilder::new()
            .with_title(conf.window_setup.title.clone())
            .with_inner_size(dpi::PhysicalSize::<f64>::from((
                conf.window_mode.width,
                conf.window_mode.height,
            )))
            .with_resizable(conf.window_mode.resizable)
            .with_visible(conf.window_mode.visible)
            .build(event_loop)?;
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
        let pipelines = Pipelines::new(&device, surface_format);

        let mut this = GraphicsContext {
            window,

            instance,
            surface,
            surface_format,
            device,
            queue,

            pipelines,
            fcx: None,
            sampler_cache: SamplerCache::new(),
        };
        this.set_window_mode(&conf.window_mode)?;
        Ok(this)
    }

    /// Sets the image that will be presented to the screen at the end of the frame.
    pub fn present(&mut self, image: image::Image) -> GameResult {
        if let Some(fcx) = &mut self.fcx {
            fcx.present = Some(image);
            Ok(())
        } else {
            Err(GameError::RenderError(String::from(
                "cannot present outside of a frame",
            )))
        }
    }

    pub(crate) fn begin_frame(&mut self) -> GameResult {
        if let Some(fcx) = &mut self.fcx {
            return Err(GameError::RenderError(String::from(
                "cannot begin a new frame while another frame is still in progress; call end_frame first",
            )));
        }

        self.fcx = Some(FrameContext {
            cmd: self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default()),
            present: None,
        });

        Ok(())
    }

    pub(crate) fn end_frame(&mut self) -> GameResult {
        if let Some(mut fcx) = self.fcx.take() {
            let frame = self.surface.get_current_texture().map_err(|_| {
                GameError::RenderError(String::from("failed to get next swapchain image"))
            })?;
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let present = fcx.present.take();

            {
                let mut present_pass = fcx.cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                if let Some(present) = &present {
                    present_pass.set_pipeline(&self.pipelines.copy);
                    present_pass.set_bind_group(0, present.bind_group.as_ref(), &[]);
                    present_pass.draw(0..3, 0..1);
                }
            }

            self.queue.submit([fcx.cmd.finish()]);

            Ok(())
        } else {
            Err(GameError::RenderError(String::from(
                "cannot end a frame as there was never one in progress; call begin_frame first",
            )))
        }
    }

    pub(crate) fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) {
        // wgpu will handle this for us
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
