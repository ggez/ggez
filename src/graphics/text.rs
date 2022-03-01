//!

use super::Color;
use crate::{context::Context, filesystem, GameResult};
use std::{io::Read, path::Path};
use wgpu_glyph::ab_glyph;

/// Font data that can be used to create a new font in [super::context::GraphicsContext].
#[derive(Debug)]
pub struct FontData {
    pub(crate) font: ab_glyph::FontArc,
}

impl FontData {
    /// Loads font data from a given path in the filesystem.
    #[allow(unused_results)]
    pub fn from_path(ctx: &Context, path: impl AsRef<Path>) -> GameResult<Self> {
        let mut bytes = vec![];
        filesystem::open(ctx, path)?.read_to_end(&mut bytes)?;
        Ok(FontData {
            font: ab_glyph::FontArc::try_from_vec(bytes)?,
        })
    }

    /// Loads font data from owned bytes.
    pub fn from_vec(data: Vec<u8>) -> GameResult<Self> {
        Ok(FontData {
            font: ab_glyph::FontArc::try_from_vec(data)?,
        })
    }

    /// Loads font data from static bytes.
    pub fn from_slice(data: &'static [u8]) -> GameResult<Self> {
        Ok(FontData {
            font: ab_glyph::FontArc::try_from_slice(data)?,
        })
    }
}

/// Parameters of a single piece of text.
#[derive(Debug, Clone, Copy)]
pub struct Text<'a> {
    /// The text itself.
    pub text: &'a str,
    /// Font name of the text.
    pub font: &'a str,
    /// Pixel size of text.
    pub size: f32,
    /// Color of text.
    pub color: Color,
    /// Optional Z position of text.
    pub z: Option<f32>,
}

impl<'a> Default for Text<'a> {
    fn default() -> Self {
        Text {
            text: "",
            font: "",
            size: 16.,
            color: Color::BLACK,
            z: None,
        }
    }
}

impl<'a> Text<'a> {
    /// Equivalent to `Text::default()`.
    pub fn new() -> Self {
        Text::default()
    }

    /// Sets the `text` field.
    pub fn text(self, text: &'a str) -> Self {
        Text {
            text: text.as_ref(),
            ..self
        }
    }

    /// Sets the `font` field.
    pub fn font(self, font: &'a str) -> Self {
        Text {
            font: font.as_ref(),
            ..self
        }
    }

    /// Sets the `size` field.
    pub fn size(self, size: f32) -> Self {
        Text { size, ..self }
    }

    /// Sets the `color` field.
    pub fn color(self, color: impl Into<Color>) -> Self {
        Text {
            color: color.into(),
            ..self
        }
    }

    /// Sets the `z` field.
    pub fn z(self, z: impl Into<Option<f32>>) -> Self {
        Text {
            z: z.into(),
            ..self
        }
    }
}

/// Describes text alignment along a single axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextAlign {
    /// Text is aligned to the beginning of the axis (left, top).
    Begin,
    /// Text is aligned to the center of the axis.
    Middle,
    /// Text is aligned to the end of the axis (right, bottom).
    End,
}

impl From<TextAlign> for wgpu_glyph::HorizontalAlign {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Begin => wgpu_glyph::HorizontalAlign::Left,
            TextAlign::Middle => wgpu_glyph::HorizontalAlign::Center,
            TextAlign::End => wgpu_glyph::HorizontalAlign::Right,
        }
    }
}

impl From<TextAlign> for wgpu_glyph::VerticalAlign {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Begin => wgpu_glyph::VerticalAlign::Top,
            TextAlign::Middle => wgpu_glyph::VerticalAlign::Center,
            TextAlign::End => wgpu_glyph::VerticalAlign::Bottom,
        }
    }
}

/// Describes text alignment along both axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextLayout {
    /// Text is layed out in a single line.
    SingleLine {
        /// Horizontal alignment.
        h_align: TextAlign,
        /// Vertical alignment.
        v_align: TextAlign,
    },
    /// Text wraps around the bounds.
    Wrap {
        /// Horizontal alignment.
        h_align: TextAlign,
        /// Vertical alignment.
        v_align: TextAlign,
    },
}

impl TextLayout {
    /// Text on a single line aligned to the top-left.
    pub fn tl_single_line() -> Self {
        TextLayout::SingleLine {
            h_align: TextAlign::Begin,
            v_align: TextAlign::Begin,
        }
    }
}
