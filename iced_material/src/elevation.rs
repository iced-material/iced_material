// SPDX-License-Identifier: LGPL-3.0-only

//! Material elevation levels and their shadows.

use std::time::Duration;

/// One box shadow layer in logical pixels, without color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowLayer {
    /// Vertical offset.
    pub y: f32,
    /// Blur radius as defined by CSS `box-shadow`.
    pub blur: f32,
    /// Spread distance.
    pub spread: f32,
    /// Opacity applied to the shadow color.
    pub opacity: f32,
}

/// Elevation tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Elevation {
    /// Elevation of levels 0 to 5 in dp.
    pub levels: [f32; 6],
    /// Opacity of the key shadow layer.
    pub key_opacity: f32,
    /// Opacity of the ambient shadow layer.
    pub ambient_opacity: f32,
    /// Duration of elevation changes between interaction states, as in Material web.
    pub transition: Duration,
}

impl Default for Elevation {
    fn default() -> Self {
        Elevation {
            levels: [0.0, 1.0, 3.0, 6.0, 8.0, 12.0],
            key_opacity: 0.3,
            ambient_opacity: 0.15,
            transition: Duration::from_millis(280),
        }
    }
}

fn clamp(value: f32, max: f32) -> f32 {
    value.clamp(0.0, max)
}

impl Elevation {
    /// Returns the key and ambient shadow layers for a level from 0 to 5.
    ///
    /// Fractional levels interpolate linearly, which matches how Material web
    /// transitions shadows between levels.
    pub fn shadows(&self, level: f32) -> [ShadowLayer; 2] {
        let l = level;
        let key = ShadowLayer {
            y: clamp(l, 1.0) + clamp(l - 3.0, 1.0) + 2.0 * clamp(l - 4.0, 1.0),
            blur: 2.0 * clamp(l, 1.0) + clamp(l - 2.0, 1.0) + clamp(l - 4.0, 1.0),
            spread: 0.0,
            opacity: self.key_opacity,
        };
        let ambient = ShadowLayer {
            y: clamp(l, 1.0) + clamp(l - 1.0, 1.0) + 2.0 * clamp(l - 2.0, 3.0),
            blur: 3.0 * clamp(l, 2.0) + 2.0 * clamp(l - 2.0, 3.0),
            spread: clamp(l, 4.0) + 2.0 * clamp(l - 4.0, 1.0),
            opacity: self.ambient_opacity,
        };
        [key, ambient]
    }
}
