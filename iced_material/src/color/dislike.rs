// SPDX-License-Identifier: LGPL-3.0-only

use super::hct::Hct;
use super::utils::js_round;

pub fn is_disliked(hct: Hct) -> bool {
    let hue = js_round(hct.hue());
    let hue_passes = (90.0..=111.0).contains(&hue);
    let chroma_passes = js_round(hct.chroma()) > 16.0;
    let tone_passes = js_round(hct.tone()) < 65.0;
    hue_passes && chroma_passes && tone_passes
}

pub fn fix_if_disliked(hct: Hct) -> Hct {
    if is_disliked(hct) {
        Hct::from(hct.hue(), hct.chroma(), 70.0)
    } else {
        hct
    }
}
