// SPDX-License-Identifier: LGPL-3.0-only

use iced::Color;

use super::harmonize;
use super::hct::Hct;
use super::palette::TonalPalette;
use super::scheme::{argb, color};

/// Seed of the success color: `Colors.green` of Flutter, `#4CAF50`.
pub const SUCCESS_SEED: Color = Color::from_rgb8(0x4C, 0xAF, 0x50);

/// Seed of the warning color: `Colors.amber` of Flutter, `#FFC107`.
pub const WARNING_SEED: Color = Color::from_rgb8(0xFF, 0xC1, 0x07);

/// The four roles of a custom color, as defined by Material Color Utilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomColor {
    /// Accent color.
    pub color: Color,
    /// Content on top of `color`.
    pub on_color: Color,
    /// Container color.
    pub color_container: Color,
    /// Content on top of `color_container`.
    pub on_color_container: Color,
}

impl CustomColor {
    /// Builds the roles of a custom color, optionally harmonized toward `source`.
    pub fn new(value: Color, source: Color, blend: bool, dark: bool) -> CustomColor {
        let value = if blend {
            harmonize(value, source)
        } else {
            value
        };
        let hct = Hct::from_int(argb(value));
        let palette = TonalPalette::from_hue_and_chroma(hct.hue(), hct.chroma().max(48.0));
        let tones = if dark {
            [80.0, 20.0, 30.0, 90.0]
        } else {
            [40.0, 100.0, 90.0, 10.0]
        };
        CustomColor {
            color: color(palette.tone(tones[0])),
            on_color: color(palette.tone(tones[1])),
            color_container: color(palette.tone(tones[2])),
            on_color_container: color(palette.tone(tones[3])),
        }
    }

    pub(crate) fn mix(
        &self,
        other: &CustomColor,
        amount: f32,
        mix: fn(Color, Color, f32) -> Color,
    ) -> CustomColor {
        CustomColor {
            color: mix(self.color, other.color, amount),
            on_color: mix(self.on_color, other.on_color, amount),
            color_container: mix(self.color_container, other.color_container, amount),
            on_color_container: mix(self.on_color_container, other.on_color_container, amount),
        }
    }
}
