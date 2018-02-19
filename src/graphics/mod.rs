//! The `graphics` module performs the actual drawing of images, text, and other
//! objects with the `Drawable` trait.  It also handles basic loading of images
//! and text.
//!
//! This module also manages graphics state, coordinate systems, etc.
//! The default coordinate system has the origin in the upper-left
//! corner of the screen, with Y increasing downwards.

use std::fmt;
use std::path;
use std::convert::From;
use std::collections::HashMap;
use std::io::Read;
use std::u16;
use std::cell::RefCell;
use std::rc::Rc;

use sdl2;
use image;
use gfx;
use gfx::texture;
use gfx::traits::Device;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_sdl;
use gfx::Factory;

use conf;
use conf::{WindowMode, WindowSetup};
use context::Context;
use GameError;
use GameResult;

mod canvas;
mod mesh;
mod shader;
mod text;
mod types;
use nalgebra as na;

pub mod spritebatch;

pub use self::canvas::*;
pub use self::mesh::*;
pub use self::shader::*;
pub use self::text::*;
pub use self::types::*;

/// A marker trait saying that something is a label for a particular backend,
/// with associated gfx-rs types for that backend.
pub trait BackendSpec: fmt::Debug {
    /// gfx resource type
    type Resources: gfx::Resources;
    /// gfx factory type
    type Factory: gfx::Factory<Self::Resources>;
    /// gfx command buffer type
    type CommandBuffer: gfx::CommandBuffer<Self::Resources>;
    /// gfx device type
    type Device: gfx::Device<Resources = Self::Resources, CommandBuffer = Self::CommandBuffer>;
}

/// A backend specification for OpenGL.
/// This is different from `conf::Backend` because
/// this needs to be its own struct to implement traits upon,
/// and because there may need to be a layer of translation
/// between what the user specifies in the config, and what the
/// graphics backend init code actually gets.
///
/// You shouldn't normally need to fiddle with this directly
/// but it has to be exported cause generic types like
/// `Shader` depend on it.
#[derive(Debug, Copy, Clone, PartialEq, Eq, SmartDefault, Hash)]
pub struct GlBackendSpec {
    #[default = r#"3"#] major: u8,
    #[default = r#"2"#] minor: u8,
}

impl From<conf::Backend> for GlBackendSpec {
    fn from(c: conf::Backend) -> Self {
        match c {
            conf::Backend::OpenGL { major, minor } => Self {
                major: major,
                minor: minor,
            },
        }
    }
}

impl BackendSpec for GlBackendSpec {
    type Resources = gfx_device_gl::Resources;
    type Factory = gfx_device_gl::Factory;
    type CommandBuffer = gfx_device_gl::CommandBuffer;
    type Device = gfx_device_gl::Device;
}

const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        pos: [0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        pos: [1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0],
        uv: [0.0, 1.0],
    },
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

type ColorFormat = gfx::format::Srgba8;
// I don't know why this gives a dead code warning
// since this type is definitely used... oh well.
#[allow(dead_code)]
type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    /// Internal structure containing vertex data.
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    /// Internal structure containing values that are different for each
    /// drawable object.
    vertex InstanceProperties {
        // the columns here are for the transform matrix;
        // you can't shove a full matrix into an instance
        // buffer, annoyingly.
        src: [f32; 4] = "a_Src",
        col1: [f32; 4] = "a_TCol1",
        col2: [f32; 4] = "a_TCol2",
        col3: [f32; 4] = "a_TCol3",
        col4: [f32; 4] = "a_TCol4",
        color: [f32; 4] = "a_Color",
    }

    /// Internal structure containing global shader state.
    constant Globals {
        mvp_matrix: [[f32; 4]; 4] = "u_MVP",
    }

    // Internal structure containing graphics pipeline state.
    // This can't be a doc comment though because it somehow
    // breaks the gfx_defines! macro though.  :-(
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        globals: gfx::ConstantBuffer<Globals> = "Globals",
        rect_instance_properties: gfx::InstanceBuffer<InstanceProperties> = (),
        out: gfx::BlendTarget<ColorFormat> =
          ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

impl Default for InstanceProperties {
    fn default() -> Self {
        InstanceProperties {
            src: [0.0, 0.0, 1.0, 1.0],
            col1: [1.0, 0.0, 0.0, 0.0],
            col2: [0.0, 1.0, 0.0, 0.0],
            col3: [1.0, 0.0, 1.0, 0.0],
            col4: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl From<DrawParam> for InstanceProperties {
    fn from(p: DrawParam) -> Self {
        let mat: [[f32; 4]; 4] = p.into_matrix().into();
        let linear_color: types::LinearColor = p.color
            .expect("Converting DrawParam to InstanceProperties had None for a color; this should never happen!")
            .into();
        Self {
            src: p.src.into(),
            col1: mat[0],
            col2: mat[1],
            col3: mat[2],
            col4: mat[3],
            color: linear_color.into(),
        }
    }
}

/// A structure for conveniently storing Sampler's, based off
/// their `SamplerInfo`.
struct SamplerCache<B>
where
    B: BackendSpec,
{
    samplers: HashMap<texture::SamplerInfo, gfx::handle::Sampler<B::Resources>>,
}

impl<B> SamplerCache<B>
where
    B: BackendSpec,
{
    fn new() -> Self {
        SamplerCache {
            samplers: HashMap::new(),
        }
    }

    fn get_or_insert(
        &mut self,
        info: texture::SamplerInfo,
        factory: &mut B::Factory,
    ) -> gfx::handle::Sampler<B::Resources> {
        let sampler = self.samplers
            .entry(info)
            .or_insert_with(|| factory.create_sampler(info));
        sampler.clone()
    }
}

/// A structure that contains graphics state.
/// For instance, background and foreground colors,
/// window info, DPI, rendering pipeline state, etc.
///
/// As an end-user you shouldn't ever have to touch this.
pub(crate) struct GraphicsContextGeneric<B>
where
    B: BackendSpec,
{
    foreground_color: Color,
    background_color: Color,
    shader_globals: Globals,
    projection: Matrix4,
    modelview_stack: Vec<Matrix4>,
    white_image: Image,
    screen_rect: Rect,
    dpi: (f32, f32, f32),

    backend_spec: B,
    window: sdl2::video::Window,
    multisample_samples: u8,
    #[allow(dead_code)] gl_context: sdl2::video::GLContext,
    device: Box<B::Device>,
    factory: Box<B::Factory>,
    encoder: gfx::Encoder<B::Resources, B::CommandBuffer>,
    screen_render_target: gfx::handle::RenderTargetView<B::Resources, gfx::format::Srgba8>,
    #[allow(dead_code)]
    depth_view: gfx::handle::DepthStencilView<B::Resources, gfx::format::DepthStencil>,

    data: pipe::Data<B::Resources>,
    quad_slice: gfx::Slice<B::Resources>,
    quad_vertex_buffer: gfx::handle::Buffer<B::Resources, Vertex>,

    default_sampler_info: texture::SamplerInfo,
    samplers: SamplerCache<B>,

    default_shader: ShaderId,
    current_shader: Rc<RefCell<Option<ShaderId>>>,
    shaders: Vec<Box<ShaderHandle<B>>>,
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

/// This can probably be removed but might be
/// handy to keep around a bit longer.  Just in case something else
/// crazy happens.
#[allow(unused)]
fn test_opengl_versions(video: &sdl2::VideoSubsystem) {
    let mut major_versions = [4u8, 3u8, 2u8, 1u8];
    let minor_versions = [5u8, 4u8, 3u8, 2u8, 1u8, 0u8];
    major_versions.reverse();
    for major in &major_versions {
        for minor in &minor_versions {
            let gl = video.gl_attr();
            gl.set_context_version(*major, *minor);
            gl.set_context_profile(sdl2::video::GLProfile::Core);
            gl.set_red_size(5);
            gl.set_green_size(5);
            gl.set_blue_size(5);
            gl.set_alpha_size(8);

            print!("Requesting GL {}.{}... ", major, minor);
            let window_builder = video.window("so full of hate", 640, 480);
            let result = gfx_window_sdl::init::<ColorFormat, DepthFormat>(video, window_builder);
            match result {
                Ok(_) => println!(
                    "Ok, got GL {}.{}.",
                    gl.context_major_version(),
                    gl.context_minor_version()
                ),
                Err(res) => println!("Request failed: {:?}", res),
            }
        }
    }
}

impl From<gfx::buffer::CreationError> for GameError {
    fn from(e: gfx::buffer::CreationError) -> Self {
        use gfx::buffer::CreationError;
        match e {
            CreationError::UnsupportedBind(b) => GameError::RenderError(format!(
                "Could not create buffer: Unsupported Bind ({:?})",
                b
            )),
            CreationError::UnsupportedUsage(u) => GameError::RenderError(format!(
                "Could not create buffer: Unsupported Usage ({:?})",
                u
            )),
            CreationError::Other => {
                GameError::RenderError("Could not create buffer: Unknown error".to_owned())
            }
        }
    }
}

impl GraphicsContext {
    /// Create a new GraphicsContext
    pub(crate) fn new(
        video: &sdl2::VideoSubsystem,
        window_setup: &WindowSetup,
        window_mode: WindowMode,
        backend: GlBackendSpec,
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

        GraphicsContext::set_vsync(video, window_mode.vsync);

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
            Image::make_raw(&mut factory, &sampler_info, 1, 1, &[255, 255, 255, 255])?;
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
            white_image: white_image,
            screen_rect: Rect::new(left, top, right - left, bottom - top),
            dpi: dpi,

            backend_spec: backend,
            window: window,
            multisample_samples: samples,
            gl_context: gl_context,
            device: Box::new(device),
            factory: Box::new(factory),
            encoder: encoder,
            screen_render_target: screen_render_target,
            depth_view: depth_view,

            data: data,
            quad_slice: quad_slice,
            quad_vertex_buffer: quad_vertex_buffer,

            default_sampler_info: sampler_info,
            samplers: samplers,

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
    fn update_globals(&mut self) -> GameResult<()> {
        self.encoder
            .update_buffer(&self.data.globals, &[self.shader_globals], 0)?;
        Ok(())
    }

    /// Recalculates the context's Model-View-Projection matrix based on
    /// the matrices on the top of the respective stacks and the projection
    /// matrix.
    fn calculate_transform_matrix(&mut self) {
        let modelview = self.modelview_stack
            .last()
            .expect("Transform stack empty; should never happen");
        let mvp = self.projection * modelview;
        self.shader_globals.mvp_matrix = mvp.into();
    }

    /// Pushes a homogeneous transform matrix to the top of the transform
    /// (model) matrix stack.
    fn push_transform(&mut self, t: Matrix4) {
        self.modelview_stack.push(t);
    }

    /// Pops the current transform matrix off the top of the transform
    /// (model) matrix stack.
    fn pop_transform(&mut self) {
        if self.modelview_stack.len() > 1 {
            self.modelview_stack.pop();
        }
    }

    /// Sets the current model-view transform matrix.
    fn set_transform(&mut self, t: Matrix4) {
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
    fn get_transform(&self) -> Matrix4 {
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
    fn update_instance_properties(&mut self, draw_params: DrawParam) -> GameResult<()> {
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
    fn draw(&mut self, slice: Option<&gfx::Slice<gfx_device_gl::Resources>>) -> GameResult<()> {
        let slice = slice.unwrap_or(&self.quad_slice);
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &self.shaders[id];

        shader_handle.draw(&mut self.encoder, slice, &self.data)?;
        Ok(())
    }

    /// Sets the blend mode of the active shader
    fn set_blend_mode(&mut self, mode: BlendMode) -> GameResult<()> {
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &mut self.shaders[id];
        shader_handle.set_blend_mode(mode)
    }

    /// Gets the current blend mode of the active shader
    fn get_blend_mode(&self) -> BlendMode {
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &self.shaders[id];
        shader_handle.get_blend_mode()
    }

    /// Shortcut function to set the projection matrix to an
    /// orthographic projection based on the given `Rect`.
    ///
    /// Call `update_globals()` to apply it after calling this.
    fn set_projection_rect(&mut self, rect: Rect) {
        type Vec3 = na::Vector3<f32>;
        self.screen_rect = rect;
        self.projection =
            Matrix4::new_orthographic(rect.x, rect.x + rect.w, rect.y, rect.y + rect.h, -1.0, 1.0)
                .append_nonuniform_scaling(&Vec3::new(1.0, -1.0, 1.0));
    }

    /// Sets the raw projection matrix to the given Matrix.
    ///
    /// Call `update_globals()` to apply after calling this.
    fn set_projection(&mut self, mat: Matrix4) {
        self.projection = mat;
    }

    /// Gets a copy of the raw projection matrix.
    fn get_projection(&self) -> Matrix4 {
        self.projection
    }

    /// Just a helper method to set window mode from a WindowMode object.
    fn set_window_mode(&mut self, mode: WindowMode) -> GameResult<()> {
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
    fn set_vsync(video: &sdl2::VideoSubsystem, vsync: bool) {
        let vsync_int = if vsync { 1 } else { 0 };
        video.gl_set_swap_interval(vsync_int);
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

// **********************************************************************
// DRAWING
// **********************************************************************

/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    let linear_color: types::LinearColor = gfx.background_color.into();
    gfx.encoder.clear(&gfx.data.out, linear_color.into());
}

/// Draws the given `Drawable` object to the screen by calling its
/// `draw()` method.
pub fn draw(ctx: &mut Context, drawable: &Drawable, dest: Point2, rotation: f32) -> GameResult<()> {
    drawable.draw(ctx, dest, rotation)
}

/// Draws the given `Drawable` object to the screen by calling its `draw_ex()` method.
pub fn draw_ex(ctx: &mut Context, drawable: &Drawable, params: DrawParam) -> GameResult<()> {
    drawable.draw_ex(ctx, params)
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your `EventHandler`'s `draw()` method.
///
/// Unsets any active canvas.
pub fn present(ctx: &mut Context) {
    let gfx = &mut ctx.gfx_context;
    gfx.data.out = gfx.screen_render_target.clone();
    // We might want to give the user more control over when the
    // encoder gets flushed eventually, if we want them to be able
    // to do their own gfx drawing.  HOWEVER, the whole pipeline type
    // thing is a bigger hurdle, so this is fine for now.
    gfx.encoder.flush(&mut *gfx.device);
    gfx.window.gl_swap_window();
    gfx.device.cleanup();
}

/// Take a screenshot by outputting the current render surface
/// (screen or selected canvas) to a PNG file with the given name,
/// in the game's config directory.
pub fn screenshot(ctx: &mut Context, name: &str) -> GameResult<()> {
    use std::path::PathBuf;

    let mut full_path = PathBuf::from(ctx.filesystem.get_user_config_dir());
    full_path.push("screenshots");
    full_path.push(name);
    screenshot_to_path(ctx, &*full_path.to_string_lossy()).unwrap();
    Ok(())
}

// TODO link screenshot fn in below doc

/// Take a screenshot by outputting the current render surface
/// (screen or selected canvas) to a PNG file.
/// 
/// `screenshot` should be preferred if you do not need absolute control
/// over where the screenshot will be placed.
pub fn screenshot_to_path(ctx: &mut Context, path: &str) -> GameResult<()> {
    use gfx::memory::Typed;
    use gfx::format::Formatted;
    use gfx::format::SurfaceTyped;
    use gfx::traits::FactoryExt;
    use ::image as image_crate;

    let gfx = &mut ctx.gfx_context;
    let (w, h, _, _) = gfx.data.out.get_dimensions();
    type SurfaceData = <<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType;

    // Note: In the GFX example, the download buffer is created ahead of time
    // and updated on screen resize events. This may be preferable, but then
    // the buffer also needs to be updated when we switch to/from a canvas.
    // Unsure of the performance impact of creating this as it is needed.
    let dl_buffer = gfx.factory.create_download_buffer::<SurfaceData>(w as usize * h as usize)?;

    let mut local_encoder: gfx::Encoder<
        gfx_device_gl::Resources,
        gfx_device_gl::CommandBuffer> = gfx.factory.create_command_buffer().into();
    
    local_encoder.copy_texture_to_buffer_raw(
        gfx.data.out.raw().get_texture(),
        None,
        gfx::texture::RawImageInfo {
                    xoffset: 0,
                    yoffset: 0,
                    zoffset: 0,
                    width: w,
                    height: h,
                    depth: 0,
                    format: ColorFormat::get_format(),
                    mipmap: 0,
        },
        dl_buffer.raw(),
        0
    )?;
    local_encoder.flush(&mut *gfx.device);

    let reader = gfx.factory.read_mapping(&dl_buffer)?;

    // intermediary buffer to avoid casting
    // and also to reverse the order in which we pass the rows
    // so the screenshot isn't upside-down
    let mut data = Vec::with_capacity(w as usize * h as usize * 4);
    for row in reader.chunks(w as usize).rev() {
        for pixel in row.iter() {
            data.extend(pixel);
        }
    }
    
    image_crate::save_buffer(path, &data, u32::from(w), u32::from(h), image_crate::ColorType::RGBA(8))?;

    Ok(())
}

/*
// Draw an arc.
// Punting on this until later.
pub fn arc(_ctx: &mut Context,
           _mode: DrawMode,
           _point: Point,
           _radius: f32,
           _angle1: f32,
           _angle2: f32,
           _segments: u32)
           -> GameResult<()> {
    unimplemented!();
}
*/

/// Draw a circle.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
///
/// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
pub fn circle(
    ctx: &mut Context,
    mode: DrawMode,
    point: Point2,
    radius: f32,
    tolerance: f32,
) -> GameResult<()> {
    let m = Mesh::new_circle(ctx, mode, point, radius, tolerance)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

/// Draw an ellipse.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
///
/// For the meaning of the `tolerance` parameter, [see here](https://docs.rs/lyon_geom/0.9.0/lyon_geom/#flattening).
pub fn ellipse(
    ctx: &mut Context,
    mode: DrawMode,
    point: Point2,
    radius1: f32,
    radius2: f32,
    tolerance: f32,
) -> GameResult<()> {
    let m = Mesh::new_ellipse(ctx, mode, point, radius1, radius2, tolerance)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

/// Draws a line of one or more connected segments.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn line(ctx: &mut Context, points: &[Point2], width: f32) -> GameResult<()> {
    let m = Mesh::new_line(ctx, points, width)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

/// Draws points (as rectangles)
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn points(ctx: &mut Context, points: &[Point2], point_size: f32) -> GameResult<()> {
    for p in points {
        let r = Rect::new(p.x, p.y, point_size, point_size);
        rectangle(ctx, DrawMode::Fill, r)?;
    }
    Ok(())
}

/// Draws a closed polygon
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn polygon(ctx: &mut Context, mode: DrawMode, vertices: &[Point2]) -> GameResult<()> {
    let m = Mesh::new_polygon(ctx, mode, vertices)?;
    m.draw(ctx, Point2::origin(), 0.0)
}

// Renders text with the default font.
// Not terribly efficient as it re-renders the text with each call,
// but good enough for debugging.
// Doesn't actually work, double-borrow on ctx.  Bah.
// pub fn print(ctx: &mut Context, dest: Point, text: &str) -> GameResult<()> {
//     let rendered_text = {
//         let font = &ctx.default_font;
//         text::Text::new(ctx, text, font)?
//     };
//     draw(ctx, &rendered_text, dest, 0.0)
// }

/// Draws a rectangle.
///
/// Allocates a new `Mesh`, draws it, and throws it away, so if you are drawing many of them
/// you should create the `Mesh` yourself.
pub fn rectangle(ctx: &mut Context, mode: DrawMode, rect: Rect) -> GameResult<()> {
    let x1 = rect.x;
    let x2 = rect.x + rect.w;
    let y1 = rect.y;
    let y2 = rect.y + rect.h;
    let pts = [
        Point2::new(x1, y1),
        Point2::new(x2, y1),
        Point2::new(x2, y2),
        Point2::new(x1, y2),
    ];
    polygon(ctx, mode, &pts)
}

// **********************************************************************
// GRAPHICS STATE
// **********************************************************************

/// Returns the current background color.
pub fn get_background_color(ctx: &Context) -> Color {
    ctx.gfx_context.background_color
}

/// Returns the current foreground color.
pub fn get_color(ctx: &Context) -> Color {
    ctx.gfx_context.foreground_color
}

/// Get the default filter mode for new images.
pub fn get_default_filter(ctx: &Context) -> FilterMode {
    let gfx = &ctx.gfx_context;
    gfx.default_sampler_info.filter.into()
}

/// Returns a string that tells a little about the obtained rendering mode.
/// It is supposed to be human-readable and will change; do not try to parse
/// information out of it!
pub fn get_renderer_info(ctx: &Context) -> GameResult<String> {
    let video = ctx.sdl_context.video()?;

    let gl = video.gl_attr();

    Ok(format!(
        "Requested GL {}.{} Core profile, actually got GL {}.{} {:?} profile.",
        ctx.gfx_context.backend_spec.major,
        ctx.gfx_context.backend_spec.minor,
        gl.context_major_version(),
        gl.context_minor_version(),
        gl.context_profile()
    ))
}

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: left, y: top, w: width, h: height }`
///
/// If the Y axis increases downwards, the `height` of the Rect
/// will be negative.
pub fn get_screen_coordinates(ctx: &Context) -> Rect {
    ctx.gfx_context.screen_rect
}

/// Sets the background color.  Default: blue.
pub fn set_background_color(ctx: &mut Context, color: Color) {
    ctx.gfx_context.background_color = color;
}

/// Sets the foreground color, which will be used for drawing
/// rectangles, lines, etc.  Default: white.
pub fn set_color(ctx: &mut Context, color: Color) -> GameResult<()> {
    let gfx = &mut ctx.gfx_context;
    gfx.foreground_color = color;
    Ok(())
}

/// Sets the default filter mode used to scale images.
///
/// This does not apply retroactively to already created images.
pub fn set_default_filter(ctx: &mut Context, mode: FilterMode) {
    let gfx = &mut ctx.gfx_context;
    let new_mode = mode.into();
    let sampler_info = texture::SamplerInfo::new(new_mode, texture::WrapMode::Clamp);
    // We create the sampler now so we don't end up creating it at some
    // random-ass time while we're trying to draw stuff.
    let _sampler = gfx.samplers.get_or_insert(sampler_info, &mut *gfx.factory);
    gfx.default_sampler_info = sampler_info;
}

/// Sets the bounds of the screen viewport.
///
/// The default coordinate system has (0,0) at the top-left corner
/// with X increasing to the right and Y increasing down, with the
/// viewport scaled such that one coordinate unit is one pixel on the
/// screen.  This function lets you change this coordinate system to
/// be whatever you prefer.
///
/// The `Rect`'s x and y will define the top-left corner of the screen,
/// and that plus its w and h will define the bottom-right corner.
pub fn set_screen_coordinates(context: &mut Context, rect: Rect) -> GameResult<()> {
    let gfx = &mut context.gfx_context;
    gfx.set_projection_rect(rect);
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}

/// Sets the raw projection matrix to the given homogeneous
/// transformation matrix.
///
/// You must call `apply_transformations(ctx)` after calling this to apply 
/// these changes and recalculate the underlying MVP matrix.
pub fn set_projection(context: &mut Context, proj: Matrix4) {
    let gfx = &mut context.gfx_context;
    gfx.set_projection(proj);
}

/// Premultiplies the given transformation matrix with the current projection matrix
///
/// You must call `apply_transformations(ctx)` after calling this to apply 
/// these changes and recalculate the underlying MVP matrix.
pub fn transform_projection(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    let curr = gfx.get_projection();
    gfx.set_projection(transform * curr);
}

/// Gets a copy of the context's raw projection matrix
pub fn get_projection(context: &Context) -> Matrix4 {
    let gfx = &context.gfx_context;
    gfx.get_projection()
}

/// Pushes a homogeneous transform matrix to the top of the transform
/// (model) matrix stack of the `Context`. If no matrix is given, then
/// pushes a copy of the current transform matrix to the top of the stack.
///
/// You must call `apply_transformations(ctx)` after calling this to apply 
/// these changes and recalculate the underlying MVP matrix.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn push_transform(context: &mut Context, transform: Option<Matrix4>) {
    let gfx = &mut context.gfx_context;
    if let Some(t) = transform {
        gfx.push_transform(t);
    } else {
        let copy = *gfx.modelview_stack
            .last()
            .expect("Matrix stack empty, should never happen");
        gfx.push_transform(copy);
    }
}

/// Pops the transform matrix off the top of the transform
/// (model) matrix stack of the `Context`.
///
/// You must call `apply_transformations(ctx)` after calling this to apply 
/// these changes and recalculate the underlying MVP matrix.
pub fn pop_transform(context: &mut Context) {
    let gfx = &mut context.gfx_context;
    gfx.pop_transform();
}

/// Sets the current model transformation to the given homogeneous
/// transformation matrix.
///
/// You must call `apply_transformations(ctx)` after calling this to apply 
/// these changes and recalculate the underlying MVP matrix.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn set_transform(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    gfx.set_transform(transform);
}

/// Gets a copy of the context's current transform matrix
pub fn get_transform(context: &Context) -> Matrix4 {
    let gfx = &context.gfx_context;
    gfx.get_transform()
}

/// Premultiplies the given transform with the current model transform.
///
/// You must call `apply_transformations(ctx)` after calling this to apply 
/// these changes and recalculate the underlying MVP matrix.
///
/// A `DrawParam` can be converted into an appropriate transform
/// matrix by calling `param.into_matrix()`.
pub fn transform(context: &mut Context, transform: Matrix4) {
    let gfx = &mut context.gfx_context;
    let curr = gfx.get_transform();
    gfx.set_transform(transform * curr);
}

/// Sets the current model transform to the origin transform (no transformation)
///
/// You must call `apply_transformations(ctx)` after calling this to apply 
/// these changes and recalculate the underlying MVP matrix.
pub fn origin(context: &mut Context) {
    let gfx = &mut context.gfx_context;
    gfx.set_transform(Matrix4::identity());
}

/// Calculates the new total transformation (Model-View-Projection) matrix
/// based on the matrices at the top of the transform and view matrix stacks
/// and sends it to the graphics card.
pub fn apply_transformations(context: &mut Context) -> GameResult<()> {
    let gfx = &mut context.gfx_context;
    gfx.calculate_transform_matrix();
    gfx.update_globals()
}

/// Sets the blend mode of the currently active shader program
pub fn set_blend_mode(ctx: &mut Context, mode: BlendMode) -> GameResult<()> {
    ctx.gfx_context.set_blend_mode(mode)
}

/// Sets the window mode, such as the size and other properties.
///
/// Setting the window mode may have side effects, such as clearing
/// the screen or setting the screen coordinates viewport to some undefined value.
/// It is recommended to call `set_screen_coordinates()` after changing the window
/// size to make sure everything is what you want it to be.
pub fn set_mode(context: &mut Context, mode: WindowMode) -> GameResult<()> {
    {
        let gfx = &mut context.gfx_context;
        gfx.set_window_mode(mode)?;
    }
    {
        let video = &mut context.sdl_context.video()?;
        GraphicsContext::set_vsync(video, mode.vsync);
    }
    Ok(())
}

/// Toggles the fullscreen state of the window subsystem
///
pub fn set_fullscreen(context: &mut Context, fullscreen: bool) -> GameResult<()> {
    let fs_type = if fullscreen {
        sdl2::video::FullscreenType::True
    } else {
        sdl2::video::FullscreenType::Off
    };
    let gfx = &mut context.gfx_context;
    gfx.window.set_fullscreen(fs_type)?;

    Ok(())
}

/// Queries the fullscreen state of the window subsystem.
/// If true, then the game is running in fullscreen mode.
///
pub fn is_fullscreen(context: &mut Context) -> bool {
    let gfx = &context.gfx_context;
    gfx.window.fullscreen_state() == sdl2::video::FullscreenType::True
}

/// Sets the window resolution based on the specified width and height
///
pub fn set_resolution(context: &mut Context, width: u32, height: u32) -> GameResult<()> {
    let mut window_mode = context.conf.window_mode;
    window_mode.width = width;
    window_mode.height = height;
    set_mode(context, window_mode)
}

/// Returns a `Vec` of `(width, height)` tuples describing what
/// fullscreen resolutions are available for the given display.
pub fn get_fullscreen_modes(context: &Context, display_idx: i32) -> GameResult<Vec<(u32, u32)>> {
    let video = context.sdl_context.video()?;
    let display_count = video.num_video_displays()?;
    assert!(display_idx < display_count);

    let num_modes = video.num_display_modes(display_idx)?;

    (0..num_modes)
        .map(|i| video.display_mode(display_idx, i))
        .map(|ires| ires.map_err(GameError::VideoError))
        .map(|gres| gres.map(|dispmode| (dispmode.w as u32, dispmode.h as u32)))
        .collect()
}

/// Returns the number of connected displays.
pub fn get_display_count(context: &Context) -> GameResult<i32> {
    let video = context.sdl_context.video()?;
    video.num_video_displays().map_err(GameError::VideoError)
}

/// Returns a reference to the SDL window.
/// Ideally you should not need to use this because ggez
/// would provide all the functions you need without having
/// to dip into SDL itself.  But life isn't always ideal.
pub fn get_window(context: &Context) -> &sdl2::video::Window {
    let gfx = &context.gfx_context;
    &gfx.window
}

/// Returns a mutable reference to the SDL window.
pub fn get_window_mut(context: &mut Context) -> &mut sdl2::video::Window {
    let gfx = &mut context.gfx_context;
    &mut gfx.window
}

/// Returns the size of the window in pixels as (height, width).
pub fn get_size(context: &Context) -> (u32, u32) {
    let gfx = &context.gfx_context;
    gfx.window.size()
}

/// Returns the size of the window's underlying drawable in pixels as (height, width).
/// This may return a different value than `get_size()` when run on a platform with high-DPI support
pub fn get_drawable_size(context: &Context) -> (u32, u32) {
    let gfx = &context.gfx_context;
    gfx.window.drawable_size()
}

/// EXPERIMENTAL function to get the gfx-rs `Factory` object.
pub fn get_factory(context: &mut Context) -> &mut gfx_device_gl::Factory {
    let gfx = &mut context.gfx_context;
    &mut gfx.factory
}

/// EXPERIMENTAL function to get the gfx-rs `Device` object.
pub fn get_device(context: &mut Context) -> &mut gfx_device_gl::Device {
    let gfx = &mut context.gfx_context;
    gfx.device.as_mut()
}

/// EXPERIMENTAL function to get the gfx-rs `Encoder` object.
pub fn get_encoder(
    context: &mut Context,
) -> &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer> {
    let gfx = &mut context.gfx_context;
    &mut gfx.encoder
}

/// EXPERIMENTAL function to get the gfx-rs depth view
pub fn get_depth_view(
    context: &mut Context,
) -> gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil> {
    let gfx = &mut context.gfx_context;
    gfx.depth_view.clone()
}

/// EXPERIMENTAL function to get the gfx-rs color view
pub fn get_screen_render_target(
    context: &Context,
) -> gfx::handle::RenderTargetView<
    gfx_device_gl::Resources,
    (gfx::format::R8_G8_B8_A8, gfx::format::Srgb),
> {
    let gfx = &context.gfx_context;
    gfx.data.out.clone()
}

/// EXPERIMENTAL function to get gfx-rs objects.
/// Getting them one by one is awkward 'cause it tends to create double-borrows
/// on the Context object.
pub fn get_gfx_objects(
    context: &mut Context,
) -> (
    &mut <GlBackendSpec as BackendSpec>::Factory,
    &mut <GlBackendSpec as BackendSpec>::Device,
    &mut gfx::Encoder<<GlBackendSpec as BackendSpec>::Resources, <GlBackendSpec as BackendSpec>::CommandBuffer>,
    gfx::handle::DepthStencilView<<GlBackendSpec as BackendSpec>::Resources, gfx::format::DepthStencil>,
    gfx::handle::RenderTargetView<
        <GlBackendSpec as BackendSpec>::Resources,
        (gfx::format::R8_G8_B8_A8, gfx::format::Srgb),
    >,
) {
    let gfx = &mut context.gfx_context;
    let f = &mut gfx.factory;
    let d = gfx.device.as_mut();
    let e = &mut gfx.encoder;
    let dv = gfx.depth_view.clone();
    let cv = gfx.data.out.clone();
    (f, d, e, dv, cv)
}

// **********************************************************************
// TYPES
// **********************************************************************

/// A struct containing all the necessary info for drawing a Drawable.
///
/// This struct implements the `Default` trait, so to set only some parameter
/// you can just do:
///
/// ```rust,ignore
/// graphics::draw_ex(ctx, drawable, DrawParam{ dest: my_dest, .. Default::default()} )
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawParam {
    /// a portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image (1.0) if omitted.
    pub src: Rect,
    /// the position to draw the graphic expressed as a `Point2`.
    pub dest: Point2,
    /// orientation of the graphic in radians.
    pub rotation: f32,
    /// x/y scale factors expressed as a `Point2`.
    pub scale: Point2,
    /// specifies an offset from the center for transform operations like scale/rotation,
    /// with `0,0` meaning the origin and `1,1` meaning the opposite corner from the origin.
    /// By default these operations are done from the top-left corner, so to rotate something
    /// from the center specify `Point::new(0.5, 0.5)` here.
    pub offset: Point2,
    /// x/y shear factors expressed as a `Point2`.
    pub shear: Point2,
    /// A color to draw the target with.
    /// If `None`, the color set by `graphics::set_color()` is used; default white.
    pub color: Option<Color>,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            src: Rect::one(),
            dest: Point2::origin(),
            rotation: 0.0,
            scale: Point2::new(1.0, 1.0),
            offset: Point2::new(0.0, 0.0),
            shear: Point2::new(0.0, 0.0),
            color: None,
        }
    }
}

impl DrawParam {
    /// Turn the DrawParam into a model matrix, combining
    /// destination, rotation, scale, offset and shear.
    pub fn into_matrix(self) -> Matrix4 {
        type Vec3 = na::Vector3<f32>;
        let translate = Matrix4::new_translation(&Vec3::new(self.dest.x, self.dest.y, 0.0));
        let offset = Matrix4::new_translation(&Vec3::new(self.offset.x, self.offset.y, 0.0));
        let offset_inverse =
            Matrix4::new_translation(&Vec3::new(-self.offset.x, -self.offset.y, 0.0));
        let axang = Vec3::z() * self.rotation;
        let rotation = Matrix4::new_rotation(axang);
        let scale = Matrix4::new_nonuniform_scaling(&Vec3::new(self.scale.x, self.scale.y, 1.0));
        let shear = Matrix4::new(
            1.0,
            self.shear.x,
            0.0,
            0.0,
            self.shear.y,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );
        translate * offset * rotation * shear * scale * offset_inverse
    }
}

/// All types that can be drawn on the screen implement the `Drawable` trait.
pub trait Drawable {
    /// Actually draws the object to the screen.
    ///
    /// This is the most general version of the operation, which is all that
    /// is required for implementing this trait.
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()>;

    /// Draws the drawable onto the rendering target.
    ///
    /// It just is a shortcut that calls `draw_ex()` with a default `DrawParam`
    /// except for the destination and rotation.
    ///
    /// * `ctx` - The `Context` this graphic will be rendered to.
    /// * `dest` - the position to draw the graphic expressed as a `Point2`.
    /// * `rotation` - orientation of the graphic in radians.
    ///
    fn draw(&self, ctx: &mut Context, dest: Point2, rotation: f32) -> GameResult<()> {
        self.draw_ex(
            ctx,
            DrawParam {
                dest: dest,
                rotation: rotation,
                ..Default::default()
            },
        )
    }

    /// Sets the blend mode to be used when drawing this drawable.
    /// This overrides the general `graphics::set_blend_mode()`.
    /// If `None` is set, defers to the blend mode set by
    /// `graphics::set_blend_mode()`.
    fn set_blend_mode(&mut self, mode: Option<BlendMode>);

    /// Gets the blend mode to be used when drawing this drawable.
    fn get_blend_mode(&self) -> Option<BlendMode>;
}

/// Generic in-GPU-memory image data available to be drawn on the screen.
#[derive(Clone)]
pub struct ImageGeneric<R>
where
    R: gfx::Resources,
{
    texture: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    sampler_info: gfx::texture::SamplerInfo,
    blend_mode: Option<BlendMode>,
    width: u32,
    height: u32,
}

/// In-GPU-memory image data available to be drawn on the screen,
/// using the OpenGL backend.
///
/// Note that this is really just an `Arc`'ed texture handle and some metadata,
/// so `clone()`'ing it is quite cheap since it doesn't need to actually
/// copy the image data.
pub type Image = ImageGeneric<gfx_device_gl::Resources>;

impl Image {
    /// Load a new image from the file at the given path.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Image> {
        let img = {
            let mut buf = Vec::new();
            let mut reader = context.filesystem.open(path)?;
            reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_rgba()
        };
        let (width, height) = img.dimensions();
        Image::from_rgba8(context, width as u16, height as u16, &img)
    }

    /// Creates a new `Image` from the given buffer of `u8` RGBA values.
    pub fn from_rgba8(
        context: &mut Context,
        width: u16,
        height: u16,
        rgba: &[u8],
    ) -> GameResult<Image> {
        Image::make_raw(
            &mut context.gfx_context.factory,
            &context.gfx_context.default_sampler_info,
            width,
            height,
            rgba,
        )
    }
    /// A helper function that just takes a factory directly so we can make an image
    /// without needing the full context object, so we can create an Image while still
    /// creating the GraphicsContext.
    fn make_raw(
        factory: &mut gfx_device_gl::Factory,
        sampler_info: &texture::SamplerInfo,
        width: u16,
        height: u16,
        rgba: &[u8],
    ) -> GameResult<Image> {
        if width == 0 || height == 0 {
            let msg = format!(
                "Tried to create a texture of size {}x{}, each dimension must
                be >0",
                width, height
            );
            return Err(GameError::ResourceLoadError(msg));
        }
        let kind = gfx::texture::Kind::D2(width, height, gfx::texture::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, gfx::texture::Mipmap::Provided, &[rgba])?;
        Ok(Image {
            texture: view,
            sampler_info: *sampler_info,
            blend_mode: None,
            width: u32::from(width),
            height: u32::from(height),
        })
    }

    /// A little helper function that creates a new Image that is just
    /// a solid square of the given size and color.  Mainly useful for
    /// debugging.
    pub fn solid(context: &mut Context, size: u16, color: Color) -> GameResult<Image> {
        // let pixel_array: [u8; 4] = color.into();
        let (r, g, b, a) = color.into();
        let pixel_array: [u8; 4] = [r, g, b, a];
        let size_squared = size as usize * size as usize;
        let mut buffer = Vec::with_capacity(size_squared);
        for _i in 0..size_squared {
            buffer.extend(&pixel_array[..]);
        }
        Image::from_rgba8(context, size, size, &buffer)
    }

    /// Return the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Return the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the filter mode for the image.
    pub fn get_filter(&self) -> FilterMode {
        self.sampler_info.filter.into()
    }

    /// Set the filter mode for the image.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.sampler_info.filter = mode.into();
    }

    /// Returns the dimensions of the image.
    pub fn get_dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
    }

    /// Gets the `Image`'s `WrapMode` along the X and Y axes.
    pub fn get_wrap(&self) -> (WrapMode, WrapMode) {
        (self.sampler_info.wrap_mode.0, self.sampler_info.wrap_mode.1)
    }

    /// Sets the `Image`'s `WrapMode` along the X and Y axes.
    pub fn set_wrap(&mut self, wrap_x: WrapMode, wrap_y: WrapMode) {
        self.sampler_info.wrap_mode.0 = wrap_x;
        self.sampler_info.wrap_mode.1 = wrap_y;
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<Image: {}x{}, {:p}, texture address {:p}, sampler: {:?}>",
            self.width(),
            self.height(),
            self,
            &self.texture,
            &self.sampler_info
        )
    }
}

impl Drawable for Image {
    fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
        let gfx = &mut ctx.gfx_context;
        let src_width = param.src.w;
        let src_height = param.src.h;
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let real_scale = Point2::new(
            src_width * param.scale.x * self.width as f32,
            src_height * param.scale.y * self.height as f32,
        );
        let mut new_param = param;
        new_param.scale = real_scale;
        gfx.update_instance_properties(new_param)?;
        let sampler = gfx.samplers
            .get_or_insert(self.sampler_info, gfx.factory.as_mut());
        gfx.data.vbuf = gfx.quad_vertex_buffer.clone();
        gfx.data.tex = (self.texture.clone(), sampler);
        let previous_mode: Option<BlendMode> = if let Some(mode) = self.blend_mode {
            let current_mode = gfx.get_blend_mode();
            if current_mode != mode {
                gfx.set_blend_mode(mode)?;
                Some(current_mode)
            } else {
                None
            }
        } else {
            None
        };
        gfx.draw(None)?;
        if let Some(mode) = previous_mode {
            gfx.set_blend_mode(mode)?;
        }
        Ok(())
    }
    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }
    fn get_blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}

#[cfg(test)]
mod tests {}
