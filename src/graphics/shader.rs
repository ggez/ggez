use std::marker::PhantomData;

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
///         let mut canvas = graphics::Canvas::from_frame(&ctx.gfx, Color::BLACK);
///         let dim = Dim { rate: 0.5 };
///         // NOTE: This is for simplicity; do not recreate your shader every frame like this!
///         //       For more info look at the full example.
///         let shader = Shader::from_wgsl(
///             &ctx.gfx,
///             include_str!("../resources/dimmer.wgsl"),
///             "main"
///         );
///         let params = ShaderParams::new(&mut ctx.gfx, &dim, &[], &[]);
///         params.set_uniforms(&ctx.gfx, &dim);
///
///         canvas.set_shader(shader);
///         canvas.set_shader_params(params);
///         // draw something...
///         canvas.finish(&mut ctx.gfx)
///     }
///
///     /* ... */
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shader {
    pub(crate) fragment: ArcShaderModule,
    pub(crate) fs_entry: String,
}

impl Shader {
    /// Creates a shader from a WGSL string.
    pub fn from_wgsl(gfx: &GraphicsContext, wgsl: &str, fs_entry: &str) -> Self {
        let module = ArcShaderModule::new(gfx.wgpu.device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl.into()),
            },
        ));

        Shader {
            fragment: module,
            fs_entry: fs_entry.into(),
        }
    }
}

pub use crevice::std140::AsStd140;

/// Parameters that can be passed to a custom shader, including uniforms, images, and samplers.
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
        gfx: &mut GraphicsContext,
        uniforms: &Uniforms,
        images: &[&Image],
        samplers: &[Sampler],
    ) -> Self {
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

        let (bind_group, layout) = builder.create(&gfx.wgpu.device, &mut gfx.bind_group_cache);

        ShaderParams {
            uniforms,
            layout,
            bind_group,
            _marker: PhantomData,
        }
    }

    /// Updates the uniform data.
    pub fn set_uniforms(&self, gfx: &GraphicsContext, uniforms: &Uniforms) {
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
