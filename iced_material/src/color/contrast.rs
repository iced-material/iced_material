// SPDX-License-Identifier: LGPL-3.0-only

use super::utils;

pub fn ratio_of_tones(tone_a: f64, tone_b: f64) -> f64 {
    let tone_a = utils::clamp_double(0.0, 100.0, tone_a);
    let tone_b = utils::clamp_double(0.0, 100.0, tone_b);
    ratio_of_ys(utils::y_from_lstar(tone_a), utils::y_from_lstar(tone_b))
}

pub fn ratio_of_ys(y1: f64, y2: f64) -> f64 {
    let lighter = if y1 > y2 { y1 } else { y2 };
    let darker = if lighter == y2 { y1 } else { y2 };
    (lighter + 5.0) / (darker + 5.0)
}

pub fn lighter(tone: f64, ratio: f64) -> f64 {
    if !(0.0..=100.0).contains(&tone) {
        return -1.0;
    }
    let dark_y = utils::y_from_lstar(tone);
    let light_y = ratio * (dark_y + 5.0) - 5.0;
    let real_contrast = ratio_of_ys(light_y, dark_y);
    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > 0.04 {
        return -1.0;
    }
    let value = utils::lstar_from_y(light_y) + 0.4;
    if !(0.0..=100.0).contains(&value) {
        return -1.0;
    }
    value
}

pub fn darker(tone: f64, ratio: f64) -> f64 {
    if !(0.0..=100.0).contains(&tone) {
        return -1.0;
    }
    let light_y = utils::y_from_lstar(tone);
    let dark_y = (light_y + 5.0) / ratio - 5.0;
    let real_contrast = ratio_of_ys(light_y, dark_y);
    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > 0.04 {
        return -1.0;
    }
    let value = utils::lstar_from_y(dark_y) - 0.4;
    if !(0.0..=100.0).contains(&value) {
        return -1.0;
    }
    value
}

pub fn lighter_unsafe(tone: f64, ratio: f64) -> f64 {
    let safe = lighter(tone, ratio);
    if safe < 0.0 { 100.0 } else { safe }
}

pub fn darker_unsafe(tone: f64, ratio: f64) -> f64 {
    let safe = darker(tone, ratio);
    if safe < 0.0 { 0.0 } else { safe }
}
