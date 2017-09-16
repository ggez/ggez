use sdl2;

pub use nalgebra as na;

pub type Point = na::Point2<f32>;
pub type Vector = na::Vector2<f32>;
pub type Matrix4 = na::Matrix4<f32>;

pub fn pt2arr(pt: Point) -> [f32;2] {
    [pt.x, pt.y]
    // pt.into()
}


pub fn arr2pt(pt: [f32;2]) -> Point {
    Point::new(pt[0], pt[1])
}

// pub type Point = na::Point2<f32>;

// /// A simple 2D point.
// #[derive(Copy, Clone, PartialEq, Debug, Default)]
// pub struct Point {
//     pub x: f32,
//     pub y: f32,
// }

// impl Point {
//     pub fn new(x: f32, y: f32) -> Self {
//         Point { x: x, y: y }
//     }

//     /// Creates a new `Point` at coordinates 0,0.
//     pub fn zero() -> Self {
//         Self::new(0.0, 0.0)
//     }
// }


// impl From<Point> for [f32; 2] {
//     fn from(p: Point) -> [f32; 2] {
//         [p.x, p.y]
//     }
// }


// impl From<[f32; 2]> for Point {
//     fn from(p: [f32; 2]) -> Point {
//         Point::new(p[0], p[1])
//     }
// }

// #[cfg(feature = "mint-exports")]
// extern crate mint;
// #[cfg(feature = "mint-exports")]
// impl From<mint::Point2<f32>> for Point {
//     fn from(p: mint::Point2<f32>) -> Point {
//         Point::new(p.x, p.y)
//     }
// }

// #[cfg(feature = "mint-exports")]
// impl From<Point> for mint::Point2<f32> {
//     fn from(p: Point) -> mint::Point2<f32> {
//         mint::Point2 { x: p.x, y: p.y }
//     }
// }


/// A simple 2D rectangle.
///
/// The ggez convention is that `x` and `y` are the **center** of the rectangle,
/// with `width` and `height` being the total width and height, because this
/// is generally also how OpenGL tends to think about the world.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect {
            x: x,
            y: y,
            w: w,
            h: h,
        }
    }

    /// Ceates a new rect a la Love2D's love.graphics.newQuad,
    /// as a fraction of the reference rect's size.
    pub fn fraction(x: f32, y: f32, w: f32, h: f32, reference: &Rect) -> Rect {
        Rect {
            x: x / reference.w,
            y: y / reference.h,
            w: w / reference.w,
            h: h / reference.h,
        }
    }

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

    /// Gets the `Rect`'s x and y coordinates as a `Point`.
    pub fn point(&self) -> Point {
        Point::new(self.x, self.y)
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
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};


pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

impl Color {
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
    Line,
    Fill,
}

/// Specifies what blending method to use when scaling up/down images.
#[derive(Debug, Copy, Clone)]
pub enum FilterMode {
    Linear,
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


pub type FullscreenType = sdl2::video::FullscreenType;

/// A builder structure containing flags for defining window settings.
#[derive(Debug, Copy, Clone)]
pub struct WindowMode {
    pub borderless: bool,
    pub fullscreen_type: FullscreenType,
    pub vsync: bool,
    /// Minimum dimensions for resizable windows; (0, 0) means no limit
    pub min_dimensions: (u32, u32),
    /// Maximum dimensions for resizable windows; (0, 0) means no limit
    pub max_dimensions: (u32, u32),
}

impl Default for WindowMode {
    fn default() -> Self {
        Self {
            borderless: false,
            fullscreen_type: sdl2::video::FullscreenType::Off,
            vsync: true,
            min_dimensions: (0, 0),
            max_dimensions: (0, 0),
        }
    }
}

impl WindowMode {
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.borderless = borderless;
        self
    }

    pub fn fullscreen_type(mut self, fullscreen_type: FullscreenType) -> Self {
        self.fullscreen_type = fullscreen_type;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn min_dimensions(mut self, width: u32, height: u32) -> Self {
        self.min_dimensions = (width, height);
        self
    }

    pub fn max_dimensions(mut self, width: u32, height: u32) -> Self {
        self.max_dimensions = (width, height);
        self
    }
}


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
}
