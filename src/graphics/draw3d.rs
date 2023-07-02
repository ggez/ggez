use glam::{Mat4, Vec3};

use crate::graphics::{Canvas3d, Color};

use super::{LinearColor, ZIndex};

/// A 3d version of `DrawParam` used for transformation of 3d meshes
#[derive(Clone, Copy, Debug)]
pub struct DrawParam3d {
    /// The transform of the mesh to draw see `Transform3d`
    pub transform: Transform3d,
    /// The alpha component is used for intensity of blending instead of actual alpha
    pub color: Color,
    /// This is used for the order of rendering
    pub z: ZIndex,
}

impl DrawParam3d {
    /// Change the scale of the `Transform3d`
    pub fn scale<V>(mut self, scale_: V) -> Self
    where
        V: Into<mint::Vector3<f32>>,
    {
        let p: mint::Vector3<f32> = scale_.into();
        let _ = self.transform.scale(p);
        self
    }

    /// Change the position of the `Transform3d`
    pub fn position<P>(mut self, position_: P) -> Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = position_.into();
        let _ = self.transform.position(p);
        self
    }

    /// Change the rotation of the `Transform3d`
    pub fn rotation<R>(mut self, rotation_: R) -> Self
    where
        R: Into<mint::Quaternion<f32>>,
    {
        let p: mint::Quaternion<f32> = rotation_.into();
        let _ = self.transform.rotation(p);
        self
    }

    /// Move the position by given amount
    pub fn translate<T>(mut self, translate_: T) -> Self
    where
        T: Into<mint::Vector3<f32>>,
    {
        let t: mint::Vector3<f32> = translate_.into();
        let _ = self.transform.translate(t);
        self
    }
    /// Change the pivot of the `DrawParam3d`
    pub fn pivot<P>(mut self, pivot_: P) -> Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = pivot_.into();
        let _ = self.transform.pivot(p);
        self
    }

    /// Change the offset of the `DrawParam3d`
    pub fn offset<O>(mut self, offset_: O) -> Self
    where
        O: Into<mint::Point3<f32>>,
    {
        let o: mint::Point3<f32> = offset_.into();
        let _ = self.transform.offset(o);
        self
    }
    /// Change the color of the `DrawParam3d`
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Change the transform of the `DrawParam3d`
    pub fn transform(mut self, transform: Transform3d) -> Self {
        self.transform = transform;
        self
    }
}

impl Default for DrawParam3d {
    fn default() -> Self {
        Self {
            transform: Transform3d::default(),
            color: Color::new(1.0, 1.0, 1.0, 0.0),
            z: 0,
        }
    }
}

/// Represents transformations for 3d objects
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Transform3d {
    /// Transform made of individual values
    Values {
        /// The position to draw the graphic expressed as a `Point2`.
        pos: mint::Point3<f32>,
        /// The orientation of the graphic in radians.
        rotation: mint::Quaternion<f32>,
        /// The x/y scale factors expressed as a `Vector2`.
        scale: mint::Vector3<f32>,
        /// An offset, which is applied before scaling and rotation happen.
        ///
        /// For most objects this works as a relative offset (meaning `[0.5,0.5]` is an offset which
        /// centers the object on the destination). These objects are:
        /// * `Image`, `Canvas`, `Text` and the sprites inside an `InstanceArray` (as long as you're
        /// not making an instanced mesh-draw)
        offset: Option<mint::Point3<f32>>,
        /// The pivot point or origin of the transform3d
        pivot: Option<mint::Point3<f32>>,
    },
    /// Transform made of an arbitrary matrix.
    ///
    /// It should represent the final model matrix of the given drawable.  This is useful for
    /// situations where, for example, you build your own hierarchy system, where you calculate
    /// matrices of each hierarchy item and store a calculated world-space model matrix of an item.
    /// This lets you implement transform stacks, skeletal animations, etc.
    Matrix(mint::ColumnMatrix4<f32>),
}

impl Default for Transform3d {
    fn default() -> Self {
        Transform3d::Values {
            pos: mint::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: glam::Quat::IDENTITY.into(),
            scale: mint::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            offset: None,
            pivot: None,
        }
    }
}

impl Transform3d {
    /// Change the scale of the `Transform3d`
    pub fn scale<V>(&mut self, scale_: V) -> &mut Self
    where
        V: Into<mint::Vector3<f32>>,
    {
        let p: mint::Vector3<f32> = scale_.into();
        if let Self::Values { scale, .. } = self {
            *scale = p;
        }
        self
    }

    /// Change the position of the `Transform3d`
    pub fn position<P>(&mut self, position_: P) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = position_.into();
        if let Self::Values { pos, .. } = self {
            *pos = p;
        }
        self
    }

    /// Change the rotation of the `Transform3d`
    pub fn rotation<R>(&mut self, rotation_: R) -> &mut Self
    where
        R: Into<mint::Quaternion<f32>>,
    {
        let p: mint::Quaternion<f32> = rotation_.into();
        if let Self::Values { rotation, .. } = self {
            *rotation = p;
        }
        self
    }

    /// Move the position by given amount
    pub fn translate<T>(&mut self, translate_: T) -> &mut Self
    where
        T: Into<mint::Vector3<f32>>,
    {
        let t: mint::Vector3<f32> = translate_.into();
        if let Self::Values { pos, .. } = self {
            *pos = (glam::Vec3::from(*pos) + glam::Vec3::from(t)).into();
        }
        self
    }
    /// Sets the pivot point basically the origin of the mesh
    pub fn pivot<P>(&mut self, pivot_: P) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = pivot_.into();
        if let Self::Values { pivot, .. } = self {
            *pivot = Some(p)
        }
        self
    }

    /// Change the offset of the `DrawParam3d`
    pub fn offset<O>(&mut self, offset_: O) -> &mut Self
    where
        O: Into<mint::Point3<f32>>,
    {
        let o: mint::Point3<f32> = offset_.into();
        if let Self::Values { offset, .. } = self {
            *offset = Some(o);
        }
        self
    }
    /// Crunches the transform down to a single matrix, if it's not one already.
    #[must_use]
    pub fn to_matrix(self) -> Self {
        Transform3d::Matrix(self.to_bare_matrix())
    }

    /// Same as `to_matrix()` but just returns a bare `mint` matrix.
    #[must_use]
    pub fn to_bare_matrix(self) -> mint::ColumnMatrix4<f32> {
        match self {
            Transform3d::Matrix(m) => m,
            Transform3d::Values {
                pos,
                rotation,
                scale,
                offset,
                pivot,
            } => {
                let offset = if let Some(offset) = offset {
                    offset
                } else {
                    Vec3::ZERO.into()
                };
                let pivot = if let Some(piv) = pivot {
                    Vec3::from(piv) + Vec3::from(offset)
                } else {
                    Vec3::from(pos) + Vec3::from(offset)
                };
                let transform = Mat4::from_translation(pivot)
                    * Mat4::from_scale(scale.into())
                    * Mat4::from_quat(rotation.into())
                    * Mat4::from_translation(-(pivot))
                    * Mat4::from_translation(Vec3::from(pos));

                transform.into()
            }
        }
    }
}

/// All types that can be drawn onto a canvas3d implement the `Drawable3d` trait.
pub trait Drawable3d {
    /// Draws the drawable onto the canvas.
    fn draw(&self, canvas: &mut Canvas3d, param: impl Into<DrawParam3d>);
}

#[derive(Debug, Copy, Clone, crevice::std140::AsStd140)]
pub(crate) struct DrawUniforms3d {
    pub color: mint::Vector4<f32>,
    pub model_transform: mint::ColumnMatrix4<f32>,
    pub camera_transform: mint::ColumnMatrix4<f32>,
}

#[allow(unsafe_code)]
unsafe impl bytemuck::Zeroable for DrawUniforms3d {}
#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for DrawUniforms3d {}

impl DrawUniforms3d {
    pub fn from_param(param: &DrawParam3d) -> Self {
        let param = match param.transform {
            Transform3d::Values { .. } => *param,
            Transform3d::Matrix(m) => {
                param.transform(Transform3d::Matrix(glam::Mat4::from(m).into()))
            }
        };

        let color = LinearColor::from(param.color);

        DrawUniforms3d {
            color: <[f32; 4]>::from(color).into(),
            model_transform: param.transform.to_bare_matrix(),
            camera_transform: Mat4::IDENTITY.into(),
        }
    }

    pub fn projection(mut self, projection: impl Into<mint::ColumnMatrix4<f32>>) -> Self {
        self.camera_transform = projection.into();
        self
    }
}
