//! The `shader` module allows user-defined shaders to be used
//! with ggez for cool and spooky effects. See the
//! [`shader`](https://github.com/ggez/ggez/blob/devel/examples/shader.rs)
//! and [`shadows`](https://github.com/ggez/ggez/blob/devel/examples/shadows.rs)
//! examples for a taste.
#![allow(unsafe_code)]
use gfx::format;
use gfx::handle::*;
use gfx::preset::blend;
use gfx::pso::buffer::*;
use gfx::pso::*;
use gfx::shade::*;
use gfx::state::*;
use gfx::traits::{FactoryExt, Pod};
use gfx::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

use crate::context::DebugId;
use crate::error::*;
use crate::graphics;
use crate::Context;

/// A type for empty shader data for shaders that do not require any additional
/// data to be sent to the GPU
#[derive(Clone, Copy, Debug)]
pub struct EmptyConst;

impl<F> Structure<F> for EmptyConst {
    fn query(_name: &str) -> Option<Element<F>> {
        None
    }
}

unsafe impl Pod for EmptyConst {}

/// An enum for specifying default and custom blend modes
///
/// If you want to know what these actually do take a look at the implementation of `From<BlendMode> for Blend`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// When combining two fragments, add their values together, saturating
    /// at 1.0
    Add,
    /// When combining two fragments, subtract the source value from the
    /// destination value
    Subtract,
    /// When combining two fragments, add the value of the source times its
    /// alpha channel with the value of the destination multiplied by the inverse
    /// of the source alpha channel. Has the usual transparency effect: mixes the
    /// two colors using a fraction of each one specified by the alpha of the source.
    Alpha,
    /// When combining two fragments, subtract the destination color from a constant
    /// color using the source color as weight. Has an invert effect with the constant
    /// color as base and source color controlling displacement from the base color.
    /// A white source color and a white value results in plain invert. The output
    /// alpha is same as destination alpha.
    Invert,
    /// When combining two fragments, multiply their values together (including alpha)
    Multiply,
    /// When combining two fragments, choose the source value (including source alpha)
    Replace,
    /// When combining two fragments, choose the lighter value
    Lighten,
    /// When combining two fragments, choose the darker value
    Darken,
    /// When using premultiplied alpha, use this.
    ///
    /// You usually want to use this blend mode for drawing canvases
    /// containing semi-transparent imagery.
    /// For an explanation on this see: <https://github.com/ggez/ggez/issues/694#issuecomment-853724926>
    Premultiplied,
}

impl From<BlendMode> for Blend {
    fn from(bm: BlendMode) -> Self {
        match bm {
            BlendMode::Add => Blend {
                color: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::ZeroPlus(BlendValue::SourceAlpha),
                    destination: Factor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::OneMinus(BlendValue::DestAlpha),
                    destination: Factor::One,
                },
            },
            BlendMode::Subtract => Blend {
                color: BlendChannel {
                    equation: Equation::RevSub,
                    source: Factor::ZeroPlus(BlendValue::SourceAlpha),
                    destination: Factor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::Zero,
                    destination: Factor::One,
                },
            },
            BlendMode::Alpha => Blend {
                color: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::ZeroPlus(BlendValue::SourceAlpha),
                    destination: Factor::OneMinus(BlendValue::SourceAlpha),
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::OneMinus(BlendValue::DestAlpha),
                    destination: Factor::One,
                },
            },
            BlendMode::Invert => blend::INVERT,
            BlendMode::Multiply => blend::MULTIPLY,
            BlendMode::Replace => blend::REPLACE,
            BlendMode::Lighten => Blend {
                color: BlendChannel {
                    equation: Equation::Max,
                    source: Factor::ZeroPlus(BlendValue::SourceAlpha),
                    destination: Factor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::OneMinus(BlendValue::DestAlpha),
                    destination: Factor::One,
                },
            },
            BlendMode::Darken => Blend {
                color: BlendChannel {
                    equation: Equation::Min,
                    source: Factor::ZeroPlus(BlendValue::SourceAlpha),
                    destination: Factor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::OneMinus(BlendValue::DestAlpha),
                    destination: Factor::One,
                },
            },
            BlendMode::Premultiplied => Blend {
                color: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::One,
                    destination: Factor::OneMinus(BlendValue::SourceAlpha),
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: Factor::OneMinus(BlendValue::DestAlpha),
                    destination: Factor::One,
                },
            },
        }
    }
}

/// A struct to easily store a set of pipeline state objects that are
/// associated with a specific shader program.
///
/// In gfx, because Vulkan and DX are more strict
/// about how blend modes work than GL is, blend modes are
/// baked in as a piece of state for a PSO and you can't change it
/// dynamically. After chatting with @kvark on IRC and looking
/// how he does it in three-rs, the best way to change blend
/// modes is to just make multiple PSOs with respective blend modes baked in.
/// The `PsoSet` struct is basically just a hash map for easily
/// storing each shader set's PSOs and then retrieving them based
/// on a [`BlendMode`](enum.BlendMode.html).
struct PsoSet<Spec, C>
where
    Spec: graphics::BackendSpec,
    C: Structure<ConstFormat>,
{
    psos: HashMap<BlendMode, PipelineState<Spec::Resources, ConstMeta<C>>>,
}

impl<Spec, C> PsoSet<Spec, C>
where
    Spec: graphics::BackendSpec,
    C: Structure<ConstFormat>,
{
    pub fn new(cap: usize) -> Self {
        Self {
            psos: HashMap::with_capacity(cap),
        }
    }

    pub fn insert_mode(
        &mut self,
        mode: BlendMode,
        pso: PipelineState<Spec::Resources, ConstMeta<C>>,
    ) {
        let _ = self.psos.insert(mode, pso);
    }

    pub fn mode(
        &self,
        mode: BlendMode,
    ) -> GameResult<&PipelineState<Spec::Resources, ConstMeta<C>>> {
        match self.psos.get(&mode) {
            Some(pso) => Ok(pso),
            None => Err(GameError::RenderError(
                "Could not find a pipeline for the specified shader and BlendMode".into(),
            )),
        }
    }
}

/// An ID used by the ggez graphics context to uniquely identify a shader
pub type ShaderId = usize;

/// A `ShaderGeneric` reprensents a handle user-defined shader that can be used
/// with a ggez graphics context that is generic over `gfx::Resources`
///
/// As an end-user you shouldn't ever have to touch this and should use
/// [`Shader`](type.Shader.html) instead.
#[derive(Clone)]
pub struct ShaderGeneric<Spec: graphics::BackendSpec, C: Structure<ConstFormat>> {
    pub(crate) id: ShaderId,
    pub(crate) buffer: Buffer<Spec::Resources, C>,
    debug_id: DebugId,
}

/// A `Shader` represents a handle to a user-defined shader that can be used
/// with a ggez graphics context
pub type Shader<C> = ShaderGeneric<graphics::GlBackendSpec, C>;

type ShaderHandlePtr<Spec> = Box<dyn ShaderHandle<Spec>>;

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_shader<C, S, Spec>(
    vertex_source: &[u8],
    pixel_source: &[u8],
    consts: C,
    name: S,
    encoder: &mut Encoder<Spec::Resources, Spec::CommandBuffer>,
    factory: &mut Spec::Factory,
    multisample_samples: u8,
    blend_modes: Option<&[BlendMode]>,
    color_format: format::Format,
    debug_id: DebugId,
) -> GameResult<(ShaderGeneric<Spec, C>, ShaderHandlePtr<Spec>)>
where
    C: 'static + Pod + Structure<ConstFormat> + Clone + Copy,
    S: Into<String>,
    Spec: graphics::BackendSpec + 'static,
{
    let buffer = factory.create_constant_buffer(1);

    encoder.update_buffer(&buffer, &[consts], 0)?;

    let default_mode = vec![BlendMode::Alpha];
    let blend_modes = blend_modes.unwrap_or(&default_mode[..]);

    let mut psos = PsoSet::new(blend_modes.len());
    let name: String = name.into();
    for mode in blend_modes {
        let init = ConstInit::<C>(
            graphics::pipe::Init {
                out: (
                    "Target0",
                    color_format,
                    ColorMask::all(),
                    Some((*mode).into()),
                ),
                ..graphics::pipe::new()
            },
            name.clone(),
            PhantomData,
        );
        let set = factory.create_shader_set(vertex_source, pixel_source)?;
        let sample = if multisample_samples > 1 {
            Some(MultiSample)
        } else {
            None
        };
        let rasterizer = Rasterizer {
            front_face: FrontFace::CounterClockwise,
            cull_face: CullFace::Nothing,
            method: RasterMethod::Fill,
            offset: None,
            samples: sample,
        };

        let pso = factory.create_pipeline_state(&set, Primitive::TriangleList, rasterizer, init)?;
        psos.insert_mode(*mode, pso);
    }

    let program = ShaderProgram {
        buffer: buffer.clone(),
        psos,
        active_blend_mode: blend_modes[0],
    };
    let draw: ShaderHandlePtr<Spec> = Box::new(program);

    let id = 0;
    let shader = ShaderGeneric {
        id,
        buffer,
        debug_id,
    };

    Ok((shader, draw))
}

impl<Spec, C> ShaderGeneric<Spec, C>
where
    Spec: graphics::BackendSpec,
    C: 'static + Pod + Structure<ConstFormat> + Clone + Copy,
{
    #[allow(clippy::new_ret_no_self)]
    /// Create a new `Shader` given source files, constants and a name.
    ///
    /// In order to use a specific blend mode when this shader is being
    /// used, you must include that blend mode as part of the
    /// `blend_modes` parameter at creation. If `None` is given, only the
    /// default [`Alpha`](enum.BlendMode.html#variant.Alpha) blend mode is used.
    pub fn new<P: AsRef<Path>, S: Into<String>>(
        ctx: &mut Context,
        vertex_path: P,
        pixel_path: P,
        consts: C,
        name: S,
        blend_modes: Option<&[BlendMode]>,
    ) -> GameResult<Shader<C>> {
        let vertex_source = {
            let mut buf = Vec::new();
            let mut reader = ctx.filesystem.open(vertex_path)?;
            let _ = reader.read_to_end(&mut buf)?;
            buf
        };
        let pixel_source = {
            let mut buf = Vec::new();
            let mut reader = ctx.filesystem.open(pixel_path)?;
            let _ = reader.read_to_end(&mut buf)?;
            buf
        };
        Shader::from_u8(
            ctx,
            &vertex_source,
            &pixel_source,
            consts,
            name,
            blend_modes,
        )
    }

    /// Create a new `Shader` directly from GLSL source code.
    ///
    /// In order to use a specific blend mode when this shader is being
    /// used, you must include that blend mode as part of the
    /// `blend_modes` parameter at creation. If `None` is given, only the
    /// default [`Alpha`](enum.BlendMode.html#variant.Alpha) blend mode is used.
    pub fn from_u8<S: Into<String>>(
        ctx: &mut Context,
        vertex_source: &[u8],
        pixel_source: &[u8],
        consts: C,
        name: S,
        blend_modes: Option<&[BlendMode]>,
    ) -> GameResult<Shader<C>> {
        let debug_id = DebugId::get(ctx);
        let color_format = ctx.gfx_context.color_format();
        let (mut shader, draw) = create_shader(
            vertex_source,
            pixel_source,
            consts,
            name,
            &mut ctx.gfx_context.encoder,
            &mut *ctx.gfx_context.factory,
            ctx.gfx_context.multisample_samples,
            blend_modes,
            color_format,
            debug_id,
        )?;
        shader.id = ctx.gfx_context.shaders.len();
        ctx.gfx_context.shaders.push(draw);

        Ok(shader)
    }
}

impl<C> Shader<C>
where
    C: 'static + Pod + Structure<ConstFormat> + Clone + Copy,
{
    /// Send data to the GPU for use with the `Shader`
    pub fn send(&self, ctx: &mut Context, consts: C) -> GameResult {
        ctx.gfx_context
            .encoder
            .update_buffer(&self.buffer, &[consts], 0)?;
        Ok(())
    }

    /// Gets the shader ID for the `Shader` which is used by the
    /// graphics context for identifying shaders in its cache
    pub fn shader_id(&self) -> ShaderId {
        self.id
    }
}

impl<Spec, C> fmt::Debug for ShaderGeneric<Spec, C>
where
    Spec: graphics::BackendSpec,
    C: Structure<ConstFormat>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<Shader[{}]: {:p}>", self.id, self)
    }
}

struct ShaderProgram<Spec: graphics::BackendSpec, C: Structure<ConstFormat>> {
    buffer: Buffer<Spec::Resources, C>,
    psos: PsoSet<Spec, C>,
    active_blend_mode: BlendMode,
}

impl<Spec, C> fmt::Debug for ShaderProgram<Spec, C>
where
    Spec: graphics::BackendSpec,
    C: Structure<ConstFormat>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<ShaderProgram: {:p}>", self)
    }
}

/// A trait that is used to create trait objects to abstract away the
/// `gfx::Structure<ConstFormat>` type of the constant data for drawing
pub trait ShaderHandle<Spec: graphics::BackendSpec>: fmt::Debug {
    /// Draw with the current Shader
    fn draw(
        &self,
        encoder: &mut Encoder<Spec::Resources, Spec::CommandBuffer>,
        slice: &Slice<Spec::Resources>,
        data: &graphics::pipe::Data<Spec::Resources>,
    ) -> GameResult;

    /// Sets the shader program's blend mode
    fn set_blend_mode(&mut self, mode: BlendMode) -> GameResult;

    /// Gets the shader program's current blend mode
    fn blend_mode(&self) -> BlendMode;
}

impl<Spec, C> ShaderHandle<Spec> for ShaderProgram<Spec, C>
where
    Spec: graphics::BackendSpec,
    C: Structure<ConstFormat>,
{
    fn draw(
        &self,
        encoder: &mut Encoder<Spec::Resources, Spec::CommandBuffer>,
        slice: &Slice<Spec::Resources>,
        data: &graphics::pipe::Data<Spec::Resources>,
    ) -> GameResult {
        let pso = self.psos.mode(self.active_blend_mode)?;
        encoder.draw(slice, pso, &ConstData(data, &self.buffer));
        Ok(())
    }

    fn set_blend_mode(&mut self, mode: BlendMode) -> GameResult {
        let _ = self.psos.mode(mode)?;
        self.active_blend_mode = mode;
        Ok(())
    }

    fn blend_mode(&self) -> BlendMode {
        self.active_blend_mode
    }
}

/// A lock for RAII shader regions. The shader automatically gets cleared once
/// the lock goes out of scope, restoring the previous shader (if any).
///
/// Essentially, binding a [`Shader`](type.Shader.html) will return one of these,
/// and the shader will remain active as long as this object exists.  When this is
/// dropped, the previous shader is restored.
#[derive(Debug, Clone)]
pub struct ShaderLock {
    cell: Rc<RefCell<Option<ShaderId>>>,
    previous_shader: Option<ShaderId>,
}

impl Drop for ShaderLock {
    fn drop(&mut self) {
        *self.cell.borrow_mut() = self.previous_shader;
    }
}

/// Use a shader until the returned lock goes out of scope
pub fn use_shader<C>(ctx: &mut Context, ps: &Shader<C>) -> ShaderLock
where
    C: Structure<ConstFormat>,
{
    ps.debug_id.assert(ctx);
    let cell = Rc::clone(&ctx.gfx_context.current_shader);
    let previous_shader = *cell.borrow();
    set_shader(ctx, ps);
    ShaderLock {
        cell,
        previous_shader,
    }
}

/// Set the current shader for the `Context` to render with
pub fn set_shader<C>(ctx: &mut Context, ps: &Shader<C>)
where
    C: Structure<ConstFormat>,
{
    ps.debug_id.assert(ctx);
    *ctx.gfx_context.current_shader.borrow_mut() = Some(ps.id);
}

/// Clears the the current shader for the `Context`, restoring the default shader.
///
/// However, calling this and then dropping a [`ShaderLock`](struct.ShaderLock.html)
/// will still set the shader to whatever was set when the `ShaderLock` was created.
pub fn clear_shader(ctx: &mut Context) {
    *ctx.gfx_context.current_shader.borrow_mut() = None;
}

#[derive(Debug)]
struct ConstMeta<C: Structure<ConstFormat>>(graphics::pipe::Meta, ConstantBuffer<C>);

#[derive(Debug)]
struct ConstData<'a, R: Resources, C: 'a>(&'a graphics::pipe::Data<R>, &'a Buffer<R, C>);

impl<'a, R, C> PipelineData<R> for ConstData<'a, R, C>
where
    R: Resources,
    C: Structure<ConstFormat>,
{
    type Meta = ConstMeta<C>;

    fn bake_to(
        &self,
        out: &mut RawDataSet<R>,
        meta: &Self::Meta,
        man: &mut Manager<R>,
        access: &mut AccessInfo<R>,
    ) {
        self.0.bake_to(out, &meta.0, man, access);
        meta.1.bind_to(out, self.1, man, access);
    }
}

#[derive(Debug)]
struct ConstInit<'a, C>(graphics::pipe::Init<'a>, String, PhantomData<C>);

impl<'a, C> PipelineInit for ConstInit<'a, C>
where
    C: Structure<ConstFormat>,
{
    type Meta = ConstMeta<C>;

    fn link_to<'s>(
        &self,
        desc: &mut Descriptor,
        info: &'s ProgramInfo,
    ) -> Result<Self::Meta, InitError<&'s str>> {
        let mut meta1 = ConstantBuffer::<C>::new();

        let mut index = None;
        for (i, cb) in info.constant_buffers.iter().enumerate() {
            match meta1.link_constant_buffer(cb, &self.1.as_str()) {
                Some(Ok(d)) => {
                    assert!(meta1.is_active());
                    desc.constant_buffers[cb.slot as usize] = Some(d);
                    index = Some(i);
                    break;
                }
                Some(Err(e)) => return Err(InitError::ConstantBuffer(&cb.name, Some(e))),
                None => (),
            }
        }

        if let Some(index) = index {
            // create a local clone of the program info so that we can remove
            // the var we found from the `constant_buffer`
            let mut program_info = info.clone();
            let _ = program_info.constant_buffers.remove(index);

            let meta0 = match self.0.link_to(desc, &program_info) {
                Ok(m) => m,
                Err(e) => {
                    // unfortunately... the error lifetime is bound to the
                    // lifetime of our cloned program info which is bad since it
                    // will go out of scope at the end of the function, so lets
                    // convert the error to one that is bound to the lifetime of
                    // the program info that was passed in!
                    macro_rules! fixlifetimes {
                        ($e:ident {
                            $( $ty:path => $a:ident, )*
                        }) => {{
                            match $e {
                                $( $ty(name, _) => {
                                    let var = info.$a.iter().find(|v| v.name == name).unwrap();
                                    // We can do better with the error data...
                                    return Err($ty(&var.name, None));
                                } )*
                            }
                        }}
                    }
                    fixlifetimes!(e {
                        InitError::VertexImport => vertex_attributes,
                        InitError::ConstantBuffer => constant_buffers,
                        InitError::GlobalConstant => globals,
                        InitError::ResourceView => textures,
                        InitError::UnorderedView => unordereds,
                        InitError::Sampler => samplers,
                        InitError::PixelExport => outputs,
                    })
                }
            };

            Ok(ConstMeta(meta0, meta1))
        } else {
            Ok(ConstMeta(self.0.link_to(desc, info)?, meta1))
        }
    }
}
