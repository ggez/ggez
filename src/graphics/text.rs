//!

use super::Color;
use crate::{filesystem::Filesystem, GameResult};
use glyph_brush::ab_glyph;
use std::{borrow::Cow, io::Read, path::Path};

/// Font data that can be used to create a new font in [super::context::GraphicsContext].
#[derive(Debug)]
pub struct FontData {
    pub(crate) font: ab_glyph::FontArc,
}

impl FontData {
    /// Loads font data from a given path in the filesystem.
    #[allow(unused_results)]
    pub fn from_path(fs: &Filesystem, path: impl AsRef<Path>) -> GameResult<Self> {
        let mut bytes = vec![];
        fs.open(path)?.read_to_end(&mut bytes)?;
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

/// Parameters of a single piece of text, including font, color, size, and Z position.
#[derive(Debug, Clone)]
pub struct Text<'a> {
    /// The text itself.
    pub text: Cow<'a, str>,
    /// Font name of the text.
    pub font: Cow<'a, str>,
    /// Pixel size of text.
    pub size: f32,
    /// Color of text.
    pub color: Color,
}

impl<'a> Default for Text<'a> {
    fn default() -> Self {
        Text {
            text: "".into(),
            font: "".into(),
            size: 16.,
            color: Color::BLACK,
        }
    }
}

impl<'a> Text<'a> {
    /// Equivalent to `Text::default()`.
    pub fn new() -> Self {
        Text::default()
    }

    /// Sets the `text` field.
    pub fn text(self, text: impl Into<Cow<'a, str>>) -> Self {
        Text {
            text: text.into(),
            ..self
        }
    }

    /// Sets the `font` field.
    pub fn font(self, font: impl Into<Cow<'a, str>>) -> Self {
        Text {
            font: font.into(),
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

impl From<TextAlign> for glyph_brush::HorizontalAlign {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Begin => glyph_brush::HorizontalAlign::Left,
            TextAlign::Middle => glyph_brush::HorizontalAlign::Center,
            TextAlign::End => glyph_brush::HorizontalAlign::Right,
        }
    }
}

impl From<TextAlign> for glyph_brush::VerticalAlign {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Begin => glyph_brush::VerticalAlign::Top,
            TextAlign::Middle => glyph_brush::VerticalAlign::Center,
            TextAlign::End => glyph_brush::VerticalAlign::Bottom,
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
