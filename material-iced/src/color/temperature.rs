// SPDX-License-Identifier: LGPL-3.0-only

use std::f64::consts::PI;

use super::hct::Hct;
use super::utils::{self, js_round};

pub struct TemperatureCache {
    input: Hct,
    hcts_by_hue: Vec<Hct>,
    coldest: Hct,
    warmest: Hct,
}

impl TemperatureCache {
    pub fn new(input: Hct) -> TemperatureCache {
        let hcts_by_hue: Vec<Hct> = (0..=360)
            .map(|hue| Hct::from(hue as f64, input.chroma(), input.tone()))
            .collect();
        let mut by_temp: Vec<(f64, Hct)> = hcts_by_hue
            .iter()
            .chain(std::iter::once(&input))
            .map(|hct| (raw_temperature(*hct), *hct))
            .collect();
        by_temp.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        TemperatureCache {
            input,
            hcts_by_hue,
            coldest: by_temp[0].1,
            warmest: by_temp[by_temp.len() - 1].1,
        }
    }

    pub fn analogous(&self, count: i32, divisions: i32) -> Vec<Hct> {
        let start_hue = js_round(self.input.hue()) as i32;
        let start_hct = self.hcts_by_hue[start_hue as usize];
        let mut last_temp = self.relative_temperature(start_hct);
        let mut all_colors = vec![start_hct];
        let mut absolute_total_temp_delta = 0.0;
        for i in 0..360 {
            let hue = utils::sanitize_degrees_int(start_hue + i);
            let temp = self.relative_temperature(self.hcts_by_hue[hue as usize]);
            absolute_total_temp_delta += (temp - last_temp).abs();
            last_temp = temp;
        }
        let mut hue_addend = 1;
        let temp_step = absolute_total_temp_delta / divisions as f64;
        let mut total_temp_delta = 0.0;
        last_temp = self.relative_temperature(start_hct);
        while (all_colors.len() as i32) < divisions {
            let hue = utils::sanitize_degrees_int(start_hue + hue_addend);
            let hct = self.hcts_by_hue[hue as usize];
            let temp = self.relative_temperature(hct);
            total_temp_delta += (temp - last_temp).abs();
            let desired = all_colors.len() as f64 * temp_step;
            let mut index_satisfied = total_temp_delta >= desired;
            let mut index_addend = 1;
            while index_satisfied && (all_colors.len() as i32) < divisions {
                all_colors.push(hct);
                let desired = (all_colors.len() as i32 + index_addend) as f64 * temp_step;
                index_satisfied = total_temp_delta >= desired;
                index_addend += 1;
            }
            last_temp = temp;
            hue_addend += 1;
            if hue_addend > 360 {
                while (all_colors.len() as i32) < divisions {
                    all_colors.push(hct);
                }
                break;
            }
        }
        let len = all_colors.len() as i32;
        let mut answers = vec![self.input];
        let increase_hue_count = (count - 1) / 2;
        for i in 1..=increase_hue_count {
            let mut index = -i;
            while index < 0 {
                index += len;
            }
            if index >= len {
                index %= len;
            }
            answers.insert(0, all_colors[index as usize]);
        }
        let decrease_hue_count = count - increase_hue_count - 1;
        for i in 1..=decrease_hue_count {
            let mut index = i;
            if index >= len {
                index %= len;
            }
            answers.push(all_colors[index as usize]);
        }
        answers
    }

    pub fn complement(&self) -> Hct {
        let coldest_hue = self.coldest.hue();
        let coldest_temp = raw_temperature(self.coldest);
        let warmest_hue = self.warmest.hue();
        let warmest_temp = raw_temperature(self.warmest);
        let range = warmest_temp - coldest_temp;
        let start_is_coldest_to_warmest = is_between(self.input.hue(), coldest_hue, warmest_hue);
        let start_hue = if start_is_coldest_to_warmest {
            warmest_hue
        } else {
            coldest_hue
        };
        let end_hue = if start_is_coldest_to_warmest {
            coldest_hue
        } else {
            warmest_hue
        };
        let mut smallest_error = 1000.0;
        let mut answer = self.hcts_by_hue[js_round(self.input.hue()) as usize];
        let complement_relative_temp = 1.0 - self.relative_temperature(self.input);
        let mut hue_addend = 0.0;
        while hue_addend <= 360.0 {
            let hue = utils::sanitize_degrees_double(start_hue + hue_addend);
            hue_addend += 1.0;
            if !is_between(hue, start_hue, end_hue) {
                continue;
            }
            let possible = self.hcts_by_hue[js_round(hue) as usize];
            let relative = (raw_temperature(possible) - coldest_temp) / range;
            let error = (complement_relative_temp - relative).abs();
            if error < smallest_error {
                smallest_error = error;
                answer = possible;
            }
        }
        answer
    }

    pub fn relative_temperature(&self, hct: Hct) -> f64 {
        let coldest = raw_temperature(self.coldest);
        let range = raw_temperature(self.warmest) - coldest;
        if range == 0.0 {
            return 0.5;
        }
        (raw_temperature(hct) - coldest) / range
    }
}

fn is_between(angle: f64, a: f64, b: f64) -> bool {
    if a < b {
        a <= angle && angle <= b
    } else {
        a <= angle || angle <= b
    }
}

pub fn raw_temperature(color: Hct) -> f64 {
    let lab = utils::lab_from_argb(color.to_int());
    let hue = utils::sanitize_degrees_double(lab[2].atan2(lab[1]) * 180.0 / PI);
    let chroma = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
    -0.5 + 0.02
        * chroma.powf(1.07)
        * (utils::sanitize_degrees_double(hue - 50.0) * PI / 180.0).cos()
}
