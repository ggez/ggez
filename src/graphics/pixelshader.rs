//! The `pixelshader` module allows user-defined fragment shaders to be used
//! with ggez for cool and spooky effects. See the `shader` example for a
//! taste...

use gfx::*;
use gfx::handle::*;
use gfx::pso::*;
use gfx::pso::buffer::*;
use gfx::shade::*;
use gfx::state::*;
use gfx::traits::{FactoryExt, Pod};
use std::cell::RefCell;
use std::fmt;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

use Context;
use error::*;
use graphics;

/// A type for empty shader data for shaders that do not require any aditional
/// data to be sent to the GPU
#[derive(Clone, Copy, Debug)]
pub struct EmptyConst;

impl<F> Structure<F> for EmptyConst {
    fn query(_name: &str) -> Option<Element<F>> {
        None
    }
}

unsafe impl Pod for EmptyConst {}

/// An ID used by the `GraphicsContext` to uniquely identify a pixel shader
pub type PixelShaderId = usize;

/// A `PixelShader` reprensents a handle user-defined shader that can be used
/// with a ggez graphics context that is generic over `gfx::Resources`
///
/// As an end-user you shouldn't ever have to touch this and should use
/// `PixelShader` instead.
#[derive(Clone)]
pub struct PixelShaderGeneric<Spec: graphics::BackendSpec, C: Structure<ConstFormat>> {
    id: PixelShaderId,
    buffer: Rc<Buffer<Spec::Resources, C>>,
}

/// A `PixelShader` reprensents a handle user-defined shader that can be used
/// with a ggez graphics context
pub type PixelShader<C> = PixelShaderGeneric<graphics::GlBackendSpec, C>;

pub(crate) fn create_shader<C, S, Spec>
    (source: &[u8],
     consts: C,
     name: S,
     encoder: &mut Encoder<Spec::Resources, Spec::CommandBuffer>,
     factory: &mut Spec::Factory,
     multisample_samples: u8)
     -> GameResult<(PixelShaderGeneric<Spec, C>, Box<PixelShaderDraw<Spec>>)>
    where C: 'static + Pod + Structure<ConstFormat> + Clone + Copy,
          S: Into<String>,
          Spec: graphics::BackendSpec + 'static
{
    let buffer = factory.create_constant_buffer(1);
    let buffer = Rc::new(buffer);

    encoder.update_buffer(&buffer, &[consts], 0)?;

    let pso = {
        let init = ConstInit::<C>(graphics::pipe::new(), name.into(), PhantomData);
        let set = factory
            .create_shader_set(include_bytes!("shader/basic_150.glslv"), &source)?;
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

        factory
            .create_pipeline_state(&set, Primitive::TriangleList, rasterizer, init)?
    };

    let program = PixelShaderProgram {
        buffer: buffer.clone(),
        pso,
    };
    let draw: Box<PixelShaderDraw<Spec>> = Box::new(program);

    let id = 0;
    let shader = PixelShaderGeneric { id, buffer };

    Ok((shader, draw))
}

impl<C> PixelShader<C>
    where C: 'static + Pod + Structure<ConstFormat> + Clone + Copy
{
    /// Create a new `PixelShader` given a gfx pipeline object
    pub fn new<P: AsRef<Path>, S: Into<String>>(ctx: &mut Context,
                                                path: P,
                                                consts: C,
                                                name: S)
                                                -> GameResult<PixelShader<C>> {
        let source = {
            let mut buf = Vec::new();
            let mut reader = ctx.filesystem.open(path)?;
            reader.read_to_end(&mut buf)?;
            buf
        };
        PixelShader::from_u8(ctx, &source, consts, name)
    }

    /// Create a new `PixelShader` directly from source given a gfx pipeline
    /// object
    pub fn from_u8<S: Into<String>>(ctx: &mut Context,
                                    source: &[u8],
                                    consts: C,
                                    name: S)
                                    -> GameResult<PixelShader<C>> {
        let (mut shader, draw) = create_shader(&source,
                                               consts,
                                               name,
                                               &mut ctx.gfx_context.encoder,
                                               &mut *ctx.gfx_context.factory,
                                               ctx.gfx_context.multisample_samples)?;
        shader.id = ctx.gfx_context.shaders.len();
        ctx.gfx_context.shaders.push(draw);

        Ok(shader)
    }

    /// Send data to the GPU for use with the `PixelShader`
    pub fn send(&self, ctx: &mut Context, consts: C) -> GameResult<()> {
        ctx.gfx_context
            .encoder
            .update_buffer(&self.buffer, &[consts], 0)?;
        Ok(())
    }

    /// Gets the shader ID for the `PixelShader` which is used by the
    /// `GraphicsContext` for identifying shaders in its cache
    pub fn shader_id(&self) -> PixelShaderId {
        self.id
    }
}

impl<Spec, C> fmt::Debug for PixelShaderGeneric<Spec, C>
    where Spec: graphics::BackendSpec,
          C: Structure<ConstFormat>
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<PixelShader[{}]: {:p}>", self.id, self)
    }
}

struct PixelShaderProgram<Spec: graphics::BackendSpec, C: Structure<ConstFormat>> {
    buffer: Rc<Buffer<Spec::Resources, C>>,
    pso: PipelineState<Spec::Resources, ConstMeta<C>>,
}

impl<Spec, C> fmt::Debug for PixelShaderProgram<Spec, C>
    where Spec: graphics::BackendSpec,
          C: Structure<ConstFormat>
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<PixelShaderProgram: {:p}>", self)
    }
}

/// A trait that is used to create trait objects to abstract away the
/// Structure<ConstFormat> type of the constant data for drawing
pub trait PixelShaderDraw<Spec: graphics::BackendSpec>: fmt::Debug {
    /// Draw with the current PixelShader
    fn draw(&self,
            &mut Encoder<Spec::Resources, Spec::CommandBuffer>,
            &Slice<Spec::Resources>,
            &graphics::pipe::Data<Spec::Resources>);
}

impl<Spec, C> PixelShaderDraw<Spec> for PixelShaderProgram<Spec, C>
    where Spec: graphics::BackendSpec,
          C: Structure<ConstFormat>
{
    fn draw(&self,
            encoder: &mut Encoder<Spec::Resources, Spec::CommandBuffer>,
            slice: &Slice<Spec::Resources>,
            data: &graphics::pipe::Data<Spec::Resources>) {
        encoder.draw(slice, &self.pso, &ConstData(data, &self.buffer));
    }
}

/// A lock for RAII shader regions. The shader automatically gets cleared once
/// the lock goes out of scope
#[derive(Debug)]
pub struct PixelShaderLock {
    cell: Rc<RefCell<Option<PixelShaderId>>>,
    previous_shader: Option<PixelShaderId>,
}

impl Drop for PixelShaderLock {
    fn drop(&mut self) {
        *self.cell.borrow_mut() = self.previous_shader;
    }
}

/// Use a shader until the returned lock goes out of scope
pub fn use_shader<C>(ctx: &mut Context, ps: &PixelShader<C>) -> PixelShaderLock
    where C: Structure<ConstFormat>
{
    let cell = ctx.gfx_context.current_shader.clone();
    let previous_shader = (*cell.borrow()).clone();
    set_shader(ctx, ps);
    PixelShaderLock {
        cell,
        previous_shader,
    }
}

/// Set the current pixel shader for the Context to render with
pub fn set_shader<C>(ctx: &mut Context, ps: &PixelShader<C>)
    where C: Structure<ConstFormat>
{
    *ctx.gfx_context.current_shader.borrow_mut() = Some(ps.id);
}

/// Clears the the current pixel shader for the Context making use the default
pub fn clear_shader(ctx: &mut Context) {
    *ctx.gfx_context.current_shader.borrow_mut() = None;
}

#[derive(Debug)]
struct ConstMeta<C: Structure<ConstFormat>>(graphics::pipe::Meta, ConstantBuffer<C>);

#[derive(Debug)]
struct ConstData<'a, R: Resources, C: 'a>(&'a graphics::pipe::Data<R>, &'a Buffer<R, C>);

impl<'a, R, C> PipelineData<R> for ConstData<'a, R, C>
    where R: Resources,
          C: Structure<ConstFormat>
{
    type Meta = ConstMeta<C>;

    fn bake_to(&self,
               out: &mut RawDataSet<R>,
               meta: &Self::Meta,
               man: &mut Manager<R>,
               access: &mut AccessInfo<R>) {
        self.0.bake_to(out, &meta.0, man, access);
        meta.1.bind_to(out, &self.1, man, access);
    }
}

#[derive(Debug)]
struct ConstInit<'a, C>(graphics::pipe::Init<'a>, String, PhantomData<C>);

impl<'a, C> PipelineInit for ConstInit<'a, C>
    where C: Structure<ConstFormat>
{
    type Meta = ConstMeta<C>;

    fn link_to<'s>(&self,
                   desc: &mut Descriptor,
                   info: &'s ProgramInfo)
                   -> Result<Self::Meta, InitError<&'s str>> {
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
            program_info.constant_buffers.remove(index);

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
                    });
                }
            };

            Ok(ConstMeta(meta0, meta1))
        } else {
            Ok(ConstMeta(self.0.link_to(desc, &info)?, meta1))
        }
    }
}
