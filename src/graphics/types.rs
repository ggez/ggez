pub use nalgebra as na;

/// A 2 dimensional point representing a location
pub type Point2 = na::Point2<f32>;
/// A 2 dimensional vector representing an offset of a location
pub type Vector2 = na::Vector2<f32>;
/// A 4 dimensional matrix representing an arbitrary 3d transformation
pub type Matrix4 = na::Matrix4<f32>;

/// Turns a point into an array of floats
pub fn pt2arr(pt: Point2) -> [f32; 2] {
    [pt.x, pt.y]
}

/// Turns an array of floats into a point.
pub fn arr2pt(pt: [f32; 2]) -> Point2 {
    Point2::new(pt[0], pt[1])
}

/// A simple 2D rectangle.
///
/// The ggez convention is that `x` and `y` are the **center** of the rectangle,
/// with `width` and `height` being the total width and height, because this
/// is generally also how OpenGL tends to think about the world.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Rect {
    /// X coordinate of the center of the rect.
    pub x: f32,
    /// Y coordinate of the center of the rect.
    pub y: f32,
    /// Total width of the rect
    pub w: f32,
    /// Total height of the rect.
    pub h: f32,
}

impl Rect {
    /// Create a new rect.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect {
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }

    /// Creates a new rect a la Love2D's love.graphics.newQuad,
    /// as a fraction of the reference rect's size.
    pub fn fraction(x: f32, y: f32, w: f32, h: f32, reference: &Rect) -> Rect {
        Rect {
            x: x / reference.w,
            y: y / reference.h,
            w: w / reference.w,
            h: h / reference.h,
        }
    }

    /// Create a new rect from i32 coordinates.
    pub fn new_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        }
    }

    /// Create a new `Rect` with all values zero.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Creates a new `Rect` at 0,0 with width and height 1.
    pub fn one() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Gets the `Rect`'s x and y coordinates as a `Point2`.
    pub fn point(&self) -> Point2 {
        Point2::new(self.x, self.y)
    }

    /// Returns the left edge of the `Rect`
    pub fn left(&self) -> f32 {
        self.x - (self.w / 2.0)
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> f32 {
        self.x + (self.w / 2.0)
    }

    /// Returns the top edge of the `Rect`
    pub fn top(&self) -> f32 {
        self.y + (self.h / 2.0)
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> f32 {
        self.y - (self.h / 2.0)
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains(&self, point: &Point2) -> bool {
        point.x >= self.left() && point.x <= self.right() && point.y >= self.bottom() &&
        point.y <= self.top()
    }

    /// Checks whether the `Rect` overlaps another `Rect`
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() < other.right() && self.right() > other.left() &&
        self.top() > other.bottom() && self.bottom() < other.top()
    }

    /// Translates the `Rect` by an offset of (x, y)
    pub fn translate(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    /// Moves the `Rect`'s center to (x, y)
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    /// Scales the `Rect` about its center by a factor of (sx, sy)
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.w *= sx;
        self.h *= sy;
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


/// A RGBA color.
#[derive(Copy, Clone, PartialEq, Debug)]
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
    /// Create a new Color from components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8, u8)) -> Self {
        let (r, g, b, a) = val;
        let rf = (r as f32) / 255.0;
        let gf = (g as f32) / 255.0;
        let bf = (b as f32) / 255.0;
        let af = (a as f32) / 255.0;
        Color::new(rf, gf, bf, af)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8)) -> Self {
        let (r, g, b) = val;
        Color::from((r, g, b, 255))
    }
}

impl From<[f32; 4]> for Color {
    fn from(val: [f32; 4]) -> Self {
        Color::new(val[0], val[1], val[2], val[3])
    }
}

impl From<Color> for (u8, u8, u8, u8) {
    fn from(color: Color) -> Self {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        (r, g, b, a)
    }
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        let (r, g, b, a) = color.into();
        [r, g, b, a]
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}


impl From<Color> for (u8, u8, u8) {
    fn from(color: Color) -> Self {
        let (r, g, b, _) = color.into();
        (r, g, b)
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        let (r, g, b, a): (u8, u8, u8, u8) = color.into();
        let rp = (r as u32) << 24;
        let gp = (g as u32) << 16;
        let bp = (b as u32) << 8;
        let ap = a as u32;
        (rp | gp | bp | ap)
    }
}

/// Specifies whether a shape should be drawn
/// filled or as an outline.
#[derive(Debug, Copy, Clone)]
pub enum DrawMode {
    /// A stroked line with the given width
    Line(f32),
    /// A filled shape.
    Fill,
}

/// Specifies what blending method to use when scaling up/down images.
#[derive(Debug, Copy, Clone)]
pub enum FilterMode {
    /// Use linear interpolation
    Linear,
    /// Use nearest-neighbor interpolation
    Nearest,
}

use gfx::texture;
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
pub type WrapMode = texture::WrapMode;



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_color_conversions() {
        let white = Color::new(1.0, 1.0, 1.0, 1.0);
        let w1 = Color::from((255, 255, 255, 255));
        assert_eq!(white, w1);
        let w2: u32 = white.into();
        assert_eq!(w2, 0xFFFFFFFF);

        let grey = Color::new(0.5019608, 0.5019608, 0.5019608, 1.0);
        let g1 = Color::from((128, 128, 128, 255));
        assert_eq!(grey, g1);
        let g2: u32 = grey.into();
        assert_eq!(g2, 0x808080FF);

        let black = Color::new(0.0, 0.0, 0.0, 1.0);
        let b1 = Color::from((0, 0, 0, 255));
        assert_eq!(black, b1);
        let b2: u32 = black.into();
        assert_eq!(b2, 0x000000FF);
    }

    #[test]
    fn test_rect_scaling() {
        let r1 = Rect::new(0.0, 0.0, 128.0, 128.0);
        let r2 = Rect::fraction(0.0, 0.0, 32.0, 32.0, &r1);
        assert_eq!(r2, Rect::new(0.0, 0.0, 0.25, 0.25));


        let r2 = Rect::fraction(32.0, 32.0, 32.0, 32.0, &r1);
        assert_eq!(r2, Rect::new(0.25, 0.25, 0.25, 0.25));
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0.0, 0.0, 128.0, 128.0);
        let p = Point2::new(0.0, 0.0);
        assert!(r.contains(&p));

        let p = Point2::new(500.0, 0.0);
        assert!(!r.contains(&p));
    }

    #[test]
    fn test_rect_overlaps() {
        let r1 = Rect::new(0.0, 0.0, 128.0, 128.0);
        let r2 = Rect::new(0.0, 0.0, 64.0, 64.0);
        assert!(r1.overlaps(&r2));

        let r2 = Rect::new(100.0, 0.0, 128.0, 128.0);
        assert!(r1.overlaps(&r2));

        let r2 = Rect::new(500.0, 0.0, 64.0, 64.0);
        assert!(!r1.overlaps(&r2));
    }

    #[test]
    fn test_rect_transform() {
        let mut r1 = Rect::new(0.0, 0.0, 64.0, 64.0);
        let r2 = Rect::new(64.0, 64.0, 64.0, 64.0);
        r1.translate(64.0, 64.0);
        assert!(r1 == r2);

        let mut r1 = Rect::new(0.0, 0.0, 64.0, 64.0);
        let r2 = Rect::new(0.0, 0.0, 128.0, 128.0);
        r1.scale(2.0, 2.0);
        assert!(r1 == r2);

        let mut r1 = Rect::new(32.0, 32.0, 64.0, 64.0);
        let r2 = Rect::new(64.0, 64.0, 64.0, 64.0);
        r1.move_to(64.0, 64.0);
        assert!(r1 == r2);
    }
}
