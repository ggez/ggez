use graphics::*;

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
    /// x/y shear factors expressed as a `Point2`.
    /// TODO: Should it be a Vector2?
    pub(crate) shear: Point2,
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
            shear: Point2::new(0.0, 0.0),
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

    /// TODO
    pub fn shear<P>(mut self, shear: P) -> Self
    where
        P: Into<mint::Point2<f32>>,
    {
        let p: mint::Point2<f32> = shear.into();
        self.shear = Point2::from(p);
        self
    }

    // TODO: Easy mirror functions for X and Y axis might be nice.
}

/// Create a DrawParam from a location
impl<P> From<(P,)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from(location: (P,)) -> Self {
        DrawParam::new().dest(location.0)
    }
}

/// Create a DrawParam from a location and color
impl<P, C> From<(P, C)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
    C: Into<Color>,
{
    fn from((location, color): (P, C)) -> Self {
        DrawParam::new().dest(location).color(color)
    }
}

/// Create a DrawParam from a location, rotation and color
impl<P, C> From<(P, f32, C)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
    C: Into<Color>,
{
    fn from((location, rotation, color): (P, f32, C)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .color(color)
    }
}

/// Create a DrawParam from a location, rotation, offset and color
impl<P, C> From<(P, f32, P, C)> for DrawParam
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
    }
}

/// Create a DrawParam from a location, rotation, offset, scale and color
impl<P, V, C> From<(P, f32, P, V, C)> for DrawParam
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
    }
}

/// A `DrawParam` that has been crunched down to a single matrix.
/// Useful for doing matrix-based coordiante transformations, I hope.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PrimitiveDrawParam {
    /// The transform matrix for the DrawParams
    pub matrix: Matrix4,
    /// a portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image (1.0) if omitted.
    pub src: Rect,
    /// A color to draw the target with.
    /// Default: white.
    pub color: Color,
}

impl Default for PrimitiveDrawParam {
    fn default() -> Self {
        PrimitiveDrawParam {
            matrix: na::one(),
            src: Rect::one(),
            color: WHITE,
        }
    }
}

impl From<DrawParam> for PrimitiveDrawParam {
    fn from(param: DrawParam) -> Self {
        let translate = Matrix4::new_translation(&Vec3::new(param.dest.x, param.dest.y, 0.0));
        let offset = Matrix4::new_translation(&Vec3::new(param.offset.x, param.offset.y, 0.0));
        let offset_inverse =
            Matrix4::new_translation(&Vec3::new(-param.offset.x, -param.offset.y, 0.0));
        let axis_angle = Vec3::z() * param.rotation;
        let rotation = Matrix4::new_rotation(axis_angle);
        let scale = Matrix4::new_nonuniform_scaling(&Vec3::new(param.scale.x, param.scale.y, 1.0));
        let shear = Matrix4::new(
            1.0,
            param.shear.x,
            0.0,
            0.0,
            param.shear.y,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );
        let transform = translate * offset * rotation * shear * scale * offset_inverse;
        PrimitiveDrawParam {
            src: param.src,
            color: param.color,
            matrix: transform,
        }
    }
}
