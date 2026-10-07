// SPDX-License-Identifier: LGPL-3.0-only

use super::hct::Hct;
use super::utils;

pub fn harmonize(design_color: u32, source_color: u32) -> u32 {
    let from = Hct::from_int(design_color);
    let to = Hct::from_int(source_color);
    let difference = utils::difference_degrees(from.hue(), to.hue());
    let rotation = (difference * 0.5).min(15.0);
    let output_hue = utils::sanitize_degrees_double(
        from.hue() + rotation * utils::rotation_direction(from.hue(), to.hue()),
    );
    Hct::from(output_hue, from.chroma(), from.tone()).to_int()
}
