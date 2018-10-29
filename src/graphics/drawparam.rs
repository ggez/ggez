use crate::graphics::*;

use mint;

type Vec3 = na::Vector3<f32>;

/// A struct containing all the necessary info for drawing a Drawable.
///
/// This struct implements the `Default` trait, so to set only some parameter
/// you can just do:
///
/// ```rust,ignore
/// graphics::draw_ex(ctx, drawable, DrawParam{ dest: my_dest, .. Default::default()} )
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawParam {
    /// a portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image (1.0) if omitted.
    pub(crate) src: Rect,
    /// the position to draw the graphic expressed as a `Point2`.
    pub(crate) dest: Point2,
    /// orientation of the graphic in radians.
    pub(crate) rotation: f32,
    /// x/y scale factors expressed as a `Vector2`.
    pub(crate) scale: Vector2,
    /// specifies an offset from the center for transform operations like scale/rotation,
    /// with `0,0` meaning the origin and `1,1` meaning the opposite corner from the origin.
    /// By default these operations are done from the top-left corner, so to rotate something
    /// from the center specify `Point2::new(0.5, 0.5)` here.
    pub(crate) offset: Point2,
    /// A color to draw the target with.
    /// Default: white.
    pub(crate) color: Color,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            src: Rect::one(),
            dest: Point2::origin(),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
            offset: Point2::new(0.0, 0.0),
            color: WHITE,
        }
    }
}

impl DrawParam {
    /// Create a new DrawParam with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the source rect
    pub fn src(mut self, src: Rect) -> Self {
        self.src = src;
        self
    }

    /// Set the dest point
    pub fn dest<P>(mut self, dest: P) -> Self
    where
        P: Into<mint::Point2<f32>>,
    {
        let p: mint::Point2<f32> = dest.into();
        self.dest = Point2::from(p);
        self
    }

    /// TODO
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.color = color.into();
        self
    }

    /// TODO
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// TODO
    pub fn scale<V>(mut self, scale: V) -> Self
    where
        V: Into<mint::Vector2<f32>>,
    {
        let p: mint::Vector2<f32> = scale.into();
        self.scale = Vector2::from(p);
        self
    }

    /// TODO
    pub fn offset<P>(mut self, offset: P) -> Self
    where
        P: Into<mint::Point2<f32>>,
    {
        let p: mint::Point2<f32> = offset.into();
        self.offset = Point2::from(p);
        self
    }

    // TODO: Easy mirror functions for X and Y axis might be nice.
}

/// Create a DrawTransform from a location
impl<P> From<(P,)> for DrawTransform
where
    P: Into<mint::Point2<f32>>,
{
    fn from(location: (P,)) -> Self {
        DrawParam::new().dest(location.0).into()
    }
}

/// Create a DrawTransform from a location and color
impl<P, C> From<(P, C)> for DrawTransform
where
    P: Into<mint::Point2<f32>>,
    C: Into<Color>,
{
    fn from((location, color): (P, C)) -> Self {
        DrawParam::new().dest(location).color(color).into()
    }
}

/// Create a DrawTransform from a location, rotation and color
impl<P, C> From<(P, f32, C)> for DrawTransform
where
    P: Into<mint::Point2<f32>>,
    C: Into<Color>,
{
    fn from((location, rotation, color): (P, f32, C)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .color(color)
            .into()
    }
}

/// Create a DrawTransform from a location, rotation, offset and color
impl<P, C> From<(P, f32, P, C)> for DrawTransform
where
    P: Into<mint::Point2<f32>>,
    C: Into<Color>,
{
    fn from((location, rotation, offset, color): (P, f32, P, C)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .offset(offset)
            .color(color)
            .into()
    }
}

/// Create a DrawTransform from a location, rotation, offset, scale and color
impl<P, V, C> From<(P, f32, P, V, C)> for DrawTransform
where
    P: Into<mint::Point2<f32>>,
    V: Into<mint::Vector2<f32>>,
    C: Into<Color>,
{
    fn from((location, rotation, offset, scale, color): (P, f32, P, V, C)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .offset(offset)
            .scale(scale)
            .color(color)
            .into()
    }
}

/// A `DrawParam` that has been crunched down to a single matrix.
/// Useful for doing matrix-based coordinate transformations, I hope.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawTransform {
    /// The transform matrix for the DrawParams
    pub matrix: Matrix4,
    /// A portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image (1.0) if omitted.
    pub src: Rect,
    /// A color to draw the target with.
    /// Default: white.
    pub color: Color,
}

impl Default for DrawTransform {
    fn default() -> Self {
        DrawTransform {
            matrix: na::one(),
            src: Rect::one(),
            color: WHITE,
        }
    }
}

impl From<DrawParam> for DrawTransform {
    fn from(param: DrawParam) -> Self {
        let translate = Matrix4::new_translation(&Vec3::new(param.dest.x, param.dest.y, 0.0));
        let offset = Matrix4::new_translation(&Vec3::new(param.offset.x, param.offset.y, 0.0));
        let offset_inverse =
            Matrix4::new_translation(&Vec3::new(-param.offset.x, -param.offset.y, 0.0));
        let axis_angle = Vec3::z() * param.rotation;
        let rotation = Matrix4::new_rotation(axis_angle);
        let scale = Matrix4::new_nonuniform_scaling(&Vec3::new(param.scale.x, param.scale.y, 1.0));
        let transform = translate * offset * rotation * scale * offset_inverse;
        DrawTransform {
            src: param.src,
            color: param.color,
            matrix: transform,
        }
    }
}

impl DrawTransform {
    /// Returns a new `PrimitiveDrawParam` with its matrix multiplied
    /// by the given one.
    ///
    /// TODO: Make some way to implement `matrix * self.matrix`, or just implement `Mul`...
    pub fn mul(self, matrix: Matrix4) -> Self {
        DrawTransform {
            matrix: self.matrix * matrix,
            ..self
        }
    }

    pub(crate) fn to_instance_properties(&self, srgb: bool) -> InstanceProperties {
        let mat: [[f32; 4]; 4] = self.matrix.into();
        let color: [f32; 4] = if srgb {
            let linear_color: types::LinearColor = self.color.into();
            linear_color.into()
        } else {
            self.color.into()
        };
        InstanceProperties {
            src: self.src.into(),
            col1: mat[0],
            col2: mat[1],
            col3: mat[2],
            col4: mat[3],
            color,
        }
    }
}
