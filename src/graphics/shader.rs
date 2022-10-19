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

#[derive(Debug)]
enum ShaderSource<'a> {
    None,
    Path(&'a str),
    Code(&'a str),
}

/// Builder pattern for assembling shaders.
#[derive(Debug)]
pub struct ShaderBuilder<'a> {
    fragment_path: ShaderSource<'a>,
    vertex_path: ShaderSource<'a>,
}

impl<'a> ShaderBuilder<'a> {
    /// Create a new builder with no associated shader code.
    pub fn new_wgsl() -> Self {
        ShaderBuilder {
            fragment_path: ShaderSource::None,
            vertex_path: ShaderSource::None,
        }
    }

    /// Use this wgsl shader code for the fragment shader. The vertex shader entry point must be `fs_main`.
    pub fn fragment_code(self, source: &'a str) -> Self {
        ShaderBuilder {
            fragment_path: ShaderSource::Code(source),
            vertex_path: self.vertex_path,
        }
    }
    /// Use this wgsl code resource path for the fragment shader. The vertex shader entry point must be `fs_main`.
    pub fn fragment_path(self, path: &'a str) -> Self {
        ShaderBuilder {
            fragment_path: ShaderSource::Path(path),
            vertex_path: self.vertex_path,
        }
    }

    /// Use this wgsl shader code for the vertex shader. The vertex shader entry point must be `vs_main`.
    pub fn vertex_code(self, source: &'a str) -> Self {
        ShaderBuilder {
            fragment_path: self.vertex_path,
            vertex_path: ShaderSource::Code(source),
        }
    }

    /// Use this wgsl code resource path for the vertex shader. The vertex shader entry point must be `vs_main`.
    pub fn vertex_path(self, path: &'a str) -> Self {
        ShaderBuilder {
            fragment_path: self.vertex_path,
            vertex_path: ShaderSource::Path(path),
        }
    }

    /// Use this wgsl code as both a vertex and fragment shader.
    pub fn combined_code(self, source: &'a str) -> Self {
        ShaderBuilder {
            fragment_path: ShaderSource::Code(source),
            vertex_path: ShaderSource::Code(source),
        }
    }

    /// Use a single wgsl resource as both a vertex and fragment shader.
    pub fn combined_path(self, path: &'a str) -> Self {
        ShaderBuilder {
            fragment_path: ShaderSource::Path(path),
            vertex_path: ShaderSource::Path(path),
        }
    }

    /// Create a Shader from the builder.
    pub fn build(self, gfx: &impl Has<GraphicsContext>) -> GameResult<Shader> {
        let gfx = gfx.retrieve();
        let load = |s: &str| {
            ArcShaderModule::new(gfx.wgpu.device.create_shader_module(
                wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(s.into()),
                },
            ))
        };
        let load_resource = |path: &str| -> GameResult<ArcShaderModule> {
            let mut encoded = Vec::new();
            _ = gfx.fs.open(path)?.read_to_end(&mut encoded)?;
            Ok(load(
                &String::from_utf8(encoded).map_err(|e| GameError::ShaderEncodingError(e))?,
            ))
        };
        let load_any = |source| -> GameResult<ArcShaderModule> {
            Ok(match source {
                ShaderSource::Code(source) => load(source),
                ShaderSource::Path(source) => load_resource(source)?,
                ShaderSource::None => panic!("dead code"),
            })
        };
        Ok(match (self.vertex_path, self.fragment_path) {
            (ShaderSource::None, ShaderSource::None) => Shader {
                vs_module: None,
                fs_module: None,
            },
            (ShaderSource::None, fs) => Shader {
                vs_module: None,
                fs_module: Some(load_any(fs)?),
            },
            (vs, ShaderSource::None) => Shader {
                vs_module: Some(load_any(vs)?),
                fs_module: None,
            },
            (ShaderSource::Code(vs), ShaderSource::Code(fs)) => {
                if vs == fs {
                    let module = load(vs);
                    Shader {
                        vs_module: Some(module.clone()),
                        fs_module: Some(module),
                    }
                } else {
                    Shader {
                        vs_module: Some(load(vs)),
                        fs_module: Some(load(fs)),
                    }
                }
            }
            (ShaderSource::Path(vs), ShaderSource::Path(fs)) => {
                if vs == fs {
                    let module = load_resource(vs)?;
                    Shader {
                        vs_module: Some(module.clone()),
                        fs_module: Some(module),
                    }
                } else {
                    Shader {
                        vs_module: Some(load_resource(vs)?),
                        fs_module: Some(load_resource(fs)?),
                    }
                }
            }
            (vs, fs) => Shader {
                vs_module: Some(load_any(vs)?),
                fs_module: Some(load_any(fs)?),
            },
        })
    }
}

/// A custom fragment shader that can be used to render with shader effects.
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
///         let params = ShaderParams::new(ctx, &dim, &[], &[]);
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

/// Parameters that can be passed to a custom shader, including uniforms, images, and samplers.
///
/// These parameters are bound to group 4. With WGSL, for example,
/// ```rust,ignore
/// ggez::graphics::ShaderParams::new(ctx, &my_uniforms, &[&image1, &image2], &[sampler1])
/// ```
/// Corresponds to...
/// ```ignore
/// @group(4) @binding(0)
/// var<uniform> my_uniforms: MyUniforms;
/// @group(4) @binding(1)
/// var image1: texture_2d<f32>;
/// @group(4) @binding(2)
/// var image2: texture_2d<f32>;
/// @group(4) @binding(3)
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
    /// Creates a new [ShaderParams], initialized with the given uniforms, images, and samplers.
    pub fn new(
        gfx: &mut impl HasMut<GraphicsContext>,
        uniforms: &Uniforms,
        images: &[&Image],
        samplers: &[Sampler],
    ) -> Self {
        let gfx = gfx.retrieve_mut();
        let uniforms = ArcBuffer::new(gfx.wgpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: uniforms.as_std140().as_bytes(),
            },
        ));

        let samplers = samplers
            .iter()
            .map(|&sampler| gfx.sampler_cache.get(&gfx.wgpu.device, sampler))
            .collect::<Vec<_>>();

        let mut builder = BindGroupBuilder::new();
        builder = builder.buffer(
            &uniforms,
            0,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            wgpu::BufferBindingType::Uniform,
            false,
            None,
        );

        for image in images {
            builder = builder.image(&image.view, wgpu::ShaderStages::FRAGMENT);
        }

        for sampler in &samplers {
            builder = builder.sampler(sampler, wgpu::ShaderStages::FRAGMENT);
        }

        let (bind_group, layout) = builder.create_uncached(&gfx.wgpu.device);

        ShaderParams {
            uniforms,
            layout,
            bind_group,
            _marker: PhantomData,
        }
    }

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
