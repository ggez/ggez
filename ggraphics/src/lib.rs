//! A basic rendering library designed to run on desktop and in
//! web browsers.  Uses the `glow` crate for OpenGL, WebGL and
//! OpenGL ES.
//!
//! For now, this is mostly an implementation detail of the `ggez`
//! crate.
//!
//! Note this is deliberately NOT thread-safe, because threaded
//! OpenGL is not worth the trouble.  Create your OpenGL context
//! on a particular thread and do all your rendering from that
//! thread.
//! See
//! <https://github.com/FNA-XNA/FNA/blob/76554b7ca3d7aa33229c12c6ab5bf3dbdb114d59/src/FNAPlatform/OpenGLDevice.cs#L10-L39> for more info
//!
//! TODO:
//!  * Unheck mesh pipelines/batches
//!  * Unheck pipelines in general a little
//!  * Unheck render passes a little, render to texture probably doesn't work
//!  * Maybe make all these things store fewer vec's and instead get them passed in
//!  * Look at all other todo's and figure out wtf to do
//!
//! Ok, so for OpenGL ES 2 we don't have instancing.  This changes everything.
//! So let's start with meshes.  Each mesh instance is going to have to be drawn with
//! its own draw call.  So for now let's just collect a list of meshes and
//! do that, minimizing state transitions.  As an optimization, eventually we're
//! going to try, for small meshes, to pre-compute transforms on the CPU and
//! moosh the vertices all into one mesh.  But for now we're purely in suck-it
//! -and-see mode.

// Next up:
// Impl mesh pipelines
// Blend modes!
// Try out termhn's wireframe shader with barycentric coords
// audit unsafes, figure out what can be safe,

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![warn(bare_trait_objects)]
#![warn(missing_copy_implementations)]

use std::cell::RefCell;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::mem;
use std::rc::Rc;

/// Re-export
pub use glow;

use bytemuck;
use glam::Mat4;
use glow::*;
use log::*;

// Shortcuts for various OpenGL types.

type GlTexture = <Context as glow::HasContext>::Texture;
//type GlSampler = <Context as glow::HasContext>::Sampler;
type GlProgram = <Context as glow::HasContext>::Program;
//type GlVertexArray = <Context as glow::HasContext>::VertexArray;
type GlFramebuffer = <Context as glow::HasContext>::Framebuffer;
type GlRenderbuffer = <Context as glow::HasContext>::Renderbuffer;
type GlBuffer = <Context as glow::HasContext>::Buffer;
type GlUniformLocation = <Context as glow::HasContext>::UniformLocation;

/// A type that contains all the STUFF we need for displaying graphics
/// and handling events on both desktop and web.
/// Anything it contains is specialized to the correct type via cfg flags
/// at compile time, rather than trying to use generics or such.
#[derive(Debug)]
pub struct GlContext {
    /// The OpenGL context.
    pub gl: Rc<glow::Context>,
    // /// The list of render passes.
    //pub passes: Vec<RenderPass>,
    /// Samplers are cached and managed entirely by the GlContext.
    /// You usually only need a few of them so there's no point freeing
    /// them separately, you just ask for the one you want and it gives
    /// it to you.
    samplers: RefCell<HashSet<SamplerSpec>>,
    default_shader: Shader,
}

const VERTEX_SHADER_SOURCE: &str = include_str!("data/quad_es100.vert.glsl");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("data/quad_es100.frag.glsl");

impl GlContext {
    /// Create a new `GlContext` from the given `glow::Context`.  Does
    /// basic setup and state setting.
    pub fn new(gl: glow::Context) -> Self {
        // GL SETUP
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            let gl = Rc::new(gl);
            let default_shader =
                ShaderHandle::new_raw(gl.clone(), VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
            let s = GlContext {
                gl,
                samplers: RefCell::new(HashSet::new()),
                default_shader: default_shader.into_shared(),
            };
            s.register_debug_callback();

            s
        }
    }

    /// Get a copy of the default quad shader.
    pub fn default_shader(&self) -> Shader {
        self.default_shader.clone()
    }

    /// Log OpenGL errors as possible.
    /// TODO: Figure out why this panics on wasm
    fn register_debug_callback(&self) {
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        unsafe {
            self.gl
                .debug_message_callback(|source, typ, id, severity, message| {
                    // The ordering of severities is basically awful, best to
                    // not even try and just match them manually.
                    match severity {
                        glow::DEBUG_SEVERITY_HIGH => {
                            error!(
                                "GL error type {} id {} from {}: {}",
                                typ, id, source, message
                            );
                        }
                        glow::DEBUG_SEVERITY_MEDIUM => {
                            warn!(
                                "GL error type {} id {} from {}: {}",
                                typ, id, source, message
                            );
                        }
                        glow::DEBUG_SEVERITY_LOW => {
                            info!(
                                "GL error type {} id {} from {}: {}",
                                typ, id, source, message
                            );
                        }
                        glow::DEBUG_SEVERITY_NOTIFICATION => (),
                        _ => (),
                    }
                });
        }
    }

    /*
    /// Get a sampler given the given spec.  Samplers are cached, and
    /// usually few in number, so you shouldn't free them and this handles
    /// caching them for you.
    pub fn get_sampler(&self, spec: &SamplerSpec) -> GlSampler {
        let gl = &*self.gl;
        // unsafety: This takes no inputs besides spec, which has
        // constrained types.
        *self
            .samplers
            .borrow_mut() // We don't store this borrow anywhere, so it should never panic
            .entry(*spec)
            .or_insert_with(|| unsafe {
                let sampler = gl.create_sampler().unwrap();
                gl.sampler_parameter_i32(
                    sampler,
                    glow::TEXTURE_MIN_FILTER,
                    spec.min_filter.to_gl() as i32,
                );
                gl.sampler_parameter_i32(
                    sampler,
                    glow::TEXTURE_MAG_FILTER,
                    spec.mag_filter.to_gl() as i32,
                );
                gl.sampler_parameter_i32(sampler, glow::TEXTURE_WRAP_S, spec.wrap.to_gl() as i32);
                gl.sampler_parameter_i32(sampler, glow::TEXTURE_WRAP_T, spec.wrap.to_gl() as i32);
                sampler
            })
    }
    */

    /// Returns OpenGL version info.
    /// Vendor, renderer, GL version, GLSL version
    pub fn get_info(&self) -> (String, String, String, String) {
        unsafe {
            let vendor = self.gl.get_parameter_string(glow::VENDOR);
            let rend = self.gl.get_parameter_string(glow::RENDERER);
            let vers = self.gl.get_parameter_string(glow::VERSION);
            let glsl_vers = self.gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION);

            (vendor, rend, vers, glsl_vers)
        }
    }
}

/// This is actually not safe to Clone, we'd have to Rc the GlTexture.
/// Having the Rc on the *outside* of this type is what we actually want.
#[derive(Debug)]
pub struct TextureHandle {
    ctx: Rc<glow::Context>,
    tex: GlTexture,
}

impl PartialEq for TextureHandle {
    fn eq(&self, rhs: &Self) -> bool {
        // TODO: Compare ctx?  Is it worth it?  idk.
        self.tex == rhs.tex
    }
}

impl Drop for TextureHandle {
    fn drop(&mut self) {
        unsafe {
            self.ctx.delete_texture(self.tex);
        }
    }
}

/// A shared, clone-able texture type.
pub type Texture = Rc<TextureHandle>;

impl TextureHandle {
    /// Create a new texture from the given slice of RGBA bytes.
    pub fn new(ctx: &GlContext, rgba: &[u8], width: usize, height: usize) -> Self {
        assert_eq!(width * height * 4, rgba.len());
        let gl = &*ctx.gl;
        // Unsafety: Takes no user inputs.
        let t = unsafe { gl.create_texture().unwrap() };
        let mut tex = Self {
            ctx: ctx.gl.clone(),
            tex: t,
        };
        tex.replace(rgba, width, height);
        tex
    }

    /// Make a new empty texture with the given format.  Note that reading from the texture
    /// will give undefined results until it is filled with data, hence why this is unsafe.
    pub unsafe fn new_empty(
        ctx: &GlContext,
        format: u32,
        component_format: u32,
        width: usize,
        height: usize,
    ) -> Self {
        let gl = &*ctx.gl;
        let t = gl.create_texture().unwrap();
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(t));
        gl.tex_image_2d(
            glow::TEXTURE_2D,               // Texture target
            0,                              // mipmap level
            i32::try_from(format).unwrap(), // format to store the texture in (can't fail)
            i32::try_from(width).unwrap(),  // width
            i32::try_from(height).unwrap(), // height
            0,                              // border, must always be 0, lulz
            format,                         // format to load the texture from
            component_format,               // Type of each color element
            None,                           // Actual data
        );

        gl.bind_texture(glow::TEXTURE_2D, None);
        Self {
            ctx: ctx.gl.clone(),
            tex: t,
        }
    }

    /// Replace the existing texture with the given slice of RGBA bytes.
    pub fn replace(&mut self, rgba: &[u8], width: usize, height: usize) {
        assert_eq!(width * height * 4, rgba.len());
        let gl = &*self.ctx;
        // Unsafety: Same as `new()`
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,                   // Texture target
                0,                                  // mipmap level
                i32::try_from(glow::RGBA).unwrap(), // format to store the texture in (can't fail)
                i32::try_from(width).unwrap(),      // width
                i32::try_from(height).unwrap(),     // height
                0,                                  // border, must always be 0, lulz
                glow::RGBA,                         // format to load the texture from
                glow::UNSIGNED_BYTE,                // Type of each color element
                Some(rgba),                         // Actual data
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Replace part of the texture with the slice of RGBA bytes.
    pub fn replace_subimage(
        &mut self,
        rgba: &[u8],
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) {
        //assert_eq!(width * height * 4, rgba.len());
        // TODO: asserts on offsets, sizes, etc.
        let gl = &*self.ctx;
        // Unsafety: Same as `new()`
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,                   // Texture target
                0,                                  // mipmap level
                i32::try_from(x_offset).unwrap(),   // xoffset
                i32::try_from(y_offset).unwrap(),   // yoffset
                i32::try_from(width).unwrap(),      // width
                i32::try_from(height).unwrap(),     // height
                glow::RGBA,                         // format to load the texture from
                glow::UNSIGNED_BYTE,                // Type of each color element
                glow::PixelUnpackData::Slice(rgba), // Actual data
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Turn this texture handle into a share-able, refcounted one.
    pub fn into_shared(self) -> Texture {
        Rc::new(self)
    }
}

/// Similar to `TextureHandle`, this
/// is a shader resource that can be Rc'ed and shared.
#[derive(Debug)]
pub struct ShaderHandle {
    ctx: Rc<glow::Context>,
    program: GlProgram,
}

impl Drop for ShaderHandle {
    fn drop(&mut self) {
        unsafe {
            self.ctx.delete_program(self.program);
        }
    }
}

/// A share-able refcounted shader.
pub type Shader = Rc<ShaderHandle>;

impl ShaderHandle {
    fn new_raw(gl: Rc<glow::Context>, vertex_src: &str, fragment_src: &str) -> ShaderHandle {
        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_src),
            (glow::FRAGMENT_SHADER, fragment_src),
        ];

        // TODO: Audit unsafe
        unsafe {
            let program = gl.create_program().expect("Cannot create program");
            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!(gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!(gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }
            // TODO:
            // By default only one output so this isn't necessary
            // glBindFragDataLocation(shaderProgram, 0, "outColor");
            ShaderHandle { ctx: gl, program }
        }
    }

    /// Create new shader
    pub fn new(ctx: &GlContext, vertex_src: &str, fragment_src: &str) -> ShaderHandle {
        Self::new_raw(ctx.gl.clone(), vertex_src, fragment_src)
    }

    /// Turn this shader into a clone-able, refcounted one.
    pub fn into_shared(self) -> Shader {
        Rc::new(self)
    }
}

/// A vertex in a mesh.  Other vertex types can be defined, but then you
/// have to make your own version of MeshPipeline to go with them.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Vertex {
    /// Position
    pub pos: [f32; 4],
    /// Color for that vertex
    pub color: [f32; 4],
    /// Normal
    pub normal: [f32; 4],
    /// UV coordinates, if any
    pub uv: [f32; 2],
}

unsafe impl bytemuck::Zeroable for Vertex {}

unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    /// Returns an empty `Vertex` with default values.
    ///
    /// TODO: The normal is pretty bogus, though maybe less
    /// bogus than a zero vector?
    pub const fn empty() -> Self {
        Vertex {
            pos: [0.0, 0.0, 0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
            normal: [0.0, 0.0, 0.0, 0.0],
            uv: [0.0, 0.0],
        }
    }

    /// Returns a Vec of (element offset, element size)
    /// pairs.  This is proooobably technically a little UB,
    /// see https://github.com/rust-lang/rust/issues/48956#issuecomment-544506419
    /// but with repr(C) it's probably safe enough.
    ///
    /// Also returns the name of the shader variable associated with each field...
    unsafe fn layout() -> Vec<(&'static str, usize, usize)> {
        // It'd be nice if we could make this `const` but
        // doing const pointer arithmatic is unstable.
        let thing = Vertex::empty();
        let thing_base = &thing as *const _;
        let pos_offset = (&thing.pos as *const _ as usize) - thing_base as usize;
        let pos_size = mem::size_of_val(&thing.pos);

        let normal_offset = (&thing.normal as *const _ as usize) - thing_base as usize;
        let normal_size = mem::size_of_val(&thing.normal);

        let color_offset = (&thing.color as *const _ as usize) - thing_base as usize;
        let color_size = mem::size_of_val(&thing.color);

        let uv_offset = (&thing.uv as *const _ as usize) - thing_base as usize;
        let uv_size = mem::size_of_val(&thing.uv);

        vec![
            ("a_pos", pos_offset, pos_size),
            ("a_normal", normal_offset, normal_size),
            ("a_color", color_offset, color_size),
            ("a_uv", uv_offset, uv_size),
        ]
    }
}

/// Similar to a `TextureHandle`, this is drawable geometry with a given layout.
#[derive(Debug)]
pub struct MeshHandle {
    ctx: Rc<glow::Context>,
    /// Vertex buffer array -- contains verts
    vbo: GlBuffer,
    /// Element buffer array -- contains indices
    ebo: GlBuffer,
    /// Number of indices to draw in the mesh
    count: u16,
}

impl PartialEq for MeshHandle {
    fn eq(&self, rhs: &Self) -> bool {
        // TODO: Compare ctx?  Is it worth it?  idk.
        (self.vbo == rhs.vbo) && (self.ebo == rhs.ebo)
    }
}

impl Drop for MeshHandle {
    fn drop(&mut self) {
        unsafe {
            self.ctx.delete_buffer(self.vbo);
            self.ctx.delete_buffer(self.ebo);
        }
    }
}

/// A shared, clone-able texture type.
pub type Mesh = Rc<MeshHandle>;

impl MeshHandle {
    /// Create a new texture from the given slice of `Vertex`'s, with the given index array
    pub fn new(ctx: &GlContext, verts: &[Vertex], indices: &[u16]) -> Self {
        assert!(verts.len() > 0);
        assert!(indices.len() > 0);
        let gl = &*ctx.gl;
        unsafe {
            let vbo = gl.create_buffer().unwrap();
            let ebo = gl.create_buffer().unwrap();
            // Upload verts
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let bytes_slice: &[u8] = bytemuck::try_cast_slice(verts).unwrap();
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes_slice, glow::STATIC_DRAW);

            // Upload indices
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            let bytes_slice: &[u8] = bytemuck::try_cast_slice(indices).unwrap();
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytes_slice, glow::STATIC_DRAW);

            MeshHandle {
                ctx: ctx.gl.clone(),
                vbo,
                ebo,
                count: indices.len() as u16,
            }
        }
    }
    /// Turn this mesh into a share-able, refcounted one.
    pub fn into_shared(self) -> Mesh {
        Rc::new(self)
    }
}

/// Filter modes a sampler may have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterMode {
    /// Nearest-neighbor filtering.  Use this for pixel-y effects.
    Nearest,
    /// Linear filtering.  Use this for smooth effects.
    Linear,
}

impl FilterMode {
    /// Turns the filter mode into the appropriate OpenGL enum
    fn to_gl(self) -> u32 {
        match self {
            FilterMode::Nearest => glow::NEAREST,
            FilterMode::Linear => glow::LINEAR,
        }
    }
}

/// Wrap modes a sampler may have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WrapMode {
    /// Clamp colors to the edge of the texture.
    Clamp,
    /// Tile/repeat the texture.
    Tile,
    /// Mirror the texture.
    Mirror,
}

impl WrapMode {
    /// Turns the wrap mode into the appropriate OpenGL enum
    fn to_gl(self) -> u32 {
        match self {
            WrapMode::Clamp => glow::CLAMP_TO_EDGE,
            WrapMode::Tile => glow::REPEAT,
            WrapMode::Mirror => glow::MIRRORED_REPEAT,
        }
    }
}

/// TODO
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {}

/// A description of a sampler.  We cache the actual
/// samplers as needed in the GlContext.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SamplerSpec {
    min_filter: FilterMode,
    mag_filter: FilterMode,
    wrap: WrapMode,
}

impl SamplerSpec {
    /// Shortcut for creating a new `SamplerSpec`.
    pub fn new(min: FilterMode, mag: FilterMode, wrap: WrapMode) -> Self {
        Self {
            min_filter: min,
            mag_filter: mag,
            wrap,
        }
    }
}

impl Default for SamplerSpec {
    fn default() -> Self {
        Self::new(FilterMode::Nearest, FilterMode::Nearest, WrapMode::Tile)
    }
}

/// Per-mesh uniform data for a mesh draw call.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct MeshInstance {
    /// Transform to apply to the verts in the mesh
    pub model_transform: [f32; 16],
}

unsafe impl bytemuck::Zeroable for MeshInstance {}

unsafe impl bytemuck::Pod for MeshInstance {}

impl MeshInstance {
    /// Returns an empty `MeshInstance` with default values.
    pub const fn empty() -> Self {
        MeshInstance {
            //model_transform: [0.0; 16],
            model_transform: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }

    /// Returns a Vec of (element offset, element size)
    /// pairs.  This is proooobably technically a little UB,
    /// see https://github.com/rust-lang/rust/issues/48956#issuecomment-544506419
    /// but with repr(C) it's probably safe enough.
    ///
    /// Also returns the name of the shader variable associated with each field...
    unsafe fn layout() -> Vec<(&'static str, usize, usize)> {
        // It'd be nice if we could make this `const` but
        // doing const pointer arithmatic is unstable.
        let thing = MeshInstance::empty();
        let thing_base = &thing as *const MeshInstance;
        let model_transform_offset =
            (&thing.model_transform as *const _ as usize) - thing_base as usize;
        let model_transform_size = mem::size_of_val(&thing.model_transform);

        vec![(
            "u_ModelTransform",
            model_transform_offset,
            model_transform_size,
        )]
    }
}

/// Trait for a draw call...
pub trait Batch {
    /// Type of the instance data contained by the batch
    type Instance: bytemuck::Pod + bytemuck::Zeroable;
    /// Add a new instance to the quad data.
    /// Instances are cached between `draw()` invocations.
    fn add(&mut self, quad: Self::Instance);

    /// Empty all instances out of the instance buffer.
    fn clear(&mut self);
    /// Draw all the instances at once.
    unsafe fn draw(&mut self, gl: &Context);
}

/// A mesh that will be drawn multiple times with a single draw call.
///
/// Since we don't actually have instanced drawing in ES 2, we still loop
/// over a bunch of `MeshInstance` "instances" doing a separate draw call
/// for each, but use the same sampler, texture etc for all of them to
/// minimize state transitions.
#[derive(Debug)]
pub struct MeshBatch {
    ctx: Rc<GlContext>,
    texture: Texture,
    mesh: Mesh,
    sampler: SamplerSpec,
    /// The "instances" that will be drawn.
    pub instances: Vec<MeshInstance>,
    texture_location: GlUniformLocation,

    inst_uniform_location: GlUniformLocation,
}

impl MeshBatch {
    /// Sets all the vertex pointers for the MeshInstance
    /// TODO: ...this is wrong, we need to set vertex pointers for the Vertex.
    unsafe fn set_vertex_pointers(ctx: &GlContext, shader: &ShaderHandle) {
        let gl = &*ctx.gl;
        let layout = MeshInstance::layout();
        for (name, offset, size) in layout {
            info!("Layout: {} offset, {} size", offset, size);
            let element_size = mem::size_of::<f32>();
            let attrib_location = gl.get_attrib_location(shader.program, name).unwrap();
            gl.vertex_attrib_pointer_f32(
                attrib_location,
                (size / element_size) as i32,
                glow::FLOAT,
                false,
                mem::size_of::<MeshInstance>() as i32,
                offset as i32,
            );
            gl.enable_vertex_attrib_array(attrib_location);
        }
    }

    /// New empty `MeshBatch` using the given pipeline.
    pub fn new(
        ctx: Rc<GlContext>,
        texture: Texture,
        mesh: Mesh,
        sampler: SamplerSpec,
        pipeline: &MeshPipeline,
    ) -> Self {
        let gl = &*ctx.gl;
        // TODO: Audit unsafe
        unsafe {
            // Here we tell our VAO what properties our various VBO's contain
            // First, set mesh VBO...
            // pos, color, norm, uv,
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(mesh.vbo));
            /*
            let attrib_location = gl
                .get_attrib_location(pipeline.shader.program, "pos")
                .unwrap();
            gl.vertex_attrib_pointer_f32(
                attrib_location,
                4, // TODO (size / element_size) as i32,
                glow::FLOAT,
                false,
                mem::size_of::<Vertex>() as i32,
                offset as i32,
            );
            gl.enable_vertex_attrib_array(attrib_location);
            */

            // Then set mesh EBO (index buffer)
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(mesh.ebo));
            /*
            let attrib_location = gl
                .get_attrib_location(pipeline.shader.program, "pos")
                .unwrap();
            gl.vertex_attrib_pointer_f32(
                attrib_location,
                4, // TODO (size / element_size) as i32,
                glow::FLOAT,
                false,
                mem::size_of::<Vertex>() as i32,
                offset as i32,
            );
            gl.enable_vertex_attrib_array(attrib_location);
            */

            let layout = Vertex::layout();
            for (name, offset, size) in layout {
                info!("Layout: {} offset, {} size", offset, size);
                let element_size = mem::size_of::<f32>();
                let attrib_location = gl
                    .get_attrib_location(pipeline.shader.program, name)
                    // TODO: make this not always need a format!()
                    .expect(&format!("Unknown attrib name in shader: {}", name));
                gl.vertex_attrib_pointer_f32(
                    attrib_location,
                    (size / element_size) as i32,
                    glow::FLOAT,
                    false,
                    mem::size_of::<Vertex>() as i32,
                    offset as i32,
                );
                gl.enable_vertex_attrib_array(attrib_location);
            }

            // TODO: We have some uniforms that are set per-batch,
            // and some that are set per-instance.  Currently we just kinda
            // mongle them both.
            let texture_location = gl
                .get_uniform_location(pipeline.shader.program, "u_Tex")
                .unwrap();

            let inst_uniform_location = gl
                .get_uniform_location(pipeline.shader.program, "u_ModelTransform")
                .unwrap();

            Self {
                ctx: ctx,
                mesh,
                texture,
                sampler,
                texture_location,
                inst_uniform_location,
                instances: vec![],
            }
        }
    }

    /// Add a new instance to the quad data.
    /// Instances are cached between `draw()` invocations.
    pub fn add(&mut self, mesh: MeshInstance) {
        self.instances.push(mesh);
    }

    /// Empty all instances out of the instance buffer.
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    unsafe fn draw(&mut self, gl: &Context) {
        // Bind texture
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.texture.tex));
        gl.uniform_1_i32(Some(&self.texture_location), 0);

        // Set sampler info
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            self.sampler.min_filter.to_gl() as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            self.sampler.mag_filter.to_gl() as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            self.sampler.wrap.to_gl() as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            self.sampler.wrap.to_gl() as i32,
        );

        // Bind buffers
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.mesh.vbo));
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.mesh.ebo));
        // Draw stuff in a for loop, like a peasant.
        for i in &self.instances {
            gl.uniform_matrix_4_f32_slice(
                Some(&self.inst_uniform_location),
                false,
                &i.model_transform,
            );
            // TODO: Currently we don't actually use indices, we only want
            // to for large meshes.
            //gl.draw_arrays(glow::TRIANGLES, 0, (self.instances.len() * 3) as i32);
            gl.draw_elements(
                glow::TRIANGLES,
                self.mesh.count as i32,
                glow::UNSIGNED_SHORT,
                0,
            );
        }
    }
}

impl Batch for MeshBatch {
    type Instance = MeshInstance;
    /// TODO: Refactor
    fn add(&mut self, quad: Self::Instance) {
        self.add(quad);
    }

    /// TODO: Refactor
    fn clear(&mut self) {
        self.clear();
    }

    /// TODO: Refactor
    unsafe fn draw(&mut self, gl: &Context) {
        self.draw(gl);
    }
}

/// TODO: Docs
/// hnyrn
pub trait Pipeline: std::fmt::Debug {
    /// Ideally we should be able to get rid of this...
    type Instance: bytemuck::Pod + bytemuck::Zeroable;
    /// foo
    unsafe fn draw(&mut self, gl: &Context);
    /// foo
    fn new_batch(&mut self, texture: Texture, sampler: SamplerSpec) -> &mut dyn Batch<Instance = Self::Instance>;
    /// this seems the way to do it...
    fn get(&self, idx: usize) -> &dyn Batch<Instance = Self::Instance>;
    /// Get mut
    fn get_mut(&mut self, idx: usize) -> &mut dyn Batch<Instance = Self::Instance>;
    /// clear all draw calls
    fn clear(&mut self);
    ///  Returns iterator of batches.  The lifetimes are a PITA.
    fn batches<'a>(&'a self) -> Box<dyn Iterator<Item = &'a (dyn Batch<Instance = Self::Instance> + 'a)> + 'a>;
    ///  Returns iterator of mutable batches.  The lifetimes are a PITA.
    fn batches_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut (dyn Batch<Instance = Self::Instance> + 'a)> + 'a>;
}

/// A pipeline for drawing arbitrary meshes
#[derive(Debug)]
pub struct MeshPipeline {
    ctx: Rc<GlContext>,
    /// The draw calls in the pipeline.
    pub batches: Vec<MeshBatch>,
    /// The projection the pipeline will draw with.
    pub projection: Mat4,
    shader: Shader,
    projection_location: GlUniformLocation,
}

impl MeshPipeline {
    /// Create new pipeline with the given shader.
    pub unsafe fn new(ctx: Rc<GlContext>, shader: Shader) -> Self {
        let gl = &*ctx.gl;
        //let projection = Mat4::identity();
        let projection = Mat4::orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, 1.0, -1.0);
        let projection_location = gl
            .get_uniform_location(shader.program, "u_Projection")
            .expect("shader does not have a u_Projection uniform for some reason, or we can't find it.  Is GLSL being braindead and removing 'unused' globals again?");
        Self {
            ctx,
            batches: vec![],
            shader,
            projection,
            projection_location,
        }
    }

    /// Draw all the draw calls in the pipeline.
    pub unsafe fn draw(&mut self, gl: &Context) {
        gl.use_program(Some(self.shader.program));
        gl.uniform_matrix_4_f32_slice(
            Some(&self.projection_location),
            false,
            &self.projection.to_cols_array(),
        );
        for dc in self.batches.iter_mut() {
            dc.draw(gl);
        }
    }
}

/// aaaaa
/// TODO: Docs
#[derive(Debug)]
pub struct MeshPipelineIter<'a> {
    i: std::slice::Iter<'a, MeshBatch>,
}

impl<'a> MeshPipelineIter<'a> {
    /// TODO: Docs
    pub fn new(p: &'a MeshPipeline) -> Self {
        Self {
            i: p.batches.iter(),
        }
    }
}

impl<'a> Iterator for MeshPipelineIter<'a> {
    type Item = &'a dyn Batch<Instance = MeshInstance>;
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| x as _)
    }
}

/// Sigh
/// TODO: Docs
#[derive(Debug)]
pub struct MeshPipelineIterMut<'a> {
    i: std::slice::IterMut<'a, MeshBatch>,
}

impl<'a> MeshPipelineIterMut<'a> {
    /// TODO: Docs
    pub fn new(p: &'a mut MeshPipeline) -> Self {
        Self {
            i: p.batches.iter_mut(),
        }
    }
}

impl<'a> Iterator for MeshPipelineIterMut<'a> {
    type Item = &'a mut dyn Batch<Instance = MeshInstance>;
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| x as _)
    }
}

impl Pipeline for MeshPipeline {
    type Instance = MeshInstance;
    /// foo
    /// TODO: Docs
    unsafe fn draw(&mut self, gl: &Context) {
        self.draw(gl);
    }
    /// foo
    fn new_batch(&mut self, texture: Texture, sampler: SamplerSpec) -> &mut dyn Batch<Instance = MeshInstance> {
        // BUGGO TODO: use real mesh and shaader here
        let dummy_mesh = MeshHandle::new(&self.ctx, &[], &[]).into_shared();
        let x = MeshBatch::new(self.ctx.clone(), texture, dummy_mesh, sampler, self);
        self.batches.push(x);
        &mut *self.batches.last_mut().unwrap()
    }

    fn clear(&mut self) {
        self.batches.clear()
    }
    fn get(&self, idx: usize) -> &dyn Batch<Instance = MeshInstance> {
        &self.batches[idx]
    }
    fn get_mut(&mut self, idx: usize) -> &mut dyn Batch <Instance = MeshInstance>{
        &mut self.batches[idx]
    }

    fn batches<'a>(&'a self) -> Box<dyn Iterator<Item = &'a (dyn Batch<Instance = MeshInstance> + 'a)> + 'a> {
        let i = MeshPipelineIter::new(self);
        Box::new(i)
    }

    fn batches_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut (dyn Batch<Instance = MeshInstance> + 'a)> + 'a> {
        let i = MeshPipelineIterMut::new(self);
        Box::new(i)
    }
}

/// A render target for drawing to a texture.
#[derive(Debug)]
pub struct TextureRenderTarget {
    ctx: Rc<glow::Context>,
    output_framebuffer: GlFramebuffer,
    output_texture: Texture,
    _output_depthbuffer: GlRenderbuffer,
}

impl Drop for TextureRenderTarget {
    fn drop(&mut self) {
        unsafe {
            self.ctx.delete_framebuffer(self.output_framebuffer);
            self.ctx.delete_renderbuffer(self._output_depthbuffer);
        }
    }
}

impl TextureRenderTarget {
    /// Create a new render target rendering to a texture.
    pub unsafe fn new(ctx: &GlContext, width: usize, height: usize) -> Self {
        let gl = &*ctx.gl;

        let t = TextureHandle::new_empty(ctx, glow::RGBA, glow::UNSIGNED_BYTE, width, height)
            .into_shared();
        let depth = gl.create_renderbuffer().unwrap();
        let fb = gl.create_framebuffer().unwrap();

        // Now we have our color texture, depth buffer and framebuffer, and we
        // glue them all together.
        gl.bind_texture(glow::TEXTURE_2D, Some(t.tex));
        // We need to add filtering params to the texture, for Reasons.
        // We might be able to use samplers instead, but not yet.
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );

        /*
         * TODO: This panics for some reason
        gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth));
        gl.renderbuffer_storage(
            glow::RENDERBUFFER,
            glow::DEPTH_COMPONENT,
            width as i32,
            height as i32,
        );
        */

        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(t.tex),
            0,
        );
        /*
         * TODO: This panics for some reason
        gl.framebuffer_renderbuffer(
            glow::FRAMEBUFFER,
            glow::DEPTH_ATTACHMENT,
            glow::RENDERBUFFER,
            Some(depth),
        );
        */

        // Set list of draw buffers
        let draw_buffers = &[glow::COLOR_ATTACHMENT0];
        gl.draw_buffers(draw_buffers);

        // Verify results
        if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer hecked up");
        }

        // Reset heckin bindings
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.bind_texture(glow::TEXTURE_2D, None);
        Self {
            ctx: ctx.gl.clone(),
            output_framebuffer: fb,
            output_texture: t,
            _output_depthbuffer: depth,
        }
    }
}

/// The options for what a render pass can write to.
#[derive(Debug)]
pub enum RenderTarget {
    /// A render target rendering to a texture.
    Texture(TextureRenderTarget),
    /// A render target rendering to the screen's buffer.
    Screen,
}

impl RenderTarget {
    /// Return the render target corresponding to the output display.
    pub fn screen_target() -> Self {
        Self::Screen
    }

    /// Create a new render target rendering to a texture
    pub fn new_target(ctx: &GlContext, width: usize, height: usize) -> Self {
        unsafe {
            let target = TextureRenderTarget::new(ctx, width, height);
            Self::Texture(target)
        }
    }

    /// Bind this render target so it will be drawn to.
    unsafe fn bind(&self, gl: &glow::Context) {
        let fb = match self {
            Self::Screen => None,
            Self::Texture(trt) => Some(trt.output_framebuffer),
        };
        gl.bind_framebuffer(glow::FRAMEBUFFER, fb);
    }
}

/// Currently, no input framebuffers or such.
/// We're not actually intending to reproduce Rendy's Graph type here.
#[derive(Debug)]
pub struct RenderPass {
    target: RenderTarget,
    /// This is sort of weird.  Basically to clear the render target
    /// you have to bind it, then clear it.  So if we offer an independent
    /// `clear()` method that will make it possible to forget to bind the
    /// correct render target first, or it will change shared state, or
    /// it will unnecessarily re-bind things.  So instead we ALWAYS clear
    /// the render target if this is not None.
    clear_color: Option<(f32, f32, f32, f32)>,
    viewport: (i32, i32, i32, i32),
    // /// The pipelines to draw in the render pass.
    // pub pipelines: Vec<Box<dyn Pipeline>>,
}

impl RenderPass {
    /// Make a new render pass rendering to a texture.
    pub unsafe fn new(
        ctx: &GlContext,
        width: usize,
        height: usize,
        clear_color: Option<(f32, f32, f32, f32)>,
    ) -> Self {
        let target = RenderTarget::new_target(ctx, width, height);

        Self {
            target,
            //pipelines: vec![],
            viewport: (0, 0, width as i32, height as i32),
            clear_color,
        }
    }

    /// Create a new rnder pass rendering to the screen.
    pub unsafe fn new_screen(
        _ctx: &GlContext,
        width: usize,
        height: usize,
        clear_color: Option<(f32, f32, f32, f32)>,
    ) -> Self {
        Self {
            target: RenderTarget::Screen,
            //pipelines: vec![],
            viewport: (0, 0, width as i32, height as i32),
            clear_color,
        }
    }

    /*
    /// Add a new pipeline to the renderpass
    pub fn add_pipeline(&mut self, pipeline: impl Pipeline + 'static) {
        self.pipelines.push(Box::new(pipeline))
    }
    */

    /// Set the current clear color.  If this is not None,
    /// the render target will be cleared to the returned
    /// RGBA color before any drawing is done.
    pub fn set_clear_color(&mut self, color: Option<(f32, f32, f32, f32)>) {
        self.clear_color = color;
    }

    /// Get the current clear color.
    pub fn clear_color(&self) -> Option<(f32, f32, f32, f32)> {
        self.clear_color
    }

    /// Draw the given pipelines
    pub fn draw<'a, I>(&mut self, ctx: &GlContext, pipelines: I)
    where
        I: IntoIterator<Item = &'a mut (dyn Pipeline<Instance = MeshInstance> + 'static)>,
    {
        // TODO: Audit unsafe
        unsafe {
            self.target.bind(&*ctx.gl);
            let (x, y, w, h) = self.viewport;
            // TODO: Does this need to be set every time, or does it stick to the target binding?
            ctx.gl.viewport(x, y, w, h);

            if let Some((r, g, b, a)) = self.clear_color {
                ctx.gl.clear_color(r, g, b, a);
                ctx.gl
                    .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
            for pipeline in pipelines {
                pipeline.draw(&*ctx.gl);
            }
        }
    }

    /// Get the texture this render pass outputs to, if any.
    pub fn get_texture(&self) -> Option<Texture> {
        match &self.target {
            RenderTarget::Screen => None,
            RenderTarget::Texture(trt) => Some(trt.output_texture.clone()),
        }
    }

    /// Sets the viewport for the render pass.  Negative numbers are valid,
    /// see `glViewport` for the math involved.
    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.viewport = (x, y, w, h);
    }

    /// Returns whether the render target is goin to the actual screen.
    pub fn is_screen(&self) -> bool {
        match self.target {
            RenderTarget::Screen => true,
            _ => false,
        }
    }
}
