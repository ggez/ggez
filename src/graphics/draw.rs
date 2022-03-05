//!

use super::{Color, Rect};

/// Parameters describing how a mesh should be drawn.
#[derive(Debug, Clone, Copy)]
pub struct DrawParam {
    /// If drawing with an image, this is multiplied with the image color
    /// Otherwise, this is the solid color the mesh is rendered with.
    pub color: Color,
    /// UV sub-region to sample the image from.
    pub src_rect: Rect,
    /// Offset of the mesh from the top-left of the screen.
    pub offset: mint::Point2<f32>,
    /// Scale of the mesh.
    pub scale: mint::Vector2<f32>,
    /// Origin of the transformations.
    pub origin: mint::Vector2<f32>,
    /// When `true`, `scale` is based on the image size.
    pub image_scale: bool,
    /// Rotation (in radians) of the mesh.
    pub rotation: f32,
    /// Z position (depth), useful for visual reordering of draws.
    ///
    /// If this is left as `None`, the `z` will be automatically determined
    /// such that the mesh is drawn in the same order `draw` was called.
    pub z: Option<f32>,
    /// Additional optional transformation matrix to apply.
    pub transform: Option<mint::ColumnMatrix4<f32>>,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            color: Color::WHITE,
            src_rect: Rect::one(),
            offset: glam::Vec2::ZERO.into(),
            scale: glam::Vec2::ONE.into(),
            origin: glam::Vec2::ZERO.into(),
            image_scale: true,
            rotation: 0.,
            z: None,
            transform: None,
        }
    }
}

impl DrawParam {
    /// Equivalent to `DrawParam::default()`.
    pub fn new() -> Self {
        DrawParam::default()
    }

    /// Sets the `color` field.
    pub fn color(self, color: impl Into<Color>) -> Self {
        DrawParam {
            color: color.into(),
            ..self
        }
    }

    /// Sets the `src_rect` field.
    pub fn src_rect(self, src_rect: Rect) -> Self {
        DrawParam { src_rect, ..self }
    }

    /// Sets the `offset` field.
    pub fn offset(self, offset: impl Into<mint::Point2<f32>>) -> Self {
        DrawParam {
            offset: offset.into(),
            ..self
        }
    }

    /// Sets the `scale` field.
    pub fn scale(self, scale: impl Into<mint::Vector2<f32>>) -> Self {
        DrawParam {
            scale: scale.into(),
            ..self
        }
    }

    /// Helper function for setting `offset` and `scale` together.
    pub fn dst_rect(self, dst_rect: Rect) -> Self {
        self.offset(glam::vec2(dst_rect.x, dst_rect.y))
            .scale(glam::vec2(dst_rect.w, dst_rect.h))
    }

    /// Sets the `origin` field.
    pub fn origin(self, origin: impl Into<mint::Vector2<f32>>) -> Self {
        DrawParam {
            origin: origin.into(),
            ..self
        }
    }

    /// Sets the `image_scale` field.
    pub fn image_scale(self, image_scale: bool) -> Self {
        DrawParam {
            image_scale,
            ..self
        }
    }

    /// Sets the `rotation` field.
    pub fn rotation(self, rotation: f32) -> Self {
        DrawParam { rotation, ..self }
    }

    /// Shorthand for `.rotation(rotation.to_radians())`.
    pub fn rotation_deg(self, rotation: f32) -> Self {
        self.rotation(rotation.to_radians())
    }

    /// Sets the `z` field.
    pub fn z(self, z: impl Into<Option<f32>>) -> Self {
        DrawParam {
            z: z.into(),
            ..self
        }
    }

    /// Sets the `transform` field.
    pub fn transform(self, transform: impl Into<Option<mint::ColumnMatrix4<f32>>>) -> Self {
        DrawParam {
            transform: transform.into(),
            ..self
        }
    }
}

#[derive(crevice::std430::AsStd430)]
pub(crate) struct DrawUniforms {
    pub color: mint::Vector4<f32>,
    pub src_rect: mint::Vector4<f32>,
    pub transform: mint::ColumnMatrix4<f32>,
}

impl DrawUniforms {
    pub fn from_param(mut param: DrawParam, mut scale: mint::Vector2<f32>) -> Self {
        if !param.image_scale {
            scale.x = 1.;
            scale.y = 1.;
        }

        let mut transform = glam::Mat4::from_translation(glam::vec3(
            param.offset.x,
            param.offset.y,
            param.z.unwrap_or(0.),
        )) * glam::Mat4::from_quat(glam::Quat::from_rotation_z(param.rotation))
            * glam::Mat4::from_translation(glam::vec3(-param.origin.x, -param.origin.y, 0.))
            * glam::Mat4::from_scale(glam::vec3(param.scale.x, param.scale.y, 0.))
            * glam::Mat4::from_scale(glam::vec3(scale.x, scale.y, 0.));

        if let Some(t) = param.transform {
            transform = glam::Mat4::from(t) * transform;
        }

        DrawUniforms {
            color: <[f32; 4]>::from(param.color).into(),
            src_rect: mint::Vector4 {
                x: param.src_rect.x,
                y: param.src_rect.y,
                z: param.src_rect.x + param.src_rect.w,
                w: param.src_rect.y + param.src_rect.h,
            },
            transform: transform.into(),
        }
    }
}
