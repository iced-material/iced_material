// SPDX-License-Identifier: LGPL-3.0-only

use super::hct::Hct;
use super::utils;

const TARGET_CHROMA: f64 = 48.0;
const WEIGHT_PROPORTION: f64 = 0.7;
const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
const WEIGHT_CHROMA_BELOW: f64 = 0.1;
const CUTOFF_CHROMA: f64 = 5.0;
const CUTOFF_EXCITED_PROPORTION: f64 = 0.01;
pub const FALLBACK_COLOR: u32 = 0xff4285f4;

pub fn score(
    colors_to_population: &[(u32, u32)],
    desired: usize,
    fallback: u32,
    filter: bool,
) -> Vec<u32> {
    let mut colors_hct = Vec::with_capacity(colors_to_population.len());
    let mut hue_population = [0i32; 360];
    let mut population_sum = 0.0;
    for &(argb, population) in colors_to_population {
        let hct = Hct::from_int(argb);
        colors_hct.push(hct);
        let hue = hct.hue().floor() as usize;
        hue_population[hue] += population as i32;
        population_sum += population as f64;
    }
    let mut hue_excited_proportions = [0.0f64; 360];
    for (hue, &population) in hue_population.iter().enumerate() {
        let proportion = population as f64 / population_sum;
        for i in (hue as i32 - 14)..(hue as i32 + 16) {
            hue_excited_proportions[utils::sanitize_degrees_int(i) as usize] += proportion;
        }
    }
    let mut scored: Vec<(Hct, f64)> = Vec::new();
    for hct in colors_hct {
        let hue = utils::sanitize_degrees_int(utils::js_round(hct.hue()) as i32);
        let proportion = hue_excited_proportions[hue as usize];
        if filter && (hct.chroma() < CUTOFF_CHROMA || proportion <= CUTOFF_EXCITED_PROPORTION) {
            continue;
        }
        let proportion_score = proportion * 100.0 * WEIGHT_PROPORTION;
        let chroma_weight = if hct.chroma() < TARGET_CHROMA {
            WEIGHT_CHROMA_BELOW
        } else {
            WEIGHT_CHROMA_ABOVE
        };
        let chroma_score = (hct.chroma() - TARGET_CHROMA) * chroma_weight;
        scored.push((hct, proportion_score + chroma_score));
    }
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut chosen: Vec<Hct> = Vec::new();
    for difference in (15..=90).rev() {
        chosen.clear();
        for (hct, _) in &scored {
            let duplicate = chosen
                .iter()
                .any(|c| utils::difference_degrees(hct.hue(), c.hue()) < difference as f64);
            if !duplicate {
                chosen.push(*hct);
            }
            if chosen.len() >= desired {
                break;
            }
        }
        if chosen.len() >= desired {
            break;
        }
    }
    if chosen.is_empty() {
        return vec![fallback];
    }
    chosen.iter().map(|hct| hct.to_int()).collect()
}
