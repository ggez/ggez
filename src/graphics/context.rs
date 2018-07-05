use std::cell::RefCell;
use std::rc::Rc;

use gfx::traits::FactoryExt;
use gfx::Factory;
use gfx_glyph::{GlyphBrush, GlyphBrushBuilder};
use glutin;
use winit::dpi;

use conf::{FullscreenType, WindowMode, WindowSetup};
use context::DebugId;
use graphics::*;

use GameResult;

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
    pub(crate) modelview_stack: Vec<Matrix4>,
    pub(crate) white_image: ImageGeneric<B>,
    pub(crate) screen_rect: Rect,
    color_format: gfx::format::Format,
    depth_format: gfx::format::Format,
    srgb: bool,

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
    pub(crate) shaders: Vec<Box<dyn ShaderHandle<B>>>,

    pub(crate) glyph_brush: GlyphBrush<'static, B::Resources, B::Factory>,
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
pub(crate) type GraphicsContext =
    GraphicsContextGeneric<GlBackendSpec>;



impl<B> GraphicsContextGeneric<B>
where
    B: BackendSpec + 'static {

    /// TODO: This is sorta redundant with BackendSpec too...?
    pub(crate) fn new_encoder(&mut self) -> gfx::Encoder<
            B::Resources,
            B::CommandBuffer,
        >  {
        let factory = &mut *self.factory;
        B::get_encoder(factory)
    }


    /// Create a new GraphicsContext
    pub(crate) fn new(
        events_loop: &glutin::EventsLoop,
        window_setup: &WindowSetup,
        window_mode: WindowMode,
        backend: B,
        debug_id: DebugId,
    ) -> GameResult<Self> {
        let srgb = window_setup.srgb;
        let color_format = if srgb {
            gfx::format::Format(
                gfx::format::SurfaceType::R8_G8_B8_A8,
                gfx::format::ChannelType::Srgb
            )
        } else {
            gfx::format::Format(
                gfx::format::SurfaceType::R8_G8_B8_A8,
                gfx::format::ChannelType::Unorm
            )
        };
        let depth_format = gfx::format::Format(
            gfx::format::SurfaceType::D24_S8,
            gfx::format::ChannelType::Unorm,
        );
        

        // WINDOW SETUP
        let gl_builder = glutin::ContextBuilder::new()
            //GlRequest::Specific(Api::OpenGl, (backend.major, backend.minor))
            // TODO: Fix the "Latest" here.
            .with_gl(glutin::GlRequest::Latest)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_multisampling(window_setup.samples as u16)
            // TODO: Better pixel format here?
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
            backend.init(
                window_builder,
                gl_builder,
                events_loop,
                color_format,
                depth_format,
            );

        // TODO: see winit #548 about DPI.
        {
            // TODO: improve.
            // Log a bunch of OpenGL state info pulled out of SDL and gfx
            let api = window.get_api();
            let dpi::LogicalSize{width: w, height: h} = window
                .get_outer_size()
                .ok_or_else(|| GameError::VideoError("Window doesn't exist!".to_owned()))?;
            let dpi::LogicalSize{width: dw, height: dh} = window
                .get_inner_size()
                .ok_or_else(|| GameError::VideoError("Window doesn't exist!".to_owned()))?;
            debug!("Window created.");
            let (major, minor) = backend.version_tuple();
            debug!(
                "  Asked for     OpenGL {}.{} Core, vsync: {}",
                major, minor, window_setup.vsync
            );
            debug!("  Actually got: OpenGL ?.? {:?}, vsync: ?", api);
            debug!("  Window size: {}x{}, drawable size: {}x{}", w, h, dw, dh);
            let device_info = backend.get_info(&device);
            debug!("  {}", device_info);
        }

        // GFX SETUP
        let mut encoder = B::get_encoder(&mut factory);

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
            color_format,
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
        let mut samplers: SamplerCache<B> = SamplerCache::new();
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

        let mut gfx = Self {
            shader_globals: globals,
            projection: initial_projection,
            modelview_stack: vec![initial_transform],
            white_image,
            screen_rect: Rect::new(left, top, right - left, bottom - top),
            color_format,
            depth_format,
            srgb,

            backend_spec: backend,
            window,
            multisample_samples,
            device: Box::new(device as B::Device),
            factory: Box::new(factory as B::Factory),
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
            let _ = self.modelview_stack.pop();
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
    pub(crate) fn update_instance_properties(
        &mut self,
        draw_params: DrawTransform,
    ) -> GameResult {
        // This clone is cheap since draw_params is Copy
        // TODO: Clean up
        let mut new_draw_params = draw_params;
        new_draw_params.color = draw_params.color;
        let properties = new_draw_params.to_instance_properties(self.srgb);
        self.encoder
            .update_buffer(&self.data.rect_instance_properties, &[properties], 0)?;
        Ok(())
    }

    /// Draws with the current encoder, slice, and pixel shader. Prefer calling
    /// this method from `Drawables` so that the pixel shader gets used
    pub(crate) fn draw(
        &mut self,
        slice: Option<&gfx::Slice<B::Resources>>,
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
        if mode.min_width > 0.0 && mode.min_height > 0.0 {
            min_dimensions = Some(dpi::LogicalSize {
                width: mode.min_width, 
                height: mode.min_height,
            });
        }
        window.set_min_dimensions(min_dimensions);

        let mut max_dimensions = None;
        if mode.max_width > 0.0 && mode.max_height > 0.0 {
            max_dimensions = Some(dpi::LogicalSize {
                width: mode.max_width, 
                height: mode.max_height
            });
        }
        window.set_max_dimensions(max_dimensions);

        let monitor = window.get_current_monitor();
        match mode.fullscreen_type {
            FullscreenType::Off => {
                window.set_fullscreen(None);
                window.set_decorations(!mode.borderless);
                window.set_inner_size(dpi::LogicalSize {
                    width: mode.width, 
                    height: mode.height,
                });
            }
            FullscreenType::True => {
                window.set_fullscreen(Some(monitor));
                window.set_inner_size(dpi::LogicalSize {
                    width: mode.width, 
                    height: mode.height,
                });
            }
            FullscreenType::Desktop => {
                let position = monitor.get_position();
                let dimensions = monitor.get_dimensions();
                window.set_fullscreen(None);
                window.set_decorations(false);
                // BUGGO: Need to find and store dpi_size
                window.set_inner_size(dimensions.to_logical(1.0));
                window.set_position(position.to_logical(1.0));
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
        // Basically taken from the definition of
        // gfx_window_glutin::update_views()
        if let Some((cv, dv)) = self.backend_spec.resize_viewport(&self.screen_render_target, &self.depth_view,
        self.color_format(), self.depth_format(), &self.window) {
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
        if let gfx::format::Format(_, gfx::format::ChannelType::Srgb) = self.color_format() {
            true
        } else {
            false
        }
    }
}
