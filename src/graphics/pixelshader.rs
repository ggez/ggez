//! The `pixelshader` module allows user-defined fragment shaders to be used
//! with ggez for cool and spooky effects. See the `shader` example for a
//! taste...

use gfx;
use gfx::*;
use gfx::handle::*;
use gfx::pso::*;
use gfx::pso::buffer::*;
use gfx::shade::*;
use gfx::traits::{FactoryExt, Pod};
use gfx_device_gl::{CommandBuffer as GlCommandBuffer, Resources as GlResources};
use std::cell::RefCell;
use std::fmt;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

use Context;
use error::*;
use graphics;

/// A `PixelShader` reprensents a user-defined shader that can be used with a
/// ggez graphics context independent of gfx::Resources type.
///
/// As an end-user you shouldn't ever have to touch this, use `PixelFormat`
/// instead.
pub struct PixelShaderGeneric<R: Resources, C: Structure<ConstFormat>> {
    buffer: Buffer<R, C>,
    pso: PipelineState<R, ConstMeta<C>>,
}

/// A `PixelShader` reprensents a user-defined shader that can be used with a
/// ggez graphics context
pub type PixelShader<C> = PixelShaderGeneric<GlResources, C>;

impl<C> PixelShaderGeneric<GlResources, C>
where
    C: Pod + Structure<ConstFormat> + Clone + Copy,
{
    /// Create a new `PixelShader` given a gfx pipeline object
    pub fn new<P: AsRef<Path>, S: Into<String>>(
        ctx: &mut Context,
        path: P,
        consts: C,
        name: S,
    ) -> GameResult<PixelShaderGeneric<GlResources, C>> {
        let source = {
            let mut buf = Vec::new();
            let mut reader = ctx.filesystem.open(path)?;
            reader.read_to_end(&mut buf)?;
            buf
        };

        let buffer = (&mut ctx.gfx_context.factory).create_constant_buffer(1);
        ctx.gfx_context.encoder.update_buffer(&buffer, &[consts], 0)?;

        let init = ConstInit::<C>(graphics::pipe::new(), name.into(), PhantomData);
        let factory = &mut ctx.gfx_context.factory;
        let set = factory.create_shader_set(
            include_bytes!("shader/basic_150.glslv"),
            &source
        ).unwrap();

        let mut rasterizer = gfx::state::Rasterizer::new_fill().with_cull_back();
        if ctx.gfx_context.multisample_samples > 1 {
            rasterizer.samples = Some(gfx::state::MultiSample);
        }

        let pso = factory.create_pipeline_state
        (
            &set,
            gfx::Primitive::TriangleList,
            rasterizer,
            init
        )?;

        Ok(PixelShader { buffer, pso })
    }

    /// Send data to the GPU for use with the `PixelShader`
    pub fn send(&self, ctx: &mut Context, consts: C) -> GameResult<()> {
        ctx.gfx_context.encoder.update_buffer(
            &self.buffer,
            &[consts],
            0,
        )?;
        Ok(())
    }
}

impl<R, C> fmt::Debug for PixelShaderGeneric<R, C>
where
    R: Resources,
    C: Structure<ConstFormat>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<PixelShader: {:p}>", self)
    }
}

/// A trait to abstracts away the Structure<ConstFormat> type of the constant
/// data for drawing
trait PixelShaderDraw<R: Resources, CB: CommandBuffer<R>> {
    /// Draw with the current PixelShader
    fn draw(&self, &mut Encoder<R, CB>, &Slice<R>, &graphics::pipe::Data<R>);
}

impl<C> PixelShaderDraw<GlResources, GlCommandBuffer> for PixelShaderGeneric<GlResources, C>
where
    C: Structure<ConstFormat>,
{
    fn draw(
        &self,
        encoder: &mut Encoder<GlResources, GlCommandBuffer>,
        slice: &Slice<GlResources>,
        data: &graphics::pipe::Data<GlResources>,
    ) {
        encoder.draw(slice, &self.pso, &ConstData(data, &self.buffer));
    }
}

/// A raw handle to a PixelShader
pub struct PixelShaderHandle<R: Resources, CB: CommandBuffer<R>> {
    handle: *const PixelShaderDraw<R, CB>,
}

impl<R, CB> Clone for PixelShaderHandle<R, CB>
where
    R: Resources,
    CB: CommandBuffer<R>,
{
    fn clone(&self) -> Self {
        PixelShaderHandle { handle: self.handle }
    }
}

impl<R, CB> PixelShaderHandle<R, CB>
where
    R: Resources,
    CB: CommandBuffer<R>,
{
    /// Draw with the pixel shader
    pub fn draw(
        &self,
        encoder: &mut Encoder<R, CB>,
        slice: &Slice<R>,
        data: &graphics::pipe::Data<R>,
    ) {
        unsafe {
            (*self.handle).draw(encoder, slice, data);
        }
    }
}

impl<R, CB> fmt::Debug for PixelShaderHandle<R, CB>
where
    R: Resources,
    CB: CommandBuffer<R>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<PixelShaderHandle: {:p}>", self)
    }
}

/// A lock that automatically clears the pixel shader from the Context when
/// dropped
pub struct PixelShaderLock<R: Resources, CB: CommandBuffer<R>> {
    cell: Rc<RefCell<Option<PixelShaderHandle<R, CB>>>>,
    previous: Option<PixelShaderHandle<R, CB>>,
}

impl<R, CB> fmt::Debug for PixelShaderLock<R, CB>
where
    R: Resources,
    CB: CommandBuffer<R>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "<PixelShaderLock: {:p}>", self)
    }
}

impl<R, CB> Drop for PixelShaderLock<R, CB>
where
    R: Resources,
    CB: CommandBuffer<R>,
{
    fn drop(&mut self) {
        *self.cell.borrow_mut() = self.previous.clone();
    }
}

/// Set the current pixel shader for the Context to render with
pub fn set_pixel_shader<C>(
    ctx: &mut Context,
    ps: &PixelShaderGeneric<GlResources, C>,
) -> PixelShaderLock<GlResources, GlCommandBuffer>
where
    C: 'static + Structure<ConstFormat>,
{
    let cell = ctx.gfx_context.shader.clone();
    let previous = (*cell.borrow_mut()).clone();

    // make a fat pointer
    let handle: &PixelShaderDraw<GlResources, GlCommandBuffer> = ps;
    let handle = PixelShaderHandle::<GlResources, GlCommandBuffer> {
        handle: handle as *const PixelShaderDraw<GlResources, GlCommandBuffer>,
    };

    *cell.borrow_mut() = Some(handle);
    PixelShaderLock { cell, previous }
}

struct ConstMeta<C: Structure<ConstFormat>>(graphics::pipe::Meta, ConstantBuffer<C>);

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
        meta.1.bind_to(out, &self.1, man, access);
    }
}

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
