pub struct Point {
    pub x: f32,
    pub y: f32
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x:x, y:y, w:w, h:h }
    }

    pub fn new_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect { x:x as f32, y:y as f32, w:w as f32, h:h as f32 }
    }
}

pub struct Color(pub f32, pub f32, pub f32, pub f32);

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color(r, g, b, a)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8, u8)) -> Self {
        let (r, g, b, a) = val;
        let rf = (r as f32) / 255.0;
        let gf = (g as f32) / 255.0;
        let bf = (b as f32) / 255.0;
        let af = (a as f32) / 255.0;
        Color(rf, gf, bf, af)
    }
}

pub enum BlendMode {
    Dummy
}
