// SPDX-License-Identifier: LGPL-3.0-only

use super::hct::Hct;
use super::utils::js_round;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TonalPalette {
    pub hue: f64,
    pub chroma: f64,
    pub key_color: Hct,
}

impl TonalPalette {
    pub fn from_hct(hct: Hct) -> TonalPalette {
        TonalPalette {
            hue: hct.hue(),
            chroma: hct.chroma(),
            key_color: hct,
        }
    }

    pub fn from_hue_and_chroma(hue: f64, chroma: f64) -> TonalPalette {
        TonalPalette {
            hue,
            chroma,
            key_color: key_color(hue, chroma),
        }
    }

    pub fn tone(&self, tone: f64) -> u32 {
        tone_of(self.hue, self.chroma, tone)
    }

    pub fn get_hct(&self, tone: f64) -> Hct {
        Hct::from_int(self.tone(tone))
    }
}

pub fn tone_of(hue: f64, chroma: f64, tone: f64) -> u32 {
    if tone == 99.0 && Hct::is_yellow(hue) {
        average_argb(tone_of(hue, chroma, 98.0), tone_of(hue, chroma, 100.0))
    } else {
        Hct::from(hue, chroma, tone).to_int()
    }
}

fn average_argb(argb1: u32, argb2: u32) -> u32 {
    let channel = |shift: u32| {
        let a = (argb1 >> shift) & 0xff;
        let b = (argb2 >> shift) & 0xff;
        js_round((a + b) as f64 / 2.0) as u32 & 255
    };
    0xFF00_0000 | channel(16) << 16 | channel(8) << 8 | channel(0)
}

fn key_color(hue: f64, requested_chroma: f64) -> Hct {
    let pivot_tone = 50;
    let tone_step_size = 1;
    let epsilon = 0.01;
    let max_chroma = |tone: i32| Hct::from(hue, 200.0, tone as f64).chroma();
    let mut lower_tone = 0;
    let mut upper_tone = 100;
    while lower_tone < upper_tone {
        let mid_tone = (lower_tone + upper_tone) / 2;
        let mid_chroma = max_chroma(mid_tone);
        let is_ascending = mid_chroma < max_chroma(mid_tone + tone_step_size);
        let sufficient_chroma = mid_chroma >= requested_chroma - epsilon;
        if sufficient_chroma {
            if (lower_tone - pivot_tone).abs() < (upper_tone - pivot_tone).abs() {
                upper_tone = mid_tone;
            } else {
                if lower_tone == mid_tone {
                    return Hct::from(hue, requested_chroma, lower_tone as f64);
                }
                lower_tone = mid_tone;
            }
        } else if is_ascending {
            lower_tone = mid_tone + tone_step_size;
        } else {
            upper_tone = mid_tone;
        }
    }
    Hct::from(hue, requested_chroma, lower_tone as f64)
}
