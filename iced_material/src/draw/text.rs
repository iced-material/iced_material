// SPDX-License-Identifier: LGPL-3.0-only

//! Single line text with Material letter spacing.
//!
//! Iced does not expose letter spacing. A [`Label`] shapes the whole string
//! once and draws each grapheme at its shaped position plus the accumulated
//! tracking, so kerning is kept. Ligatures that span graphemes are lost.
//! Text outside the Latin, Greek and Cyrillic blocks is drawn without
//! tracking, because splitting it into graphemes breaks shaping.

use iced::advanced::text::{self, Paragraph as _, Renderer as _};
use iced::alignment::Vertical;
use iced::{Color, Pixels, Point, Rectangle, Renderer, Size};
use unicode_segmentation::UnicodeSegmentation;

use crate::typography::TypeStyle;

type Paragraph = <Renderer as text::Renderer>::Paragraph;

/// A shaped single line of text.
#[derive(Default)]
pub struct Label {
    content: String,
    style: Option<TypeStyle>,
    full: Paragraph,
    graphemes: Vec<(f32, Paragraph)>,
    size: Size,
}

fn paragraph(content: &str, style: &TypeStyle) -> Paragraph {
    Paragraph::with_text(text::Text {
        content,
        bounds: Size::INFINITE,
        size: Pixels(style.size),
        line_height: text::LineHeight::Absolute(Pixels(style.line_height)),
        font: style.font,
        align_x: text::Alignment::Left,
        align_y: Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
    })
}

fn splits_safely(content: &str) -> bool {
    content.chars().all(|c| {
        let c = c as u32;
        c < 0x0250 || (0x0370..0x0530).contains(&c) || (0x1E00..0x2070).contains(&c)
    })
}

impl Label {
    /// Reshapes the label when the content or style changed. Returns its size.
    pub fn update(&mut self, content: &str, style: TypeStyle) -> Size {
        if self.style == Some(style) && self.content == content {
            return self.size;
        }
        self.content = content.to_string();
        self.style = Some(style);
        self.full = paragraph(content, &style);
        self.graphemes.clear();
        let mut width = self.full.min_width();
        if style.tracking != 0.0 && splits_safely(content) {
            let count = content.graphemes(true).count();
            for (index, grapheme) in content.graphemes(true).enumerate() {
                let x = self.full.grapheme_position(0, index).map_or(0.0, |p| p.x);
                self.graphemes.push((
                    x + style.tracking * index as f32,
                    paragraph(grapheme, &style),
                ));
            }
            width += style.tracking * count as f32;
        }
        self.size = Size::new(width.max(0.0), style.line_height);
        self.size
    }

    /// Size of the label after the last [`Label::update`].
    pub fn size(&self) -> Size {
        self.size
    }

    /// Draws the label with its top left corner at `position`.
    pub fn draw(&self, renderer: &mut Renderer, position: Point, color: Color, clip: Rectangle) {
        if self.graphemes.is_empty() {
            renderer.fill_paragraph(&self.full, position, color, clip);
            return;
        }
        for (x, grapheme) in &self.graphemes {
            renderer.fill_paragraph(
                grapheme,
                Point::new(position.x + x, position.y),
                color,
                clip,
            );
        }
    }
}
