use std::cell::RefCell;
use std::rc::Rc;

use gfx::traits::FactoryExt;
use gfx::Factory;
use gfx_device_gl;
use gfx_glyph::{GlyphBrush, GlyphBrushBuilder};
use gfx_window_glutin;
use glutin;

use conf::{WindowMode, WindowSetup, FullscreenType, MonitorId};
use context::DebugId;
use graphics::*;

use GameResult;

/// A structure that contains graphics state.
/// For instance, 
/// window info, DPI, rendering pipeline state, etc.
///
/// As an end-user you shouldn't ever have to touch this.
pub(crate) struct GraphicsContextGeneric<B, C>
where
    B: BackendSpec<SurfaceType = C>,
    C: gfx::format::Formatted,
{
    shader_globals: Globals,
    projection: Matrix4,
    pub(crate) modelview_stack: Vec<Matrix4>,
    pub(crate) white_image: Image,
    pub(crate) screen_rect: Rect,
    pub(crate) color_format: gfx::format::Format,
    pub(crate) depth_format: gfx::format::Format,

    // TODO: is this needed?
    #[allow(unused)]
    pub(crate) backend_spec: B,
    pub(crate) window: glutin::GlWindow,
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
    pub(crate) current_shader: Rc<RefCell<Option<ShaderId>>>,
    pub(crate) shaders: Vec<Box<ShaderHandle<B>>>,

    pub(crate) glyph_brush: GlyphBrush<'static, B::Resources, B::Factory>,

    // TODO: there are temporary: need more winit functionality.
    // winit needs ability to get available/primary monitors from a window reference,
    // without having a reference to events loop.
    available_monitors: Vec<glutin::MonitorId>,
}

impl<B, C> GraphicsContextGeneric<B, C>
where
    B: BackendSpec<SurfaceType = C>,
    C: gfx::format::Formatted,
{
    pub(crate) fn get_format() -> gfx::format::Format {
        C::get_format()
    }
}

impl<B, C> fmt::Debug for GraphicsContextGeneric<B, C>
where
    B: BackendSpec<SurfaceType = C>,
    C: gfx::format::Formatted,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<GraphicsContext: {:p}>", self)
    }
}

/// A concrete graphics context for GL rendering.
pub(crate) type GraphicsContext =
    GraphicsContextGeneric<GlBackendSpec, <GlBackendSpec as BackendSpec>::SurfaceType>;

impl GraphicsContext {
    /// Create a new GraphicsContext
    pub(crate) fn new(
        events_loop: &glutin::EventsLoop,
        window_setup: &WindowSetup,
        window_mode: WindowMode,
        backend: GlBackendSpec,
        debug_id: DebugId,
    ) -> GameResult<GraphicsContext> {
        let color_format =
            <<GlBackendSpec as BackendSpec>::SurfaceType as gfx::format::Formatted>::get_format();
        let depth_format = gfx::format::Format(
            gfx::format::SurfaceType::D24_S8,
            gfx::format::ChannelType::Unorm,
        );

        // WINDOW SETUP
        let gl_builder = glutin::ContextBuilder::new()
            //GlRequest::Specific(Api::OpenGl, (backend.major, backend.minor))
            .with_gl(glutin::GlRequest::Latest)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_multisampling(window_setup.samples as u16)
            .with_pixel_format(5, 8)
            .with_vsync(window_setup.vsync);

        let mut window_builder = glutin::WindowBuilder::new()
            .with_title(window_setup.title.clone())
            .with_transparency(window_setup.transparent);
        window_builder = if !window_setup.icon.is_empty() {
            use winit::Icon;
            window_builder.with_window_icon(Some(Icon::from_path(&window_setup.icon)?))
        } else {
            window_builder
        };

        // TODO: see winit #540 about disabling resizing.
        /*if window_setup.resizable {
            window_builder.resizable();
        }*/

        let (window, device, mut factory, screen_render_target, depth_view) =
            gfx_window_glutin::init_raw(
                window_builder,
                gl_builder,
                events_loop,
                color_format,
                depth_format
            );

        let available_monitors = events_loop.get_available_monitors().collect();

        // TODO: see winit #548 about DPI.
        /*{
            // TODO: fix
            // Log a bunch of OpenGL state info pulled out of SDL and gfx
            let vsync = video.gl_get_swap_interval();
            let gl_attr = video.gl_attr();
            let (major, minor) = gl_attr.context_version();
            let profile = gl_attr.context_profile();
            let (w, h) = window.size();
            let (dw, dh) = window.drawable_size();
            let info = device.get_info();
            debug!("Window created.");
            debug!(
                "  Asked for     OpenGL {}.{} Core, vsync: {}",
                backend.major, backend.minor, window_mode.vsync
            );
            debug!(
                "  Actually got: OpenGL {}.{} {:?}, vsync: {:?}",
                major, minor, profile, vsync
            );
            debug!(
                "  Window size: {}x{}, drawable size: {}x{}, DPI: {:?}",
                w, h, dw, dh, dpi
            );
            debug!(
                "  Driver vendor: {}, renderer {}, version {:?}, shading language {:?}",
                info.platform_name.vendor,
                info.platform_name.renderer,
                info.version,
                info.shading_language
            );
        }*/

        // GFX SETUP
        let mut encoder: gfx::Encoder<
            gfx_device_gl::Resources,
            gfx_device_gl::CommandBuffer,
        > = factory.create_command_buffer().into();

        let blend_modes = [
            BlendMode::Alpha,
            BlendMode::Add,
            BlendMode::Subtract,
            BlendMode::Invert,
            BlendMode::Multiply,
            BlendMode::Replace,
            BlendMode::Lighten,
            BlendMode::Darken,
        ];
        let multisample_samples = window_setup.samples as u8;
        let (shader, draw) = create_shader(
            include_bytes!("shader/basic_150.glslv"),
            include_bytes!("shader/basic_150.glslf"),
            EmptyConst,
            "Empty",
            &mut encoder,
            &mut factory,
            multisample_samples,
            Some(&blend_modes[..]),
            debug_id,
        )?;

        let glyph_brush = GlyphBrushBuilder::using_font_bytes(Font::default_font_bytes().to_vec())
            .build(factory.clone());

        let rect_inst_props = factory.create_buffer(
            1,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::memory::Bind::SHADER_RESOURCE,
        )?;

        let (quad_vertex_buffer, mut quad_slice) =
            factory.create_vertex_buffer_with_slice(&QUAD_VERTS, &QUAD_INDICES[..]);

        quad_slice.instances = Some((1, 0));

        let globals_buffer = factory.create_constant_buffer(1);
        let mut samplers: SamplerCache<GlBackendSpec> = SamplerCache::new();
        let sampler_info =
            texture::SamplerInfo::new(texture::FilterMethod::Bilinear, texture::WrapMode::Clamp);
        let sampler = samplers.get_or_insert(sampler_info, &mut factory);
        let white_image = Image::make_raw(
            &mut factory,
            &sampler_info,
            1,
            1,
            &[255, 255, 255, 255],
            debug_id,
        )?;
        let texture = white_image.texture.clone();
        let typed_thingy = super::GlBackendSpec::raw_to_typed_shader_resource(texture);

        let data = pipe::Data {
            vbuf: quad_vertex_buffer.clone(),
            tex: (typed_thingy, sampler),
            rect_instance_properties: rect_inst_props,
            globals: globals_buffer,
            out: screen_render_target.clone(),
        };

        // Set initial uniform values
        let left = 0.0;
        let right = window_mode.width as f32;
        let top = 0.0;
        let bottom = window_mode.height as f32;
        let initial_projection = Matrix4::identity(); // not the actual initial projection matrix, just placeholder
        let initial_transform = Matrix4::identity();
        let globals = Globals {
            mvp_matrix: initial_projection.into(),
        };

        let mut gfx = GraphicsContext {
            shader_globals: globals,
            projection: initial_projection,
            modelview_stack: vec![initial_transform],
            white_image,
            screen_rect: Rect::new(left, top, right - left, bottom - top),
            color_format,
            depth_format,

            backend_spec: backend,
            window,
            multisample_samples,
            device: Box::new(device),
            factory: Box::new(factory),
            encoder,
            screen_render_target,
            depth_view,

            data,
            quad_slice,
            quad_vertex_buffer,

            default_sampler_info: sampler_info,
            samplers,

            default_shader: shader.shader_id(),
            current_shader: Rc::new(RefCell::new(None)),
            shaders: vec![draw],

            glyph_brush,

            available_monitors,
        };
        gfx.set_window_mode(window_mode)?;

        // Calculate and apply the actual initial projection matrix
        let w = window_mode.width as f32;
        let h = window_mode.height as f32;
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
        };
        gfx.set_projection_rect(rect);
        gfx.calculate_transform_matrix();
        gfx.update_globals()?;
        Ok(gfx)
    }

    /// Sends the current value of the graphics context's shader globals
    /// to the graphics card.
    pub(crate) fn update_globals(&mut self) -> GameResult {
        self.encoder
            .update_buffer(&self.data.globals, &[self.shader_globals], 0)?;
        Ok(())
    }

    /// Recalculates the context's Model-View-Projection matrix based on
    /// the matrices on the top of the respective stacks and the projection
    /// matrix.
    pub(crate) fn calculate_transform_matrix(&mut self) {
        let modelview = self.modelview_stack
            .last()
            .expect("Transform stack empty; should never happen");
        let mvp = self.projection * modelview;
        self.shader_globals.mvp_matrix = mvp.into();
    }

    /// Pushes a homogeneous transform matrix to the top of the transform
    /// (model) matrix stack.
    pub(crate) fn push_transform(&mut self, t: Matrix4) {
        self.modelview_stack.push(t);
    }

    /// Pops the current transform matrix off the top of the transform
    /// (model) matrix stack.
    pub(crate) fn pop_transform(&mut self) {
        if self.modelview_stack.len() > 1 {
            self.modelview_stack.pop();
        }
    }

    /// Sets the current model-view transform matrix.
    pub(crate) fn set_transform(&mut self, t: Matrix4) {
        assert!(
            !self.modelview_stack.is_empty(),
            "Tried to set a transform on an empty transform stack!"
        );
        let last = self.modelview_stack
            .last_mut()
            .expect("Transform stack empty; should never happen!");
        *last = t;
    }

    /// Gets a copy of the current transform matrix.
    pub(crate) fn get_transform(&self) -> Matrix4 {
        assert!(
            !self.modelview_stack.is_empty(),
            "Tried to get a transform on an empty transform stack!"
        );
        let last = self.modelview_stack
            .last()
            .expect("Transform stack empty; should never happen!");
        *last
    }

    /// Converts the given `DrawParam` into an `InstanceProperties` object and
    /// sends it to the graphics card at the front of the instance buffer.
    pub(crate) fn update_instance_properties(&mut self, draw_params: DrawParam) -> GameResult {
        // This clone is cheap since draw_params is Copy
        // TODO: Clean up
        let mut new_draw_params = draw_params;
        new_draw_params.color = draw_params.color;
        let properties = new_draw_params.into();
        self.encoder
            .update_buffer(&self.data.rect_instance_properties, &[properties], 0)?;
        Ok(())
    }

    /// Draws with the current encoder, slice, and pixel shader. Prefer calling
    /// this method from `Drawables` so that the pixel shader gets used
    pub(crate) fn draw(
        &mut self,
        slice: Option<&gfx::Slice<gfx_device_gl::Resources>>,
    ) -> GameResult {
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
    pub(crate) fn get_blend_mode(&self) -> BlendMode {
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &self.shaders[id];
        shader_handle.get_blend_mode()
    }

    /// Shortcut function to set the projection matrix to an
    /// orthographic projection based on the given `Rect`.
    ///
    /// Call `update_globals()` to apply it after calling this.
    pub(crate) fn set_projection_rect(&mut self, rect: Rect) {
        type Vec3 = na::Vector3<f32>;
        self.screen_rect = rect;
        self.projection =
            Matrix4::new_orthographic(rect.x, rect.x + rect.w, rect.y, rect.y + rect.h, -1.0, 1.0)
                .append_nonuniform_scaling(&Vec3::new(1.0, -1.0, 1.0));
    }

    /// Sets the raw projection matrix to the given Matrix.
    ///
    /// Call `update_globals()` to apply after calling this.
    pub(crate) fn set_projection(&mut self, mat: Matrix4) {
        self.projection = mat;
    }

    /// Gets a copy of the raw projection matrix.
    pub(crate) fn get_projection(&self) -> Matrix4 {
        self.projection
    }

    /// Sets window mode from a WindowMode object.
    pub(crate) fn set_window_mode(&mut self, mode: WindowMode) -> GameResult {
        let window = &self.window;

        window.set_maximized(mode.maximized);

        // TODO: find out if single-dimension constraints are possible.
        let mut min_dimensions = None;
        if mode.min_width > 0 && mode.min_height > 0 {
            min_dimensions = Some((mode.min_width, mode.min_height));
        }
        window.set_min_dimensions(min_dimensions);

        let mut max_dimensions = None;
        if mode.max_width > 0 && mode.max_height > 0 {
            max_dimensions = Some((mode.max_width, mode.max_height));
        }
        window.set_max_dimensions(max_dimensions);

        match mode.fullscreen_type {
            FullscreenType::Off => {
                window.set_fullscreen(None);
                window.set_decorations(!mode.borderless);
                window.set_inner_size(mode.width, mode.height);
            }
            FullscreenType::True(monitor) => {
                let monitor = match monitor {
                    MonitorId::Current => window.get_current_monitor(),
                    MonitorId::Index(i) => if i < self.available_monitors.len() {
                        self.available_monitors[i].clone()
                    } else {
                        return Err(GameError::VideoError(format!("No monitor #{} found!", i)));
                    }
                };
                window.set_fullscreen(Some(monitor));
                window.set_inner_size(mode.width, mode.height);
            }
            FullscreenType::Desktop(monitor) => {
                let monitor = match monitor {
                    MonitorId::Current => window.get_current_monitor(),
                    MonitorId::Index(i) => if i < self.available_monitors.len() {
                        self.available_monitors[i].clone()
                    } else {
                        return Err(GameError::VideoError(format!("No monitor #{} found!", i)));
                    }
                };
                let position = monitor.get_position();
                let dimensions = monitor.get_dimensions();
                window.set_fullscreen(None);
                window.set_decorations(false);
                window.set_inner_size(dimensions.0, dimensions.1);
                window.set_position(position.0, position.1);
            }
        }
        Ok(())
    }

    /// Communicates changes in the viewport size between SDL and gfx.
    ///
    /// Also replaces gfx.screen_render_target and gfx.depth_view,
    /// so it may cause squirrelliness to
    /// happen with canvases or other things that touch it.
    pub(crate) fn resize_viewport(&mut self) {
        // Basically taken from the definition of
        // gfx_window_sdl::update_views()
        let dim = self.screen_render_target.get_dimensions();
        assert_eq!(dim, self.depth_view.get_dimensions());
        if let Some((cv, dv)) = gfx_window_glutin::update_views_raw(
            &self.window,
            dim,
            self.color_format,
            self.depth_format) {
            self.screen_render_target = cv;
            self.depth_view = dv;
        }
    }
}
