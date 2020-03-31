use crate::graphics::*;

use mint;

/// A struct containing all the necessary info for drawing a [`Drawable`](trait.Drawable.html).
///
/// This struct implements the `Default` trait, so to set only some parameter
/// you can just do:
///
/// ```rust
/// # use ggez::*;
/// # use ggez::graphics::*;
/// # fn t<P>(ctx: &mut Context, drawable: &P) where P: Drawable {
/// let my_dest = nalgebra::Point2::new(13.0, 37.0);
/// graphics::draw(ctx, drawable, DrawParam::default().dest(my_dest) );
/// # }
/// ```
///
/// As a shortcut, it also implements `From` for a variety of tuple types.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawParam {
    /// A portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image `(0,0 to 1,1)` if omitted.
    pub src: Rect,
    /// The position to draw the graphic expressed as a `Point2`.
    pub dest: mint::Point2<f32>,
    /// The orientation of the graphic in radians.
    pub rotation: f32,
    /// The x/y scale factors expressed as a `Vector2`.
    pub scale: mint::Vector2<f32>,
    /// An offset from the center for transform operations like scale/rotation,
    /// with `0,0` meaning the origin and `1,1` meaning the opposite corner from the origin.
    /// By default these operations are done from the top-left corner, so to rotate something
    /// from the center specify `Point2::new(0.5, 0.5)` here.
    pub offset: mint::Point2<f32>,
    /// A color to draw the target with.
    /// Default: white.
    pub color: Color,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            src: Rect::one(),
            dest: mint::Point2 { x: 0.0, y: 0.0 },
            rotation: 0.0,
            scale: mint::Vector2 { x: 1.0, y: 1.0 },
            offset: mint::Point2 { x: 0.0, y: 0.0 },
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
        self.dest = p;
        self
    }

    /// Set the drawable color.  This will be blended with whatever
    /// color the drawn object already is.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the rotation of the drawable.
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the scaling factors of the drawable.
    pub fn scale<V>(mut self, scale: V) -> Self
    where
        V: Into<mint::Vector2<f32>>,
    {
        let p: mint::Vector2<f32> = scale.into();
        self.scale = p;
        self
    }

    /// Set the transformation offset of the drawable.
    pub fn offset<P>(mut self, offset: P) -> Self
    where
        P: Into<mint::Point2<f32>>,
    {
        let p: mint::Point2<f32> = offset.into();
        self.offset = p;
        self
    }

    /// A [`DrawParam`](struct.DrawParam.html) that has been crunched down to a single matrix.
    fn to_na_matrix(&self) -> Matrix4 {
        // Calculate a matrix equivalent to doing this:
        //  type Vec3 = na::Vector3<f32>;
        //  let translate = Matrix4::new_translation(&Vec3::new(self.dest.x, self.dest.y, 0.0));
        //  let offset = Matrix4::new_translation(&Vec3::new(self.offset.x, self.offset.y, 0.0));
        //  let offset_inverse =
        //      Matrix4::new_translation(&Vec3::new(-self.offset.x, -self.offset.y, 0.0));
        //  let axis_angle = Vec3::z() * self.rotation;
        //  let rotation = Matrix4::new_rotation(axis_angle);
        //  let scale = Matrix4::new_nonuniform_scaling(&Vec3::new(self.scale.x, self.scale.y, 1.0));
        //  translate * offset * rotation * scale * offset_inverse
        let cosr = self.rotation.cos();
        let sinr = self.rotation.sin();
        let m00 = cosr * self.scale.x;
        let m01 = -sinr * self.scale.y;
        let m10 = sinr * self.scale.x;
        let m11 = cosr * self.scale.y;
        let m03 = self.offset.x * (1.0 - m00) - self.offset.y * m01 + self.dest.x;
        let m13 = self.offset.y * (1.0 - m11) - self.offset.x * m10 + self.dest.y;
        Matrix4::new(m00, m01, 0.0, m03,  m10, m11, 0.0, m13,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0)
    }

    /// A [`DrawParam`](struct.DrawParam.html) that has been crunched down to a single
    ///matrix.  Because of this it only contains the transform part (rotation/scale/etc),
    /// with no src/dest/color info.
    pub fn to_matrix(&self) -> mint::ColumnMatrix4<f32> {
        self.to_na_matrix().into()
    }
}

/// Create a `DrawParam` from a location.
/// Note that this takes a single-element tuple.
/// It's a little weird but keeps the trait implementations
/// from clashing.
impl<P> From<(P,)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from(location: (P,)) -> Self {
        DrawParam::new().dest(location.0)
    }
}

/// Create a `DrawParam` from a location and color
impl<P> From<(P, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from((location, color): (P, Color)) -> Self {
        DrawParam::new().dest(location).color(color)
    }
}

/// Create a `DrawParam` from a location, rotation and color
impl<P> From<(P, f32, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from((location, rotation, color): (P, f32, Color)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .color(color)
    }
}

/// Create a `DrawParam` from a location, rotation, offset and color
impl<P> From<(P, f32, P, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from((location, rotation, offset, color): (P, f32, P, Color)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .offset(offset)
            .color(color)
    }
}

/// Create a `DrawParam` from a location, rotation, offset, scale and color
impl<P, V> From<(P, f32, P, V, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
    V: Into<mint::Vector2<f32>>,
{
    fn from((location, rotation, offset, scale, color): (P, f32, P, V, Color)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .offset(offset)
            .scale(scale)
            .color(color)
    }
}

/// A [`DrawParam`](struct.DrawParam.html) that has been crunched down to a single matrix.
/// This is a lot less useful for doing transformations than I'd hoped; basically, we sometimes
/// have to modify parameters of a `DrawParam` based *on* the parameters of a `DrawParam`, for
/// instance when scaling images so that they are in units of pixels.  This makes it really
/// hard to extract scale and rotation and such, so meh.
///
/// It's still useful for a couple internal things though, so it's kept around.
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct DrawTransform {
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
        let transform = param.to_matrix();
        DrawTransform {
            src: param.src,
            color: param.color,
            matrix: transform.into(),
        }
    }
}

impl DrawTransform {
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
