//! The graphics module performs the perhaps most important task of ggez, which is
//! **drawing things onto the screen**.
//!
//! The rough workflow for this is usually as follows:
//! 1. Create something that you want to render ([`Mesh`]es, [`Image`]s, [`InstanceArray`]s or [`Text`]s).
//! 2. Create a [`Canvas`] to render them onto (usually by calling [`Canvas::from_frame`], to draw directly onto the screen).
//! 3. (Select a [custom shader] and/or [blend mode] if you desire.)
//! 4. Queue draw calls by calling the appropriate draw method on [`Canvas`] ([`Canvas::draw`], `Canvas::draw_<whatever>`).
//! 5. (Go back to step 3 if you want to add more draw calls with different shaders or blend modes.)
//! 6. Submit the draw queue by calling [`Canvas::finish`].
//!
//! A [`Canvas`] represents a single render pass, operating on a certain render target.
//! You can create [`Canvas`]es that render to an image, instead of directly to the screen, with
//! [`Canvas::from_image`] and other related functions. With these you can, for example, render your
//! scene onto an [`Image`] held by a [`Canvas`] first and then render that [`Image`] onto the screen,
//! using a different shader, to do some post-processing.
//!
//! The module also handles the creation of [`Image`]s and other drawable objects and the screen
//! coordinate system / projection matrix through [`Canvas`].
//!
//! [custom shader]:Canvas::set_shader
//! [blend mode]:Canvas::set_blend_mode

pub(crate) mod canvas;
pub(crate) mod context;
pub(crate) mod draw;
pub(crate) mod gpu;
pub(crate) mod image;
pub(crate) mod instance;
pub(crate) mod internal_canvas;
pub(crate) mod mesh;
pub(crate) mod sampler;
pub(crate) mod shader;
pub(crate) mod text;
mod types;

pub use lyon::tessellation::{FillOptions, FillRule, LineCap, LineJoin, StrokeOptions};
pub use {
    self::image::*, canvas::*, context::*, draw::*, instance::*, mesh::*, sampler::*, shader::*,
    text::*, types::*,
};

/// Applies `DrawParam` to `Rect`.
pub fn transform_rect(rect: Rect, param: DrawParam) -> Rect {
    match param.transform {
        Transform::Values {
            scale,
            offset,
            dest,
            rotation,
        } => {
            // first apply the offset
            let mut r = Rect {
                w: rect.w,
                h: rect.h,
                x: rect.x - offset.x * rect.w,
                y: rect.y - offset.y * rect.h,
            };
            // apply the scale
            let real_scale = (param.src.w * scale.x, param.src.h * scale.y);
            r.w = real_scale.0 * rect.w;
            r.h = real_scale.1 * rect.h;
            r.x *= real_scale.0;
            r.y *= real_scale.1;
            // apply the rotation
            r.rotate(rotation);
            // apply the destination translation
            r.x += dest.x;
            r.y += dest.y;

            r
        }
        Transform::Matrix(_m) => todo!("Fix me"),
    }
}
