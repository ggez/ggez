use std::io::Read;
use std::marker::PhantomData;

use crate::{
    context::{Has, HasMut},
    GameError, GameResult,
};

use super::{
    context::GraphicsContext,
    gpu::{
        arc::{ArcBindGroup, ArcBindGroupLayout, ArcBuffer, ArcShaderModule},
        bind_group::BindGroupBuilder,
    },
    image::Image,
    sampler::Sampler,
};
use crevice::std140::Std140;
use wgpu::util::DeviceExt;

#[derive(Debug, PartialEq, Eq)]
enum ShaderSource<'a> {
    None,
    Path(&'a str),
    Code(&'a str),
}

/// Builder pattern for assembling shaders.
#[derive(Debug)]
pub struct ShaderBuilder<'a> {
    fs: ShaderSource<'a>,
    vs: ShaderSource<'a>,
}

impl<'a> ShaderBuilder<'a> {
    /// Create a new builder with no associated shader code.
    pub fn new_wgsl() -> Self {
        ShaderBuilder {
            fs: ShaderSource::None,
            vs: ShaderSource::None,
        }
    }

    /// Use this wgsl shader code for the fragment shader.
    pub fn fragment_code(self, source: &'a str) -> Self {
        ShaderBuilder {
            fs: ShaderSource::Code(source),
            vs: self.vs,
        }
    }
    /// Use this wgsl code resource path for the fragment shader.
    pub fn fragment_path(self, path: &'a str) -> Self {
        ShaderBuilder {
            fs: ShaderSource::Path(path),
            vs: self.vs,
        }
    }

    /// Use this wgsl shader code for the vertex shader.
    pub fn vertex_code(self, source: &'a str) -> Self {
        ShaderBuilder {
            fs: self.vs,
            vs: ShaderSource::Code(source),
        }
    }

    /// Use this wgsl code resource path for the vertex shader.
    pub fn vertex_path(self, path: &'a str) -> Self {
        ShaderBuilder {
            fs: self.vs,
            vs: ShaderSource::Path(path),
        }
    }

    /// Use this wgsl code as both a vertex and fragment shader.
    pub fn combined_code(self, source: &'a str) -> Self {
        ShaderBuilder {
            fs: ShaderSource::Code(source),
            vs: ShaderSource::Code(source),
        }
    }

    /// Use a single wgsl resource as both a vertex and fragment shader.
    pub fn combined_path(self, path: &'a str) -> Self {
        ShaderBuilder {
            fs: ShaderSource::Path(path),
            vs: ShaderSource::Path(path),
        }
    }

    /// Create a Shader from the builder.
    pub fn build(self, gfx: &impl Has<GraphicsContext>) -> GameResult<Shader> {
        let gfx = gfx.retrieve();
        let load = |s: &str| {
            Some(ArcShaderModule::new(gfx.wgpu.device.create_shader_module(
                wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(s.into()),
                },
            )))
        };
        let load_resource = |path: &str| -> GameResult<Option<ArcShaderModule>> {
            let mut encoded = Vec::new();
            _ = gfx.fs.open(path)?.read_to_end(&mut encoded)?;
            Ok(load(
                &String::from_utf8(encoded).map_err(GameError::ShaderEncodingError)?,
            ))
        };
        let load_any = |source| -> GameResult<Option<ArcShaderModule>> {
            Ok(match source {
                ShaderSource::Code(source) => load(source),
                ShaderSource::Path(source) => load_resource(source)?,
                ShaderSource::None => None,
            })
        };
        Ok(if self.vs == self.fs {
            let module = load_any(self.vs)?;
            Shader {
                vs_module: module.clone(),
                fs_module: module,
            }
        } else {
            Shader {
                vs_module: load_any(self.vs)?,
                fs_module: load_any(self.fs)?,
            }
        })
    }
}

/// A custom shader that can be used to render with shader effects.
///
/// The shader may have a user specified vertex module, fragment module, both,
/// or neither. The fragment module entry point must be named fs_main. The
/// vertex module entry point must be named vs_main. The vertex module must
/// have an output of type
/// ```wgsl
/// struct VertexOutput {
///     @builtin(position) position: vec4<f32>,
///     @location(0) uv: vec2<f32>,
///     @location(1) color: vec4<f32>,
/// }
/// ```
/// if the fragment module is left unspecified (default).
///
/// Produce a Shader using [ShaderBuilder].
///
/// Adapted from the `shader.rs` example:
/// ```rust
/// # use ggez::*;
/// # use ggez::graphics::*;
/// #[derive(AsStd140)]
/// struct Dim {
///     rate: f32,
/// }
///
/// struct MainState {}
///
/// impl event::EventHandler for MainState {
/// #   fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> { Ok(()) }
///     fn draw(&mut self, ctx: &mut Context) -> GameResult {
///         let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
///         let dim = Dim { rate: 0.5 };
///         // NOTE: This is for simplicity; do not recreate your shader every frame like this!
///         //       For more info look at the full example.
///         let shader = ShaderBuilder::new_wgsl()
///             .fragment_code(include_str!("../../resources/dimmer.wgsl"))
///             .build(&mut ctx.gfx);
///         let params = ShaderParamsBuilder::new(&dim).build(&mut ctx.gfx);
///         params.set_uniforms(ctx, &dim);
///
///         canvas.set_shader(shader);
///         canvas.set_shader_params(params);
///         // draw something...
///         canvas.finish(ctx)
///     }
///
///     /* ... */
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shader {
    pub(crate) vs_module: Option<ArcShaderModule>,
    pub(crate) fs_module: Option<ArcShaderModule>,
}

pub use crevice::std140::AsStd140;

/// A builder for [ShaderParams]
#[derive(Debug)]
pub struct ShaderParamsBuilder<'a, Uniforms: AsStd140> {
    uniforms: &'a Uniforms,
    images: &'a [(&'a Image, Sampler)],
    images_vs_visible: bool,
}

impl<'a, Uniforms: AsStd140> ShaderParamsBuilder<'a, Uniforms> {
    /// Creates a new builder for [ShaderParams].
    ///
    /// # Arguments
    ///
    /// * `uniforms` - Initial uniforms.
    pub fn new(uniforms: &'a Uniforms) -> Self {
        ShaderParamsBuilder {
            uniforms,
            images: &[],
            images_vs_visible: false,
        }
    }

    /// Provides images to the shaders.
    ///
    /// # Arguments
    ///
    /// * `vs_visible` - If the images should also be visible to the vertex shader, rather
    ///    than just the fragment shader.
    pub fn images(self, images: &'a [(&'a Image, Sampler)], vs_visible: bool) -> Self {
        ShaderParamsBuilder {
            uniforms: self.uniforms,
            images,
            images_vs_visible: vs_visible,
        }
    }

    /// Produce a [ShaderParams] from the builder.
    pub fn build(self, gfx: &mut impl HasMut<GraphicsContext>) -> ShaderParams<Uniforms> {
        let gfx = gfx.retrieve_mut();
        let uniforms = ArcBuffer::new(gfx.wgpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: self.uniforms.as_std140().as_bytes(),
            },
        ));

        let mut builder = BindGroupBuilder::new();
        builder = builder.buffer(
            &uniforms,
            0,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            wgpu::BufferBindingType::Uniform,
            false,
            None,
        );

        let vis = if self.images_vs_visible {
            wgpu::ShaderStages::VERTEX_FRAGMENT
        } else {
            wgpu::ShaderStages::FRAGMENT
        };
        for (image, _) in self.images {
            builder = builder.image(&image.view, vis);
        }

        let samplers = self
            .images
            .iter()
            .map(|&(_, sampler)| gfx.sampler_cache.get(&gfx.wgpu.device, sampler))
            .collect::<Vec<_>>();
        for sampler in &samplers {
            builder = builder.sampler(sampler, vis);
        }

        let (bind_group, layout) = builder.create_uncached(&gfx.wgpu.device);

        ShaderParams {
            uniforms,
            layout,
            bind_group,
            _marker: PhantomData,
        }
    }
}

/// Parameters that can be passed to a custom shader, including uniforms, images, and samplers.
///
/// Create with [ShaderParamsBuilder].
///
/// These parameters are bound to group 3. With WGSL, for example,
/// ```rust,ignore
/// ggez::graphics::ShaderParams::new(ctx, &my_uniforms, &[&image1, &image2], &[sampler1])
/// ```
/// Corresponds to...
/// ```ignore
/// @group(3) @binding(0)
/// var<uniform> my_uniforms: MyUniforms;
/// @group(3) @binding(1)
/// var image1: texture_2d<f32>;
/// @group(3) @binding(2)
/// var image2: texture_2d<f32>;
/// @group(3) @binding(3)
/// var sampler1: sampler;
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ShaderParams<Uniforms: AsStd140> {
    pub(crate) uniforms: ArcBuffer,
    pub(crate) layout: ArcBindGroupLayout,
    pub(crate) bind_group: ArcBindGroup,
    _marker: PhantomData<Uniforms>,
}

impl<Uniforms: AsStd140> ShaderParams<Uniforms> {
    /// Updates the uniform data.
    pub fn set_uniforms(&self, gfx: &impl Has<GraphicsContext>, uniforms: &Uniforms) {
        let gfx = gfx.retrieve();
        gfx.wgpu
            .queue
            .write_buffer(&self.uniforms, 0, uniforms.as_std140().as_bytes());
    }
}

impl<Uniforms: AsStd140> Clone for ShaderParams<Uniforms> {
    fn clone(&self) -> Self {
        Self {
            uniforms: self.uniforms.clone(),
            layout: self.layout.clone(),
            bind_group: self.bind_group.clone(),
            _marker: PhantomData,
        }
    }
}

pub use wgpu::{BlendComponent, BlendFactor, BlendOperation};

/// Describes the blend mode used when drawing images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlendMode {
    /// The blend mode for the color channels.
    pub color: BlendComponent,
    /// The blend mode for the alpha channel.
    pub alpha: BlendComponent,
}

impl BlendMode {
    /// When combining two fragments, add their values together, saturating
    /// at 1.0
    pub const ADD: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    };

    /// When combining two fragments, subtract the source value from the
    /// destination value
    pub const SUBTRACT: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::ReverseSubtract,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::Zero,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    };

    /// When combining two fragments, add the value of the source times its
    /// alpha channel with the value of the destination multiplied by the inverse
    /// of the source alpha channel. Has the usual transparency effect: mixes the
    /// two colors using a fraction of each one specified by the alpha of the source.
    pub const ALPHA: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    };

    /// When combining two fragments, subtract the destination color from a constant
    /// color using the source color as weight. Has an invert effect with the constant
    /// color as base and source color controlling displacement from the base color.
    /// A white source color and a white value results in plain invert. The output
    /// alpha is same as destination alpha.
    pub const INVERT: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::Constant,
            dst_factor: BlendFactor::Src,
            operation: BlendOperation::Subtract,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::Zero,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    };

    /// When combining two fragments, multiply their values together (including alpha)
    pub const MULTIPLY: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::Dst,
            dst_factor: BlendFactor::Zero,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::DstAlpha,
            dst_factor: BlendFactor::Zero,
            operation: BlendOperation::Add,
        },
    };

    /// When combining two fragments, choose the source value (including source alpha)
    pub const REPLACE: Self = BlendMode {
        color: wgpu::BlendState::REPLACE.color,
        alpha: wgpu::BlendState::REPLACE.alpha,
    };

    /// When combining two fragments, choose the lighter value
    pub const LIGHTEN: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Max,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    };

    /// When combining two fragments, choose the darker value
    pub const DARKEN: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Min,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    };

    /// When using premultiplied alpha, use this.
    ///
    /// You usually want to use this blend mode for drawing canvases
    /// containing semi-transparent imagery.
    /// For an explanation on this see: <https://github.com/ggez/ggez/issues/694#issuecomment-853724926>
    pub const PREMULTIPLIED: Self = BlendMode {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusDstAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    };
}
