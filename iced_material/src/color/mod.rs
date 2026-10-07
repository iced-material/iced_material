// SPDX-License-Identifier: LGPL-3.0-only

//! Color scheme generation based on Material Color Utilities.

mod blend;
mod cam16;
mod contrast;
mod custom;
mod dislike;
mod dynamic;
mod hct;
mod palette;
mod quantize;
mod scheme;
mod score;
mod spec_2021;
mod spec_2025;
mod temperature;
mod utils;

pub mod matugen;

use iced::Color;

pub use custom::{CustomColor, SUCCESS_SEED, WARNING_SEED};
pub use dynamic::{Role, SpecVersion, Variant};
pub use scheme::{ColorScheme, SchemeOptions};

use scheme::{argb, color};

/// Picks a source color from image pixels the way Material does for wallpapers.
///
/// `rgba` holds 8-bit RGBA pixels. Pixels that are not fully opaque are skipped.
/// Returns Google Blue `#4285F4` when no suitable color is found.
pub fn source_from_image(rgba: &[u8]) -> Color {
    let pixels: Vec<u32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] == 255)
        .map(|p| utils::argb_from_rgb(p[0] as u32, p[1] as u32, p[2] as u32))
        .collect();
    color(ranked_sources(&pixels, 4)[0])
}

fn ranked_sources(pixels: &[u32], desired: usize) -> Vec<u32> {
    let quantized = quantize::celebi(pixels, 128);
    score::score(&quantized, desired, score::FALLBACK_COLOR, true)
}

/// Shifts the hue of `design` toward `source` by at most 15 degrees.
pub fn harmonize(design: Color, source: Color) -> Color {
    color(blend::harmonize(argb(design), argb(source)))
}

/// WCAG contrast ratio between two opaque colors.
pub fn contrast_ratio(a: Color, b: Color) -> f64 {
    contrast::ratio_of_ys(
        utils::xyz_from_argb(argb(a))[1],
        utils::xyz_from_argb(argb(b))[1],
    )
}

#[cfg(test)]
mod tests;
