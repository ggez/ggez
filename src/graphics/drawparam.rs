use graphics::*;

/// A struct containing all the necessary info for drawing a [`Drawable`](trait.Drawable.html).
///
/// This struct implements the `Default` trait, so to set only some parameter
/// you can just do:
///
/// ```rust
/// # use ggez::*;
/// # use ggez::graphics::*;
/// # fn t(ctx: &mut Context, drawable: &Drawable) {
/// let my_dest = Point2::new(13.0, 37.0);
/// graphics::draw_ex(ctx, drawable, DrawParam{ dest: my_dest, .. Default::default()} );
/// # }
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawParam {
    /// a portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image (1.0) if omitted.
    pub src: Rect,
    /// the position to draw the graphic expressed as a `Point2`.
    pub dest: Point2,
    /// orientation of the graphic in radians.
    pub rotation: f32,
    /// x/y scale factors expressed as a `Point2`.
    pub scale: Point2,
    /// specifies an offset from the center for transform operations like scale/rotation,
    /// with `0,0` meaning the origin and `1,1` meaning the opposite corner from the origin.
    /// By default these operations are done from the top-left corner, so to rotate something
    /// from the center specify `Point2::new(0.5, 0.5)` here.
    pub offset: Point2,
    /// x/y shear factors expressed as a `Point2`.
    pub shear: Point2,
    /// A color to draw the target with.
    /// If `None`, the color set by [`graphics::set_color()`](fn.set_color.html) is used; default white.
    pub color: Option<Color>,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            src: Rect::one(),
            dest: Point2::origin(),
            rotation: 0.0,
            scale: Point2::new(1.0, 1.0),
            offset: Point2::new(0.0, 0.0),
            shear: Point2::new(0.0, 0.0),
            color: None,
        }
    }
}

impl DrawParam {
    /// Turn the DrawParam into a model matrix, combining
    /// destination, rotation, scale, offset and shear.
    pub fn into_matrix(self) -> Matrix4 {
        type Vec3 = na::Vector3<f32>;
        let translate = Matrix4::new_translation(&Vec3::new(self.dest.x, self.dest.y, 0.0));
        let offset = Matrix4::new_translation(&Vec3::new(self.offset.x, self.offset.y, 0.0));
        let offset_inverse =
            Matrix4::new_translation(&Vec3::new(-self.offset.x, -self.offset.y, 0.0));
        let axis_angle = Vec3::z() * self.rotation;
        let rotation = Matrix4::new_rotation(axis_angle);
        let scale = Matrix4::new_nonuniform_scaling(&Vec3::new(self.scale.x, self.scale.y, 1.0));
        let shear = Matrix4::new(
            1.0,
            self.shear.x,
            0.0,
            0.0,
            self.shear.y,
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
        translate * offset * rotation * shear * scale * offset_inverse
    }
}
