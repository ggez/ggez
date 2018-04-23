use std::cell::RefCell;
use std::rc::Rc;

use gfx_device_gl;
use gfx_window_sdl;
use gfx::traits::FactoryExt;
use gfx::Factory;

use context::DebugId;
use graphics::*;
use conf::WindowSetup;

/// A structure that contains graphics state.
/// For instance, background and foreground colors,
/// window info, DPI, rendering pipeline state, etc.
///
/// As an end-user you shouldn't ever have to touch this.
pub(crate) struct GraphicsContextGeneric<B>
where
    B: BackendSpec,
{
    pub(crate) foreground_color: Color,
    pub(crate) background_color: Color,
    shader_globals: Globals,
    projection: Matrix4,
    pub(crate) modelview_stack: Vec<Matrix4>,
    pub(crate) white_image: Image,
    pub(crate) screen_rect: Rect,
    pub(crate) dpi: (f32, f32, f32),

    pub(crate) backend_spec: B,
    pub(crate) window: sdl2::video::Window,
    pub(crate) multisample_samples: u8,
    #[allow(dead_code)] gl_context: sdl2::video::GLContext,
    pub(crate) device: Box<B::Device>,
    pub(crate) factory: Box<B::Factory>,
    pub(crate) encoder: gfx::Encoder<B::Resources, B::CommandBuffer>,
    pub(crate) screen_render_target:
        gfx::handle::RenderTargetView<B::Resources, gfx::format::Srgba8>,
    #[allow(dead_code)]
    pub(crate) depth_view: gfx::handle::DepthStencilView<B::Resources, gfx::format::DepthStencil>,

    pub(crate) data: pipe::Data<B::Resources>,
    pub(crate) quad_slice: gfx::Slice<B::Resources>,
    pub(crate) quad_vertex_buffer: gfx::handle::Buffer<B::Resources, Vertex>,

    pub(crate) default_sampler_info: texture::SamplerInfo,
    pub(crate) samplers: SamplerCache<B>,

    default_shader: ShaderId,
    pub(crate) current_shader: Rc<RefCell<Option<ShaderId>>>,
    pub(crate) shaders: Vec<Box<ShaderHandle<B>>>,

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

impl GraphicsContext {
    /// Create a new GraphicsContext
    pub(crate) fn new(
        video: &sdl2::VideoSubsystem,
        window_setup: &WindowSetup,
        window_mode: WindowMode,
        backend: GlBackendSpec,
        debug_id: DebugId,
    ) -> GameResult<GraphicsContext> {
        // WINDOW SETUP
        let gl = video.gl_attr();
        gl.set_context_version(backend.major, backend.minor);
        gl.set_context_profile(sdl2::video::GLProfile::Core);
        gl.set_red_size(5);
        gl.set_green_size(5);
        gl.set_blue_size(5);
        gl.set_alpha_size(8);
        let samples = window_setup.samples as u8;
        if samples > 1 {
            gl.set_multisample_buffers(1);
            gl.set_multisample_samples(samples);
        }
        let mut window_builder =
            video.window(&window_setup.title, window_mode.width, window_mode.height);
        if window_setup.resizable {
            window_builder.resizable();
        }
        if window_setup.allow_highdpi {
            window_builder.allow_highdpi();
        }
        let (window, gl_context, device, mut factory, screen_render_target, depth_view) =
            gfx_window_sdl::init(video, window_builder)?;

        GraphicsContext::set_vsync(video, window_mode.vsync)?;

        let display_index = window.display_index()?;
        let dpi = window.subsystem().display_dpi(display_index)?;

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
        let (shader, draw) = create_shader(
            include_bytes!("shader/basic_150.glslv"),
            include_bytes!("shader/basic_150.glslf"),
            EmptyConst,
            "Empty",
            &mut encoder,
            &mut factory,
            samples,
            Some(&blend_modes[..]),
            debug_id,
        )?;

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
        let white_image =
            Image::make_raw(&mut factory, &sampler_info, 1, 1, &[255, 255, 255, 255], debug_id)?;
        let texture = white_image.texture.clone();

        let data = pipe::Data {
            vbuf: quad_vertex_buffer.clone(),
            tex: (texture, sampler),
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
            foreground_color: types::WHITE,
            background_color: Color::new(0.1, 0.2, 0.3, 1.0),
            shader_globals: globals,
            projection: initial_projection,
            modelview_stack: vec![initial_transform],
            white_image,
            screen_rect: Rect::new(left, top, right - left, bottom - top),
            dpi,

            backend_spec: backend,
            window,
            multisample_samples: samples,
            gl_context,
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
    pub(crate) fn update_globals(&mut self) -> GameResult<()> {
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
    pub(crate) fn update_instance_properties(&mut self, draw_params: DrawParam) -> GameResult<()> {
        // This clone is cheap since draw_params is Copy
        let mut new_draw_params = draw_params;
        let fg = Some(self.foreground_color);
        new_draw_params.color = draw_params.color.or(fg);
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
    ) -> GameResult<()> {
        let slice = slice.unwrap_or(&self.quad_slice);
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &self.shaders[id];

        shader_handle.draw(&mut self.encoder, slice, &self.data)?;
        Ok(())
    }

    /// Sets the blend mode of the active shader
    pub(crate) fn set_blend_mode(&mut self, mode: BlendMode) -> GameResult<()> {
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

    /// Just a helper method to set window mode from a WindowMode object.
    pub(crate) fn set_window_mode(&mut self, mode: WindowMode) -> GameResult<()> {
        let window = &mut self.window;
        window.set_size(mode.width, mode.height)?;
        // SDL sets "bordered" but Love2D does "not bordered";
        // we use the Love2D convention.
        window.set_bordered(!mode.borderless);
        window.set_fullscreen(mode.fullscreen_type.into())?;
        window.set_minimum_size(mode.min_width, mode.min_height)?;
        window.set_maximum_size(mode.max_width, mode.max_height)?;
        Ok(())
    }

    /// Another helper method to set vsync.
    pub(crate) fn set_vsync(video: &sdl2::VideoSubsystem, vsync: bool) -> GameResult<()> {
        let vsync_int = if vsync { 1 } else { 0 };
        if video.gl_set_swap_interval(vsync_int) {
            Ok(())
        } else {
            let err = sdl2::get_error();
            Err(GameError::VideoError(err))
        }
    }

    /// Communicates changes in the viewport size between SDL and gfx.
    ///
    /// Also replaces gfx.data.out so it may cause squirrelliness to
    /// happen with canvases or other things that touch it.
    pub(crate) fn resize_viewport(&mut self) {
        gfx_window_sdl::update_views(
            &self.window,
            &mut self.screen_render_target,
            &mut self.depth_view,
        );
    }
}
