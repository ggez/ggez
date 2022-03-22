//!

use super::{gpu::text::Extra, Color, Rect};
use crate::{filesystem::Filesystem, GameError, GameResult};
use glyph_brush::{ab_glyph, FontId};
use std::{collections::HashMap, io::Read, path::Path};

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
pub struct Text {
    /// The text itself.
    pub text: String,
    /// Font name of the text.
    pub font: String,
    /// Pixel size of text.
    pub size: f32,
    /// Color of text.
    pub color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            text: "".into(),
            font: "LiberationMono-Regular".into(),
            size: 16.,
            color: Color::WHITE,
        }
    }
}

impl Text {
    /// Equivalent to `Text::default()`.
    pub fn new() -> Self {
        Text::default()
    }

    /// Sets the `text` field.
    pub fn text(self, text: impl Into<String>) -> Self {
        Text {
            text: text.into(),
            ..self
        }
    }

    /// Sets the `font` field.
    pub fn font(self, font: impl Into<String>) -> Self {
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

    /// Text wrapped and aligned to the top-left.
    pub fn tl_wrap() -> Self {
        TextLayout::Wrap {
            h_align: TextAlign::Begin,
            v_align: TextAlign::Begin,
        }
    }

    /// Returns the horizontal alignment, regardless of wrapping behaviour.
    pub fn h_align(&self) -> TextAlign {
        match self {
            TextLayout::SingleLine { h_align, .. } | TextLayout::Wrap { h_align, .. } => *h_align,
        }
    }

    /// Returns the vertical alignment, regardless of wrapping behaviour.
    pub fn v_align(&self) -> TextAlign {
        match self {
            TextLayout::SingleLine { v_align, .. } | TextLayout::Wrap { v_align, .. } => *v_align,
        }
    }
}

pub(crate) fn text_to_section<'a>(
    fonts: &HashMap<String, FontId>,
    text: &'a [Text],
    mut rect: Rect,
    rotation: f32,
    layout: TextLayout,
) -> GameResult<glyph_brush::Section<'a, Extra>> {
    let orect = rect;

    match layout.h_align() {
        TextAlign::Begin => {}
        TextAlign::Middle => rect.x += rect.w / 2.,
        TextAlign::End => rect.x += rect.w,
    }

    match layout.v_align() {
        TextAlign::Begin => {}
        TextAlign::Middle => rect.y += rect.h / 2.,
        TextAlign::End => rect.y += rect.h,
    }

    Ok(glyph_brush::Section {
        screen_position: (rect.x, rect.y),
        bounds: (rect.w, rect.h),
        layout: match layout {
            TextLayout::SingleLine { h_align, v_align } => {
                glyph_brush::Layout::default_single_line()
                    .h_align(h_align.into())
                    .v_align(v_align.into())
            }
            TextLayout::Wrap { h_align, v_align } => glyph_brush::Layout::default_wrap()
                .h_align(h_align.into())
                .v_align(v_align.into()),
        },
        text: text
            .iter()
            .map(|text| {
                Ok(glyph_brush::Text {
                    text: &text.text,
                    scale: text.size.into(),
                    font_id: *fonts
                        .get(&text.font)
                        .ok_or_else(|| GameError::FontSelectError(text.font.to_string()))?,
                    extra: Extra {
                        color: text.color.into(),
                        origin: orect.point().into(),
                        rotation,
                    },
                })
            })
            .collect::<GameResult<Vec<_>>>()?,
    })
}
