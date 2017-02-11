#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
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

    pub fn new_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        }
    }
}

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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BlendMode {
    Dummy,
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
}
