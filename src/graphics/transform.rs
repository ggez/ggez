//!

use crate::graphics::*;

/// A struct that represents where to put a `Drawable`.
///
/// This can either be a set of individual components, or
/// a single `Matrix4` transform.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Transform {
    /// Transform made of individual values
    Values {
        /// The position to draw the graphic expressed as a `Point2`.
        dest: mint::Point2<f32>,
        /// The orientation of the graphic in radians.
        rotation: f32,
        /// The x/y scale factors expressed as a `Vector2`.
        scale: mint::Vector2<f32>,
        /// An offset, which is applied before scaling and rotation happen.
        /// There are two possible interpretations of this value:
        ///
        /// + `Image`, `Canvas` and the sprites inside a `SpriteBatch` use the relative interpretation
        /// + `Mesh`, `MeshBatch`, `Spritebatch` (and thereby `Text` too) use the absolute interpretation
        ///
        /// The relative interpretation would be that `0.5,0.5` means "centered" and `1,1` means "bottom right".
        /// By default these operations are done from the top-left corner, so to rotate something
        /// from the center specify `Point2::new(0.5, 0.5)` here.
        ///
        /// The absolute interpretation considers the offset as a shift given in coordinates of the current coordinate system.
        ///
        /// For more info on this check the [FAQ](https://github.com/ggez/ggez/blob/devel/docs/FAQ.md#offsets)
        offset: mint::Point2<f32>,
    },
    /// Transform made of an arbitrary matrix.
    ///
    /// It should represent the final model matrix of the given drawable.  This is useful for
    /// situations where, for example, you build your own hierarchy system, where you calculate
    /// matrices of each hierarchy item and store a calculated world-space model matrix of an item.
    /// This lets you implement transform stacks, skeletal animations, etc.
    Matrix(mint::ColumnMatrix4<f32>),
}

impl Default for Transform {
    fn default() -> Self {
        Transform::Values {
            dest: mint::Point2 { x: 0.0, y: 0.0 },
            rotation: 0.0,
            scale: mint::Vector2 { x: 1.0, y: 1.0 },
            offset: mint::Point2 { x: 0.0, y: 0.0 },
        }
    }
}

impl Transform {
    /// Crunches the transform down to a single matrix, if it's not one already.
    pub fn to_matrix(self) -> Self {
        Transform::Matrix(self.to_bare_matrix())
    }

    /// Same as `to_matrix()` but just returns a bare `mint` matrix.
    pub fn to_bare_matrix(self) -> mint::ColumnMatrix4<f32> {
        match self {
            Transform::Matrix(m) => m,
            Transform::Values {
                dest,
                rotation,
                scale,
                offset,
            } => {
                // Calculate a matrix equivalent to doing this:
                // type Vec3 = na::Vector3<f32>;
                // let o = offset;
                // let translate = na::Matrix4::new_translation(&Vec3::new(dest.x, dest.y, 0.0));
                // let offset = na::Matrix4::new_translation(&Vec3::new(offset.x, offset.y, 0.0));
                // let offset_inverse =
                //     na::Matrix4::new_translation(&Vec3::new(-o.x, -o.y, 0.0));
                // let axis_angle = Vec3::z() * *rotation;
                // let rotation = na::Matrix4::new_rotation(axis_angle);
                // let scale = na::Matrix4::new_nonuniform_scaling(&Vec3::new(scale.x, scale.y, 1.0));
                // translate * rotation * scale * offset_inverse
                //
                // Doing the bits manually is faster though, or at least was last I checked.
                let (sinr, cosr) = rotation.sin_cos();
                let m00 = cosr * scale.x;
                let m01 = -sinr * scale.y;
                let m10 = sinr * scale.x;
                let m11 = cosr * scale.y;
                let m03 = offset.x * (-m00) - offset.y * m01 + dest.x;
                let m13 = offset.y * (-m11) - offset.x * m10 + dest.y;
                // Welp, this transpose fixes some bug that makes nothing draw,
                // that was introduced in commit 2c6b3cc03f34fb240f4246f5a68c75bd85b60eae.
                // The best part is, I don't know if this code is wrong, or whether there's
                // some reversed matrix multiply or such somewhere else that this cancel
                // out.  Probably the former though.
                Matrix4::from_cols_array(&[
                    m00, m01, 0.0, m03, // oh rustfmt you so fine
                    m10, m11, 0.0, m13, // you so fine you blow my mind
                    0.0, 0.0, 1.0, 0.0, // but leave my matrix formatting alone
                    0.0, 0.0, 0.0, 1.0, // plz
                ])
                .transpose()
                .into()
            }
        }
    }
}
