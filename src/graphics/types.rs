pub(crate) use nalgebra as na;
use std::f32;
use std::u32;

use crate::graphics::{FillOptions, StrokeOptions};

/// A 2 dimensional point representing a location
pub(crate) type Point2 = na::Point2<f32>;
/// A 2 dimensional vector representing an offset of a location
pub(crate) type Vector2 = na::Vector2<f32>;
/// A 4 dimensional matrix representing an arbitrary 3d transformation
pub(crate) type Matrix4 = na::Matrix4<f32>;

/// A simple 2D rectangle.
///
/// The origin of the rectangle is at the top-left,
/// with x increasing to the right and y increasing down.
#[derive(Copy, Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Rect {
    /// X coordinate of the left edge of the rect.
    pub x: f32,
    /// Y coordinate of the top edge of the rect.
    pub y: f32,
    /// Total width of the rect
    pub w: f32,
    /// Total height of the rect.
    pub h: f32,
}

impl Rect {
    /// Create a new `Rect`.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    /// Creates a new `Rect` a la Love2D's `love.graphics.newQuad`,
    /// as a fraction of the reference rect's size.
    pub fn fraction(x: f32, y: f32, w: f32, h: f32, reference: &Rect) -> Rect {
        Rect {
            x: x / reference.w,
            y: y / reference.h,
            w: w / reference.w,
            h: h / reference.h,
        }
    }

    /// Create a new rect from `i32` coordinates.
    pub const fn new_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        }
    }

    /// Create a new `Rect` with all values zero.
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a new `Rect` at `0,0` with width and height 1.
    pub const fn one() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Gets the `Rect`'s x and y coordinates as a `Point2`.
    pub const fn point(&self) -> mint::Point2<f32> {
        mint::Point2 {
            x: self.x,
            y: self.y,
        }
    }

    /// Returns the left edge of the `Rect`
    pub const fn left(&self) -> f32 {
        self.x
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Returns the top edge of the `Rect`
    pub const fn top(&self) -> f32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<mint::Point2<f32>>,
    {
        let point = point.into();
        point.x >= self.left()
            && point.x <= self.right()
            && point.y <= self.bottom()
            && point.y >= self.top()
    }

    /// Checks whether the `Rect` overlaps another `Rect`
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Translates the `Rect` by an offset of (x, y)
    pub fn translate<V>(&mut self, offset: V)
    where
        V: Into<mint::Vector2<f32>>,
    {
        let offset = offset.into();
        self.x += offset.x;
        self.y += offset.y;
    }

    /// Moves the `Rect`'s origin to (x, y)
    pub fn move_to<P>(&mut self, destination: P)
    where
        P: Into<mint::Point2<f32>>,
    {
        let destination = destination.into();
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Rect` by a factor of (sx, sy),
    /// growing towards the bottom-left
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.w *= sx;
        self.h *= sy;
    }

    /// Calculated the new Rect around the rotated one.
    pub fn rotate(&mut self, rotation: f32) {
        let rotation = na::Rotation2::new(rotation);
        let x0 = self.x;
        let y0 = self.y;
        let x1 = self.right();
        let y1 = self.bottom();
        let points = [
            rotation * na::Point2::new(x0, y0),
            rotation * na::Point2::new(x0, y1),
            rotation * na::Point2::new(x1, y0),
            rotation * na::Point2::new(x1, y1),
        ];
        let p0 = points[0];
        let mut x_max = p0.x;
        let mut x_min = p0.x;
        let mut y_max = p0.y;
        let mut y_min = p0.y;
        for p in &points {
            x_max = f32::max(x_max, p.x);
            x_min = f32::min(x_min, p.x);
            y_max = f32::max(y_max, p.y);
            y_min = f32::min(y_min, p.y);
        }
        *self = Rect {
            w: x_max - x_min,
            h: y_max - y_min,
            x: x_min,
            y: y_min,
        }
    }

    /// Returns a new `Rect` that includes all points of these two `Rect`s.
    pub fn combine_with(self, other: Rect) -> Rect {
        let x = f32::min(self.x, other.x);
        let y = f32::min(self.y, other.y);
        let w = f32::max(self.right(), other.right()) - x;
        let h = f32::max(self.bottom(), other.bottom()) - y;
        Rect { x, y, w, h }
    }
}

impl approx::AbsDiffEq for Rect {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f32::abs_diff_eq(&self.x, &other.x, epsilon)
            && f32::abs_diff_eq(&self.y, &other.y, epsilon)
            && f32::abs_diff_eq(&self.w, &other.w, epsilon)
            && f32::abs_diff_eq(&self.h, &other.h, epsilon)
    }
}

impl approx::RelativeEq for Rect {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        f32::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && f32::relative_eq(&self.y, &other.y, epsilon, max_relative)
            && f32::relative_eq(&self.w, &other.w, epsilon, max_relative)
            && f32::relative_eq(&self.h, &other.h, epsilon, max_relative)
    }
}

impl From<[f32; 4]> for Rect {
    fn from(val: [f32; 4]) -> Self {
        Rect::new(val[0], val[1], val[2], val[3])
    }
}

impl From<Rect> for [f32; 4] {
    fn from(val: Rect) -> Self {
        [val.x, val.y, val.w, val.h]
    }
}

/// A RGBA color in the `sRGB` color space represented as `f32`'s in the range `[0.0-1.0]`
///
/// For convenience, [`WHITE`](constant.WHITE.html) and [`BLACK`](constant.BLACK.html) are provided.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Color {
    /// Red component
    pub r: f32,
    /// Green component
    pub g: f32,
    /// Blue component
    pub b: f32,
    /// Alpha component
    pub a: f32,
}

/// White
pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

/// Black
pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

impl Color {
    /// Create a new `Color` from four `f32`'s in the range `[0.0-1.0]`
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Create a new `Color` from four `u8`'s in the range `[0-255]`
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::from((r, g, b, a))
    }

    /// Create a new `Color` from three u8's in the range `[0-255]`,
    /// with the alpha component fixed to 255 (opaque)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color::from((r, g, b))
    }

    /// Return a tuple of four `u8`'s in the range `[0-255]` with the `Color`'s
    /// components.
    pub fn to_rgba(self) -> (u8, u8, u8, u8) {
        self.into()
    }

    /// Return a tuple of three `u8`'s in the range `[0-255]` with the `Color`'s
    /// components.
    pub fn to_rgb(self) -> (u8, u8, u8) {
        self.into()
    }

    /// Convert a packed `u32` containing `0xRRGGBBAA` into a `Color`
    pub fn from_rgba_u32(c: u32) -> Color {
        let c = c.to_be_bytes();

        Color::from((c[0], c[1], c[2], c[3]))
    }

    /// Convert a packed `u32` containing `0x00RRGGBB` into a `Color`.
    /// This lets you do things like `Color::from_rgb_u32(0xCD09AA)` easily if you want.
    pub fn from_rgb_u32(c: u32) -> Color {
        let c = c.to_be_bytes();

        Color::from((c[1], c[2], c[3]))
    }

    /// Convert a `Color` into a packed `u32`, containing `0xRRGGBBAA` as bytes.
    pub fn to_rgba_u32(self) -> u32 {
        let (r, g, b, a): (u8, u8, u8, u8) = self.into();

        u32::from_be_bytes([r, g, b, a])
    }

    /// Convert a `Color` into a packed `u32`, containing `0x00RRGGBB` as bytes.
    pub fn to_rgb_u32(self) -> u32 {
        let (r, g, b, _a): (u8, u8, u8, u8) = self.into();

        u32::from_be_bytes([0, r, g, b])
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    /// Convert a `(R, G, B, A)` tuple of `u8`'s in the range `[0-255]` into a `Color`
    fn from(val: (u8, u8, u8, u8)) -> Self {
        let (r, g, b, a) = val;
        let rf = (f32::from(r)) / 255.0;
        let gf = (f32::from(g)) / 255.0;
        let bf = (f32::from(b)) / 255.0;
        let af = (f32::from(a)) / 255.0;
        Color::new(rf, gf, bf, af)
    }
}

impl From<(u8, u8, u8)> for Color {
    /// Convert a `(R, G, B)` tuple of `u8`'s in the range `[0-255]` into a `Color`,
    /// with a value of 255 for the alpha element (i.e., no transparency.)
    fn from(val: (u8, u8, u8)) -> Self {
        let (r, g, b) = val;
        Color::from((r, g, b, 255))
    }
}

impl From<[f32; 4]> for Color {
    /// Turns an `[R, G, B, A] array of `f32`'s into a `Color` with no format changes.
    /// All inputs should be in the range `[0.0-1.0]`.
    fn from(val: [f32; 4]) -> Self {
        Color::new(val[0], val[1], val[2], val[3])
    }
}

impl From<(f32, f32, f32)> for Color {
    /// Convert a `(R, G, B)` tuple of `f32`'s in the range `[0.0-1.0]` into a `Color`,
    /// with a value of 1.0 to for the alpha element (ie, no transparency.)
    fn from(val: (f32, f32, f32)) -> Self {
        let (r, g, b) = val;
        Color::new(r, g, b, 1.0)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    /// Convert a `(R, G, B, A)` tuple of `f32`'s in the range `[0.0-1.0]` into a `Color`
    fn from(val: (f32, f32, f32, f32)) -> Self {
        let (r, g, b, a) = val;
        Color::new(r, g, b, a)
    }
}

impl From<Color> for (u8, u8, u8, u8) {
    /// Convert a `Color` into a `(R, G, B, A)` tuple of `u8`'s in the range of `[0-255]`.
    fn from(color: Color) -> Self {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        (r, g, b, a)
    }
}

impl From<Color> for (u8, u8, u8) {
    /// Convert a `Color` into a `(R, G, B)` tuple of `u8`'s in the range of `[0-255]`,
    /// ignoring the alpha term.
    fn from(color: Color) -> Self {
        let (r, g, b, _) = color.into();
        (r, g, b)
    }
}

impl From<Color> for [f32; 4] {
    /// Convert a `Color` into an `[R, G, B, A]` array of `f32`'s in the range of `[0.0-1.0]`.
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

/// A RGBA color in the *linear* color space,
/// suitable for shoving into a shader.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub(crate) struct LinearColor {
    /// Red component
    pub r: f32,
    /// Green component
    pub g: f32,
    /// Blue component
    pub b: f32,
    /// Alpha component
    pub a: f32,
}

impl From<Color> for LinearColor {
    /// Convert an (sRGB) Color into a linear color,
    /// per https://en.wikipedia.org/wiki/Srgb#The_reverse_transformation
    fn from(c: Color) -> Self {
        fn f(component: f32) -> f32 {
            let a = 0.055;
            if component <= 0.04045 {
                component / 12.92
            } else {
                ((component + a) / (1.0 + a)).powf(2.4)
            }
        }
        LinearColor {
            r: f(c.r),
            g: f(c.g),
            b: f(c.b),
            a: c.a,
        }
    }
}

impl From<LinearColor> for Color {
    fn from(c: LinearColor) -> Self {
        fn f(component: f32) -> f32 {
            let a = 0.055;
            if component <= 0.003_130_8 {
                component * 12.92
            } else {
                (1.0 + a) * component.powf(1.0 / 2.4)
            }
        }
        Color {
            r: f(c.r),
            g: f(c.g),
            b: f(c.b),
            a: c.a,
        }
    }
}

impl From<LinearColor> for [f32; 4] {
    fn from(color: LinearColor) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

/// Specifies whether a mesh should be drawn
/// filled or as an outline.
#[derive(Debug, Copy, Clone)]
pub enum DrawMode {
    /// A stroked line with given parameters, see `StrokeOptions` documentation.
    Stroke(StrokeOptions),
    /// A filled shape with given parameters, see `FillOptions` documentation.
    Fill(FillOptions),
}

impl DrawMode {
    /// Constructs a DrawMode that draws a stroke with the given width
    pub fn stroke(width: f32) -> DrawMode {
        DrawMode::Stroke(StrokeOptions::default().with_line_width(width))
    }

    /// Constructs a DrawMode that fills shapes with default fill options.
    pub fn fill() -> DrawMode {
        DrawMode::Fill(FillOptions::default())
    }
}

/// Specifies what blending method to use when scaling up/down images.
#[derive(Debug, Copy, Clone)]
pub enum FilterMode {
    /// Use linear interpolation (ie, smooth)
    Linear,
    /// Use nearest-neighbor interpolation (ie, pixelated)
    Nearest,
}

use gfx::texture::FilterMethod;

impl From<FilterMethod> for FilterMode {
    fn from(f: FilterMethod) -> Self {
        match f {
            FilterMethod::Scale => FilterMode::Nearest,
            _other => FilterMode::Linear,
        }
    }
}

impl From<FilterMode> for FilterMethod {
    fn from(f: FilterMode) -> Self {
        match f {
            FilterMode::Nearest => FilterMethod::Scale,
            FilterMode::Linear => FilterMethod::Bilinear,
        }
    }
}

/// Specifies how to wrap textures.
pub use gfx::texture::WrapMode;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn headless_test_color_conversions() {
        let white = Color::new(1.0, 1.0, 1.0, 1.0);
        let w1 = Color::from((255, 255, 255, 255));
        assert_eq!(white, w1);
        let w2: u32 = white.to_rgba_u32();
        assert_eq!(w2, 0xFFFF_FFFFu32);

        let grey = Color::new(0.5019608, 0.5019608, 0.5019608, 1.0);
        let g1 = Color::from((128, 128, 128, 255));
        assert_eq!(grey, g1);
        let g2: u32 = grey.to_rgba_u32();
        assert_eq!(g2, 0x8080_80FFu32);

        let black = Color::new(0.0, 0.0, 0.0, 1.0);
        let b1 = Color::from((0, 0, 0, 255));
        assert_eq!(black, b1);
        let b2: u32 = black.to_rgba_u32();
        assert_eq!(b2, 0x0000_00FFu32);
        assert_eq!(black, Color::from_rgb_u32(0x00_0000u32));
        assert_eq!(black, Color::from_rgba_u32(0x00_0000FFu32));

        let puce1 = Color::from_rgb_u32(0xCC_8899u32);
        let puce2 = Color::from_rgba_u32(0xCC88_99FFu32);
        let puce3 = Color::from((0xCC, 0x88, 0x99, 255));
        let puce4 = Color::new(0.80, 0.53333336, 0.60, 1.0);
        assert_eq!(puce1, puce2);
        assert_eq!(puce1, puce3);
        assert_eq!(puce1, puce4);
    }

    #[test]
    fn headless_test_rect_scaling() {
        let r1 = Rect::new(0.0, 0.0, 128.0, 128.0);
        let r2 = Rect::fraction(0.0, 0.0, 32.0, 32.0, &r1);
        assert_eq!(r2, Rect::new(0.0, 0.0, 0.25, 0.25));

        let r2 = Rect::fraction(32.0, 32.0, 32.0, 32.0, &r1);
        assert_eq!(r2, Rect::new(0.25, 0.25, 0.25, 0.25));
    }

    #[test]
    fn headless_test_rect_contains() {
        let r = Rect::new(0.0, 0.0, 128.0, 128.0);
        println!("{} {} {} {}", r.top(), r.bottom(), r.left(), r.right());
        let p = Point2::new(1.0, 1.0);
        assert!(r.contains(p));

        let p = Point2::new(500.0, 0.0);
        assert!(!r.contains(p));
    }

    #[test]
    fn headless_test_rect_overlaps() {
        let r1 = Rect::new(0.0, 0.0, 128.0, 128.0);
        let r2 = Rect::new(0.0, 0.0, 64.0, 64.0);
        assert!(r1.overlaps(&r2));

        let r2 = Rect::new(100.0, 0.0, 128.0, 128.0);
        assert!(r1.overlaps(&r2));

        let r2 = Rect::new(500.0, 0.0, 64.0, 64.0);
        assert!(!r1.overlaps(&r2));
    }

    #[test]
    fn headless_test_rect_transform() {
        let mut r1 = Rect::new(0.0, 0.0, 64.0, 64.0);
        let r2 = Rect::new(64.0, 64.0, 64.0, 64.0);
        r1.translate(Vector2::new(64.0, 64.0));
        assert!(r1 == r2);

        let mut r1 = Rect::new(0.0, 0.0, 64.0, 64.0);
        let r2 = Rect::new(0.0, 0.0, 128.0, 128.0);
        r1.scale(2.0, 2.0);
        assert!(r1 == r2);

        let mut r1 = Rect::new(32.0, 32.0, 64.0, 64.0);
        let r2 = Rect::new(64.0, 64.0, 64.0, 64.0);
        r1.move_to(Point2::new(64.0, 64.0));
        assert!(r1 == r2);
    }

    #[test]
    fn headless_test_rect_combine_with() {
        {
            let a = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let b = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let c = a.combine_with(b);
            assert_relative_eq!(a, b);
            assert_relative_eq!(a, c);
        }
        {
            let a = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 2.0,
            };
            let b = Rect {
                x: 0.0,
                y: 0.0,
                w: 2.0,
                h: 1.0,
            };
            let real = a.combine_with(b);
            let expected = Rect {
                x: 0.0,
                y: 0.0,
                w: 2.0,
                h: 2.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let a = Rect {
                x: -1.0,
                y: 0.0,
                w: 2.0,
                h: 2.0,
            };
            let b = Rect {
                x: 0.0,
                y: -1.0,
                w: 1.0,
                h: 1.0,
            };
            let real = a.combine_with(b);
            let expected = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 3.0,
            };
            assert_relative_eq!(real, expected);
        }
    }

    #[test]
    fn headless_test_rect_rotate() {
        {
            let mut r = Rect {
                x: -0.5,
                y: -0.5,
                w: 1.0,
                h: 1.0,
            };
            let expected = r;
            r.rotate(PI * 2.0);
            assert_relative_eq!(r, expected);
        }
        {
            let mut r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 2.0,
            };
            r.rotate(PI * 0.5);
            let expected = Rect {
                x: -2.0,
                y: 0.0,
                w: 2.0,
                h: 1.0,
            };
            assert_relative_eq!(r, expected);
        }
        {
            let mut r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 2.0,
            };
            r.rotate(PI);
            let expected = Rect {
                x: -1.0,
                y: -2.0,
                w: 1.0,
                h: 2.0,
            };
            assert_relative_eq!(r, expected);
        }
        {
            let mut r = Rect {
                x: -0.5,
                y: -0.5,
                w: 1.0,
                h: 1.0,
            };
            r.rotate(PI * 0.5);
            let expected = Rect {
                x: -0.5,
                y: -0.5,
                w: 1.0,
                h: 1.0,
            };
            assert_relative_eq!(r, expected);
        }
        {
            let mut r = Rect {
                x: 1.0,
                y: 1.0,
                w: 0.5,
                h: 2.0,
            };
            r.rotate(PI * 0.5);
            let expected = Rect {
                x: -3.0,
                y: 1.0,
                w: 2.0,
                h: 0.5,
            };
            assert_relative_eq!(r, expected);
        }
    }
}
