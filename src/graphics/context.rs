use std::cell::RefCell;
use std::rc::Rc;

use gfx::traits::FactoryExt;
use gfx::Factory;
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
#[rustfmt::skip]
use ::image as imgcrate;
use winit::{self, dpi};

use crate::conf::{FullscreenType, WindowMode, WindowSetup};
use crate::context::DebugId;
use crate::filesystem::Filesystem;
use crate::graphics::*;

use crate::error::GameResult;

// Define the input struct for our MSAA resolve shader.
gfx_defines! {
    constant Fragments {
        fragments: i32 = "u_frags",
    }
}

/// A structure that contains graphics state.
/// For instance,
/// window info, DPI, rendering pipeline state, etc.
///
/// As an end-user you shouldn't ever have to touch this.
pub(crate) struct GraphicsContextGeneric<B>
where
    B: BackendSpec,
{
    shader_globals: Globals,
    pub(crate) projection: Matrix4,
    pub(crate) white_image: ImageGeneric<B>,
    pub(crate) screen_rect: Rect,
    pub(crate) to_rgba8_buffer: gfx::handle::Buffer<B::Resources, u8>,
    color_format: gfx::format::Format,
    depth_format: gfx::format::Format,
    srgb: bool,

    pub(crate) backend_spec: B,
    pub(crate) window: glutin::WindowedContext<glutin::PossiblyCurrent>,
    pub(crate) multisample_samples: u8,
    pub(crate) device: Box<B::Device>,
    pub(crate) factory: Box<B::Factory>,
    pub(crate) encoder: gfx::Encoder<B::Resources, B::CommandBuffer>,
    pub(crate) screen_render_target: gfx::handle::RawRenderTargetView<B::Resources>,
    #[allow(dead_code)]
    pub(crate) depth_view: gfx::handle::RawDepthStencilView<B::Resources>,

    pub(crate) data: pipe::Data<B::Resources>,
    pub(crate) quad_slice: gfx::Slice<B::Resources>,
    pub(crate) quad_vertex_buffer: gfx::handle::Buffer<B::Resources, Vertex>,

    pub(crate) default_sampler_info: texture::SamplerInfo,
    pub(crate) samplers: SamplerCache<B>,

    default_shader: ShaderId,
    pub(crate) resolve_shader: ShaderGeneric<B, Fragments>,
    pub(crate) current_shader: Rc<RefCell<Option<ShaderId>>>,
    pub(crate) shaders: Vec<Box<dyn ShaderHandle<B>>>,

    pub(crate) glyph_brush: Rc<RefCell<GlyphBrush<DrawParam>>>,
    pub(crate) glyph_cache: ImageGeneric<B>,
    pub(crate) glyph_state: Rc<RefCell<spritebatch::SpriteBatch>>,
}

impl<B> fmt::Debug for GraphicsContextGeneric<B>
where
    B: BackendSpec,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<GraphicsContext: {:p}>", self)
    }
}

/// A concrete graphics context for GL rendering.
pub(crate) type GraphicsContext = GraphicsContextGeneric<GlBackendSpec>;

impl GraphicsContextGeneric<GlBackendSpec> {
    /// Create a new GraphicsContext
    pub(crate) fn new(
        filesystem: &mut Filesystem,
        events_loop: &winit::event_loop::EventLoop<()>,
        window_setup: &WindowSetup,
        window_mode: WindowMode,
        backend: GlBackendSpec,
        debug_id: DebugId,
    ) -> GameResult<Self> {
        let srgb = window_setup.srgb;
        let color_format = if srgb {
            gfx::format::Format(
                gfx::format::SurfaceType::R8_G8_B8_A8,
                gfx::format::ChannelType::Srgb,
            )
        } else {
            gfx::format::Format(
                gfx::format::SurfaceType::R8_G8_B8_A8,
                gfx::format::ChannelType::Unorm,
            )
        };
        let depth_format = gfx::format::Format(
            gfx::format::SurfaceType::D24_S8,
            gfx::format::ChannelType::Unorm,
        );

        // WINDOW SETUP
        let gl_builder = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(
                backend.api(),
                backend.version_tuple(),
            ))
            .with_gl_profile(glutin::GlProfile::Core)
            .with_multisampling(match window_setup.samples.into() {
                // Fix for https://github.com/ggez/ggez/issues/552
                // 1 isn't multisampling but glutin wants a 0 to disable it
                1 => 0,
                n => u16::from(n),
            })
            // 24 color bits, 8 alpha bits
            .with_pixel_format(24, 8)
            .with_vsync(window_setup.vsync);

        let window_size = dpi::PhysicalSize::<f64>::from((window_mode.width, window_mode.height));
        let mut window_builder = winit::window::WindowBuilder::new()
            .with_title(window_setup.title.clone())
            .with_inner_size(window_size)
            .with_resizable(window_mode.resizable)
            .with_visible(window_mode.visible);

        // We need to disable drag-and-drop on windows for multithreaded stuff like cpal to work.
        // See winit bug here: https://github.com/rust-windowing/winit/pull/1524
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            window_builder = window_builder.with_drag_and_drop(false);
        }

        window_builder = if !window_setup.icon.is_empty() {
            let icon = load_icon(window_setup.icon.as_ref(), filesystem)?;
            window_builder.with_window_icon(Some(icon))
        } else {
            window_builder
        };

        let (window, device, mut factory, screen_render_target, depth_view) = backend.init(
            window_builder,
            gl_builder,
            events_loop,
            color_format,
            depth_format,
        )?;

        // see winit #548 about DPI.
        // We basically ignore it and if it's wrong, that's a winit bug
        // since we have no good control over it.
        {
            // Log a bunch of OpenGL state info pulled out of winit and gfx
            let scale_factor = window.window().scale_factor();
            let dpi::LogicalSize::<f32> {
                width: w,
                height: h,
            } = window.window().outer_size().to_logical(scale_factor);
            let dpi::LogicalSize::<f32> {
                width: dw,
                height: dh,
            } = window.window().inner_size().to_logical(scale_factor);
            debug!(
                "Window created, desired size {}x{}, scale factor {}.",
                window_mode.width, window_mode.height, scale_factor
            );
            let (major, minor) = backend.version_tuple();
            debug!(
                "  Window logical outer size: {}x{}, logical drawable size: {}x{}",
                w, h, dw, dh
            );
            let device_info = backend.info(&device);
            debug!(
                "  Asked for   : {:?} {}.{} Core, vsync: {}",
                backend.api(),
                major,
                minor,
                window_setup.vsync
            );
            debug!("  Actually got: {}", device_info);
        }

        // GFX SETUP
        let mut encoder = GlBackendSpec::encoder(&mut factory);

        let blend_modes = [
            BlendMode::Alpha,
            BlendMode::Add,
            BlendMode::Subtract,
            BlendMode::Invert,
            BlendMode::Multiply,
            BlendMode::Replace,
            BlendMode::Lighten,
            BlendMode::Darken,
            BlendMode::Premultiplied,
        ];
        let multisample_samples = window_setup.samples.into();
        let (vs_text, fs_text, fs_resolve_text) = backend.shaders();
        let (shader, draw) = create_shader(
            vs_text,
            fs_text,
            EmptyConst,
            "Empty",
            &mut encoder,
            &mut factory,
            multisample_samples,
            Some(&blend_modes[..]),
            color_format,
            debug_id,
        )?;
        let (mut resolve_shader, mut resolve_draw) = create_shader(
            vs_text,
            fs_resolve_text,
            Fragments { fragments: 1 },
            "Fragments",
            &mut encoder,
            &mut factory,
            1,
            Some(&[BlendMode::Replace]),
            color_format,
            debug_id,
        )?;

        resolve_shader.id = 1;
        resolve_draw.set_blend_mode(BlendMode::Replace)?;

        let rect_inst_props = factory.create_buffer(
            1,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::memory::Bind::SHADER_RESOURCE,
        )?;

        let (quad_vertex_buffer, mut quad_slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTS, &QUAD_INDICES[..]);

        quad_slice.instances = Some((1, 0));

        let to_rgba8_buffer = factory.create_download_buffer::<u8>(1)?;

        let globals_buffer = factory.create_constant_buffer(1);
        let mut samplers: SamplerCache<GlBackendSpec> = SamplerCache::new();
        let sampler_info =
            texture::SamplerInfo::new(texture::FilterMethod::Bilinear, texture::WrapMode::Clamp);
        let sampler = samplers.get_or_insert(sampler_info, &mut factory);
        let white_image = ImageGeneric::make_raw(
            &mut factory,
            &sampler_info,
            1,
            1,
            &[255, 255, 255, 255],
            color_format,
            debug_id,
        )?;
        let texture = white_image.texture.clone();
        let typed_thingy = backend.raw_to_typed_shader_resource(texture);

        let data = pipe::Data {
            vbuf: quad_vertex_buffer.clone(),
            tex: (typed_thingy, sampler),
            rect_instance_properties: rect_inst_props,
            globals: globals_buffer,
            out: screen_render_target.clone(),
        };

        // Glyph cache stuff.
        let font_vec = glyph_brush::ab_glyph::FontArc::try_from_slice(Font::default_font_bytes())
            .expect("Invalid default font bytes, should never happen");
        let glyph_brush = GlyphBrushBuilder::using_font(font_vec).build();
        let (glyph_cache_width, glyph_cache_height) = glyph_brush.texture_dimensions();
        let initial_contents = vec![
            255;
            4 * usize::try_from(glyph_cache_width).unwrap()
                * usize::try_from(glyph_cache_height).unwrap()
        ];
        let glyph_cache = ImageGeneric::make_raw(
            &mut factory,
            &sampler_info,
            glyph_cache_width.try_into().unwrap(),
            glyph_cache_height.try_into().unwrap(),
            &initial_contents,
            color_format,
            debug_id,
        )?;
        let glyph_state = Rc::new(RefCell::new(spritebatch::SpriteBatch::new(
            glyph_cache.clone(),
        )));

        // Set initial uniform values
        let left = 0.0;
        let right = window_mode.width;
        let top = 0.0;
        let bottom = window_mode.height;
        let initial_projection = Matrix4::IDENTITY; // not the actual initial projection matrix, just placeholder
        let globals = Globals {
            mvp_matrix: initial_projection.to_cols_array_2d(),
        };

        let mut gfx = Self {
            shader_globals: globals,
            projection: initial_projection,
            white_image,
            screen_rect: Rect::new(left, top, right - left, bottom - top),
            to_rgba8_buffer,
            color_format,
            depth_format,
            srgb,

            backend_spec: backend,
            window,
            multisample_samples,
            device: Box::new(device as <GlBackendSpec as BackendSpec>::Device),
            factory: Box::new(factory as <GlBackendSpec as BackendSpec>::Factory),
            encoder,
            screen_render_target,
            depth_view,

            data,
            quad_slice,
            quad_vertex_buffer,

            default_sampler_info: sampler_info,
            samplers,

            default_shader: shader.shader_id(),
            resolve_shader,
            current_shader: Rc::new(RefCell::new(None)),
            shaders: vec![draw, resolve_draw],

            glyph_brush: Rc::new(RefCell::new(glyph_brush)),
            glyph_cache,
            glyph_state,
        };
        gfx.set_window_mode(window_mode)?;

        // Calculate and apply the actual initial projection matrix
        let w = window_mode.width;
        let h = window_mode.height;
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
        };
        gfx.set_projection_rect(rect);
        gfx.set_global_mvp(Matrix4::IDENTITY)?;
        Ok(gfx)
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

impl<B> GraphicsContextGeneric<B>
where
    B: BackendSpec + 'static,
{
    /// Sends the current value of the graphics context's shader globals
    /// to the graphics card.
    pub(crate) fn update_globals(&mut self) -> GameResult {
        self.encoder
            .update_buffer(&self.data.globals, &[self.shader_globals], 0)?;
        Ok(())
    }

    /// Sets the shader MVP matrix to the current projection multiplied by
    /// the given matrix, and updates the uniform buffer.
    pub(crate) fn set_global_mvp(&mut self, matrix: Matrix4) -> GameResult {
        let mvp = self.projection * matrix;
        self.shader_globals.mvp_matrix = mvp.to_cols_array_2d();
        self.update_globals()
    }

    /// Converts the given `DrawParam` into an `InstanceProperties` object and
    /// sends it to the graphics card at the front of the instance buffer.
    pub(crate) fn update_instance_properties(&mut self, draw_params: DrawParam) -> GameResult {
        let mut new_draw_params = draw_params;
        new_draw_params.color = draw_params.color;
        let properties = new_draw_params.to_instance_properties(self.srgb);
        self.encoder
            .update_buffer(&self.data.rect_instance_properties, &[properties], 0)?;
        Ok(())
    }

    /// Draws with the current encoder, slice, and pixel shader. Prefer calling
    /// this method from `Drawables` so that the pixel shader gets used
    pub(crate) fn draw(&mut self, slice: Option<&gfx::Slice<B::Resources>>) -> GameResult {
        let slice = slice.unwrap_or(&self.quad_slice);
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &self.shaders[id];

        shader_handle.draw(&mut self.encoder, slice, &self.data)?;
        Ok(())
    }

    /// Sets the blend mode of the active shader
    pub(crate) fn set_blend_mode(&mut self, mode: BlendMode) -> GameResult {
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &mut self.shaders[id];
        shader_handle.set_blend_mode(mode)
    }

    /// Gets the current blend mode of the active shader
    pub(crate) fn blend_mode(&self) -> BlendMode {
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &self.shaders[id];
        shader_handle.blend_mode()
    }

    /// Shortcut function to set the projection matrix to an
    /// orthographic projection based on the given `Rect`.
    ///
    /// Call `update_globals()` to apply it after calling this.
    pub(crate) fn set_projection_rect(&mut self, rect: Rect) {
        /// Creates an orthographic projection matrix.
        /// Because nalgebra gets frumple when you try to make
        /// one that is upside-down.
        /// This is fixed now (issue here: https://github.com/rustsim/nalgebra/issues/365)
        /// but removing this kinda isn't worth it.
        fn ortho(
            left: f32,
            right: f32,
            top: f32,
            bottom: f32,
            far: f32,
            near: f32,
        ) -> [[f32; 4]; 4] {
            let c0r0 = 2.0 / (right - left);
            let c0r1 = 0.0;
            let c0r2 = 0.0;
            let c0r3 = 0.0;

            let c1r0 = 0.0;
            let c1r1 = 2.0 / (top - bottom);
            let c1r2 = 0.0;
            let c1r3 = 0.0;

            let c2r0 = 0.0;
            let c2r1 = 0.0;
            let c2r2 = -2.0 / (far - near);
            let c2r3 = 0.0;

            let c3r0 = -(right + left) / (right - left);
            let c3r1 = -(top + bottom) / (top - bottom);
            let c3r2 = -(far + near) / (far - near);
            let c3r3 = 1.0;

            // our matrices are column-major, so here we are.
            [
                [c0r0, c0r1, c0r2, c0r3],
                [c1r0, c1r1, c1r2, c1r3],
                [c2r0, c2r1, c2r2, c2r3],
                [c3r0, c3r1, c3r2, c3r3],
            ]
        }

        self.screen_rect = rect;
        self.projection = Matrix4::from_cols_array_2d(&ortho(
            rect.x,
            rect.x + rect.w,
            rect.y,
            rect.y + rect.h,
            -1.0,
            1.0,
        ));
    }

    /// Sets the raw projection matrix to the given Matrix.
    ///
    /// Call `update_globals()` to apply after calling this.
    pub(crate) fn set_projection(&mut self, mat: Matrix4) {
        self.projection = mat;
    }

    /// Gets a copy of the raw projection matrix.
    pub(crate) fn projection(&self) -> Matrix4 {
        self.projection
    }

    /// Sets window mode from a WindowMode object.
    pub(crate) fn set_window_mode(&mut self, mode: WindowMode) -> GameResult {
        let window = self.window.window();

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

    /// Communicates changes in the viewport size between glutin and gfx.
    ///
    /// Also replaces gfx.screen_render_target and gfx.depth_view,
    /// so it may cause squirrelliness to
    /// happen with canvases or other things that touch it.
    pub(crate) fn resize_viewport(&mut self) {
        if let Some((cv, dv)) = self.backend_spec.resize_viewport(
            &self.screen_render_target,
            &self.depth_view,
            self.color_format(),
            self.depth_format(),
            &self.window,
        ) {
            self.screen_render_target = cv;
            self.depth_view = dv;
        }
    }

    /// Returns the screen color format used by the context.
    pub(crate) fn color_format(&self) -> gfx::format::Format {
        self.color_format
    }

    /// Returns the screen depth format used by the context.
    ///
    pub(crate) fn depth_format(&self) -> gfx::format::Format {
        self.depth_format
    }

    /// Simple shortcut to check whether the context's color
    /// format is SRGB or not.
    pub(crate) fn is_srgb(&self) -> bool {
        self.color_format().1 == gfx::format::ChannelType::Srgb
    }
}
