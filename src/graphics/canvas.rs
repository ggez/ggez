use super::{image::Image, Color};
use crate::Context;

/// A canvas represents a render pass and is how you render primitives onto images.
#[derive(Debug)]
pub struct Canvas<'a> {
    pass: wgpu::RenderPass<'a>,
    target: &'a Image,
    resolve: Option<Image>,
}

impl<'a> Canvas<'a> {
    /// Create a new Canvas from an image. This will allow for drawing to a single color image.
    ///
    /// The image must be created for Canvas usage, i.e. [Image::new_canvas_image], or [ScreenImage].
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

    /* pub fn from_msaa(
        ctx: &'a mut Context,
        load_op: CanvasLoadOp,
        msaa_image: Image,
        resolve_image: Image,
    ) -> Self {
        assert!(msaa_image.samples() > 1);
        assert!(resolve_image.samples() == 1);

        let mut cmd = ctx
            .gfx_context
            .fcx
            .as_mut()
            .unwrap()
            .cmd
            .take()
            .expect("another canvas is already in progress");

        let pass = cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        });

        Canvas {
            cmd,
            ctx,
            pass,
            target: msaa_image,
            resolve: Some(resolve_image),
        }
    } */
}

pub enum CanvasLoadOp {
    DontClear,
    Clear(Color),
}
