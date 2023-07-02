use crate::{
    context::HasMut,
    graphics::{Canvas3d, Color, Shader},
};

use super::GraphicsContext;

/// A 3d version of `DrawParam` used for transformation of 3d meshes
#[derive(Clone, Copy, Debug)]
pub struct DrawParam3d {
    /// The transform of the mesh to draw see `Transform3d`
    pub transform: Transform3d,
    /// The alpha component is used for intensity of blending instead of actual alpha
    pub color: Color,
    /// Pivot point for the mesh rotation and scaling in world space
    pub pivot: Option<mint::Point3<f32>>,
    /// Pivot point for the mesh rotation and scaling relative to the position of the mesh
    pub offset: Option<mint::Point3<f32>>,
}

impl DrawParam3d {
    /// Change the scale of the `DrawParam3d`
    pub fn scale<V>(mut self, scale_: V) -> Self
    where
        V: Into<mint::Vector3<f32>>,
    {
        let p: mint::Vector3<f32> = scale_.into();
        self.transform.scale = p;
        self
    }

    /// Change the position of the `DrawParam3d`
    pub fn position<P>(mut self, position_: P) -> Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = position_.into();
        self.transform.position = p;
        self
    }

    /// Change the pivot of the `DrawParam3d`
    pub fn pivot<P>(mut self, pivot_: P) -> Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = pivot_.into();
        self.pivot = Some(p);
        self
    }

    /// Change the offset of the `DrawParam3d`
    pub fn offset<O>(mut self, offset_: O) -> Self
    where
        O: Into<mint::Point3<f32>>,
    {
        let o: mint::Point3<f32> = offset_.into();
        self.offset = Some(o);
        self
    }

    /// Change the rotation of the `DrawParam3d`
    pub fn rotation<R>(mut self, rotation_: R) -> Self
    where
        R: Into<mint::Quaternion<f32>>,
    {
        let p: mint::Quaternion<f32> = rotation_.into();
        self.transform.rotation = p;
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
            pivot: None,
            offset: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DrawState3d {
    pub(crate) shader: Shader,
}

/// Transform3d is used to transform 3d objects.
#[derive(Debug, Copy, Clone)]
pub struct Transform3d {
    /// The position or translation of this `Transform3d`
    pub position: mint::Point3<f32>,
    /// The rotation of this `Transform3d`
    pub rotation: mint::Quaternion<f32>,
    /// The scale of this `Transform3d`
    pub scale: mint::Vector3<f32>,
}

impl Default for Transform3d {
    fn default() -> Self {
        Self {
            position: glam::Vec3::new(0.0, 0.0, 0.0).into(),
            rotation: glam::Quat::IDENTITY.into(),
            scale: glam::Vec3::new(1.0, 1.0, 1.0).into(),
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
        self.scale = p;
        self
    }

    /// Change the position of the `Transform3d`
    pub fn position<P>(&mut self, position_: P) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = position_.into();
        self.position = p;
        self
    }

    /// Change the rotation of the `Transform3d`
    pub fn rotation<R>(&mut self, rotation_: R) -> &mut Self
    where
        R: Into<mint::Quaternion<f32>>,
    {
        let p: mint::Quaternion<f32> = rotation_.into();
        self.rotation = p;
        self
    }

    /// Move the position by given amount
    pub fn translate<T>(&mut self, translate_: T) -> &mut Self
    where
        T: Into<mint::Vector3<f32>>,
    {
        let t: mint::Vector3<f32> = translate_.into();
        self.position(glam::Vec3::from(self.position) + glam::Vec3::from(t))
    }
}

/// All types that can be drawn onto a canvas3d implement the `Drawable3d` trait.
pub trait Drawable3d {
    /// Draws the drawable onto the canvas.
    fn draw(
        &self,
        gfx: &mut impl HasMut<GraphicsContext>,
        canvas: &mut Canvas3d,
        param: impl Into<DrawParam3d>,
    );
}
