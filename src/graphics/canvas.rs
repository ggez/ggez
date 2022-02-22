use super::{image::Image, Color};
use crate::Context;

/// A canvas represents a render pass and is how you render primitives onto images.
#[derive(Debug)]
pub struct Canvas<'a> {
    pass: wgpu::RenderPass<'a>,
    target: &'a Image,
    resolve: Option<&'a Image>,
}

impl<'a> Canvas<'a> {
    /// Create a new [Canvas] from an image. This will allow for drawing to a single color image.
    ///
    /// The image must be created for Canvas usage, i.e. [Image::new_canvas_image], or [ScreenImage], and must only have a sample count of 1.
    pub fn from_image(ctx: &'a mut Context, load_op: CanvasLoadOp, image: &'a Image) -> Self {
        assert!(image.samples() == 1);

        Canvas {
            pass: ctx.gfx_context.fcx.as_mut().unwrap().cmd.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: image.view.as_ref(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: match load_op {
                                CanvasLoadOp::DontClear => wgpu::LoadOp::Load,
                                CanvasLoadOp::Clear(color) => wgpu::LoadOp::Clear(color.into()),
                            },
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                },
            ),
            target: image,
            resolve: None,
        }
    }

    /// Create a new [Canvas] from an MSAA image and a resolve target. This will allow for drawing with MSAA to a color image, then resolving the samples into a secondary target.
    ///
    /// Both images must be created for Canvas usage (see [Canvas::from_image]). `msaa_image` must have a sample count > 1 and `resolve_image` must strictly have a sample count of 1.
    pub fn from_msaa(
        ctx: &'a mut Context,
        load_op: CanvasLoadOp,
        msaa_image: &'a Image,
        resolve_image: &'a Image,
    ) -> Self {
        assert!(msaa_image.samples() > 1);
        assert!(resolve_image.samples() == 1);

        Canvas {
            pass: ctx.gfx_context.fcx.as_mut().unwrap().cmd.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: msaa_image.view.as_ref(),
                        resolve_target: Some(resolve_image.view.as_ref()),
                        ops: wgpu::Operations {
                            load: match load_op {
                                CanvasLoadOp::DontClear => wgpu::LoadOp::Load,
                                CanvasLoadOp::Clear(color) => wgpu::LoadOp::Clear(color.into()),
                            },
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                },
            ),
            target: msaa_image,
            resolve: Some(resolve_image),
        }
    }
}

/// Describes the image load operation when starting a new canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanvasLoadOp {
    /// Keep the existing contents of the image.
    DontClear,
    /// Clear the image contents to a solid color.
    Clear(Color),
}
