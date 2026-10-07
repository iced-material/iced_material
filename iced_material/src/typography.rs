// SPDX-License-Identifier: LGPL-3.0-only

//! Material type scale.

use iced::Font;
use iced::font::{Family, Weight};

/// One role of the type scale. Sizes are in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeStyle {
    /// Font family and weight.
    pub font: Font,
    /// Font size.
    pub size: f32,
    /// Line height.
    pub line_height: f32,
    /// Letter spacing added after every character.
    pub tracking: f32,
}

impl TypeStyle {
    /// Returns the style with another weight.
    pub fn with_weight(self, weight: Weight) -> TypeStyle {
        TypeStyle {
            font: Font {
                weight,
                ..self.font
            },
            ..self
        }
    }
}

/// The fifteen baseline roles of the Material type scale.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeScale {
    pub display_large: TypeStyle,
    pub display_medium: TypeStyle,
    pub display_small: TypeStyle,
    pub headline_large: TypeStyle,
    pub headline_medium: TypeStyle,
    pub headline_small: TypeStyle,
    pub title_large: TypeStyle,
    pub title_medium: TypeStyle,
    pub title_small: TypeStyle,
    pub body_large: TypeStyle,
    pub body_medium: TypeStyle,
    pub body_small: TypeStyle,
    pub label_large: TypeStyle,
    pub label_medium: TypeStyle,
    pub label_small: TypeStyle,
    pub label_large_prominent: TypeStyle,
    pub label_medium_prominent: TypeStyle,
}

impl TypeScale {
    /// Builds the baseline scale with the given brand and plain font families.
    pub fn new(brand: &'static str, plain: &'static str) -> TypeScale {
        let style =
            |family: &'static str, weight: Weight, size: f32, line_height: f32, tracking: f32| {
                TypeStyle {
                    font: Font {
                        family: Family::Name(family),
                        weight,
                        ..Font::DEFAULT
                    },
                    size,
                    line_height,
                    tracking,
                }
            };
        let label_large = style(plain, Weight::Medium, 14.0, 20.0, 0.1);
        let label_medium = style(plain, Weight::Medium, 12.0, 16.0, 0.5);
        TypeScale {
            display_large: style(brand, Weight::Normal, 57.0, 64.0, -0.25),
            display_medium: style(brand, Weight::Normal, 45.0, 52.0, 0.0),
            display_small: style(brand, Weight::Normal, 36.0, 44.0, 0.0),
            headline_large: style(brand, Weight::Normal, 32.0, 40.0, 0.0),
            headline_medium: style(brand, Weight::Normal, 28.0, 36.0, 0.0),
            headline_small: style(brand, Weight::Normal, 24.0, 32.0, 0.0),
            title_large: style(brand, Weight::Normal, 22.0, 28.0, 0.0),
            title_medium: style(plain, Weight::Medium, 16.0, 24.0, 0.15),
            title_small: style(plain, Weight::Medium, 14.0, 20.0, 0.1),
            body_large: style(plain, Weight::Normal, 16.0, 24.0, 0.5),
            body_medium: style(plain, Weight::Normal, 14.0, 20.0, 0.25),
            body_small: style(plain, Weight::Normal, 12.0, 16.0, 0.4),
            label_large,
            label_medium,
            label_small: style(plain, Weight::Medium, 11.0, 16.0, 0.5),
            label_large_prominent: label_large.with_weight(Weight::Bold),
            label_medium_prominent: label_medium.with_weight(Weight::Bold),
        }
    }
}

impl Default for TypeScale {
    fn default() -> Self {
        TypeScale::new("Roboto", "Roboto")
    }
}
