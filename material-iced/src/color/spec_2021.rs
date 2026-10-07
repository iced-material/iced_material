// SPDX-License-Identifier: LGPL-3.0-only

use super::dislike;
use super::dynamic::{
    self, Constraint, ContrastCurve, Def, DynamicScheme, Polarity, Role, ToneDeltaPair, Variant,
    foreground_tone, highest_surface, rotated_hue,
};
use super::hct::Hct;
use super::palette::TonalPalette;
use super::temperature::TemperatureCache;
use super::utils;

pub fn palettes(variant: Variant, source: Hct) -> [TonalPalette; 6] {
    [
        primary_palette(variant, source),
        secondary_palette(variant, source),
        tertiary_palette(variant, source),
        neutral_palette(variant, source),
        neutral_variant_palette(variant, source),
        TonalPalette::from_hue_and_chroma(25.0, 84.0),
    ]
}

fn from(hue: f64, chroma: f64) -> TonalPalette {
    TonalPalette::from_hue_and_chroma(hue, chroma)
}

pub fn primary_palette(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => from(hue, source.chroma()),
        Variant::FruitSalad => from(utils::sanitize_degrees_double(hue - 50.0), 48.0),
        Variant::Monochrome => from(hue, 0.0),
        Variant::Neutral => from(hue, 12.0),
        Variant::Rainbow => from(hue, 48.0),
        Variant::TonalSpot => from(hue, 36.0),
        Variant::Expressive => from(utils::sanitize_degrees_double(hue + 240.0), 40.0),
        Variant::Vibrant => from(hue, 200.0),
    }
}

pub fn secondary_palette(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => {
            from(hue, (source.chroma() - 32.0).max(source.chroma() * 0.5))
        }
        Variant::FruitSalad => from(utils::sanitize_degrees_double(hue - 50.0), 36.0),
        Variant::Monochrome => from(hue, 0.0),
        Variant::Neutral => from(hue, 8.0),
        Variant::Rainbow | Variant::TonalSpot => from(hue, 16.0),
        Variant::Expressive => from(
            rotated_hue(
                source,
                &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                &[45.0, 95.0, 45.0, 20.0, 45.0, 90.0, 45.0, 45.0, 45.0],
            ),
            24.0,
        ),
        Variant::Vibrant => from(
            rotated_hue(
                source,
                &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                &[18.0, 15.0, 10.0, 12.0, 15.0, 18.0, 15.0, 12.0, 12.0],
            ),
            24.0,
        ),
    }
}

pub fn tertiary_palette(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content => TonalPalette::from_hct(dislike::fix_if_disliked(
            TemperatureCache::new(source).analogous(3, 6)[2],
        )),
        Variant::Fidelity => TonalPalette::from_hct(dislike::fix_if_disliked(
            TemperatureCache::new(source).complement(),
        )),
        Variant::FruitSalad => from(hue, 36.0),
        Variant::Monochrome => from(hue, 0.0),
        Variant::Neutral => from(hue, 16.0),
        Variant::Rainbow | Variant::TonalSpot => {
            from(utils::sanitize_degrees_double(hue + 60.0), 24.0)
        }
        Variant::Expressive => from(
            rotated_hue(
                source,
                &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                &[120.0, 120.0, 20.0, 45.0, 20.0, 15.0, 20.0, 120.0, 120.0],
            ),
            32.0,
        ),
        Variant::Vibrant => from(
            rotated_hue(
                source,
                &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                &[35.0, 30.0, 20.0, 25.0, 30.0, 35.0, 30.0, 25.0, 25.0],
            ),
            32.0,
        ),
    }
}

pub fn neutral_palette(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => from(hue, source.chroma() / 8.0),
        Variant::FruitSalad => from(hue, 10.0),
        Variant::Monochrome | Variant::Rainbow => from(hue, 0.0),
        Variant::Neutral => from(hue, 2.0),
        Variant::TonalSpot => from(hue, 6.0),
        Variant::Expressive => from(utils::sanitize_degrees_double(hue + 15.0), 8.0),
        Variant::Vibrant => from(hue, 10.0),
    }
}

pub fn neutral_variant_palette(variant: Variant, source: Hct) -> TonalPalette {
    let hue = source.hue();
    match variant {
        Variant::Content | Variant::Fidelity => from(hue, source.chroma() / 8.0 + 4.0),
        Variant::FruitSalad => from(hue, 16.0),
        Variant::Monochrome | Variant::Rainbow => from(hue, 0.0),
        Variant::Neutral => from(hue, 2.0),
        Variant::TonalSpot => from(hue, 8.0),
        Variant::Expressive => from(utils::sanitize_degrees_double(hue + 15.0), 12.0),
        Variant::Vibrant => from(hue, 12.0),
    }
}

fn is_fidelity(s: &DynamicScheme) -> bool {
    matches!(s.variant, Variant::Fidelity | Variant::Content)
}

fn is_monochrome(s: &DynamicScheme) -> bool {
    s.variant == Variant::Monochrome
}

fn find_desired_chroma_by_tone(hue: f64, chroma: f64, tone: f64, by_decreasing_tone: bool) -> f64 {
    let mut answer = tone;
    let mut closest = Hct::from(hue, chroma, tone);
    if closest.chroma() < chroma {
        let mut chroma_peak = closest.chroma();
        while closest.chroma() < chroma {
            answer += if by_decreasing_tone { -1.0 } else { 1.0 };
            let potential = Hct::from(hue, chroma, answer);
            if chroma_peak > potential.chroma() {
                break;
            }
            if (potential.chroma() - chroma).abs() < 0.4 {
                break;
            }
            let potential_delta = (potential.chroma() - chroma).abs();
            let current_delta = (closest.chroma() - chroma).abs();
            if potential_delta < current_delta {
                closest = potential;
            }
            chroma_peak = chroma_peak.max(potential.chroma());
        }
    }
    answer
}

fn primary(s: &DynamicScheme) -> TonalPalette {
    s.primary_palette
}

fn secondary(s: &DynamicScheme) -> TonalPalette {
    s.secondary_palette
}

fn tertiary(s: &DynamicScheme) -> TonalPalette {
    s.tertiary_palette
}

fn neutral(s: &DynamicScheme) -> TonalPalette {
    s.neutral_palette
}

fn neutral_variant(s: &DynamicScheme) -> TonalPalette {
    s.neutral_variant_palette
}

fn error(s: &DynamicScheme) -> TonalPalette {
    s.error_palette
}

const fn curve(low: f64, normal: f64, medium: f64, high: f64) -> Option<ContrastCurve> {
    Some(ContrastCurve::new(low, normal, medium, high))
}

const fn nearer_pair(a: Role, b: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair::new(
        a,
        b,
        10.0,
        Polarity::Nearer,
        false,
        Constraint::Exact,
    ))
}

const fn lighter_pair(a: Role, b: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair::new(
        a,
        b,
        10.0,
        Polarity::Lighter,
        true,
        Constraint::Exact,
    ))
}

fn dark_light(s: &DynamicScheme, dark: f64, light: f64) -> f64 {
    if s.is_dark { dark } else { light }
}

pub fn def(role: Role) -> Def {
    match role {
        Role::Background | Role::Surface => Def {
            tone: Some(|s| dark_light(s, 6.0, 98.0)),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::OnBackground => Def {
            tone: Some(|s| dark_light(s, 90.0, 10.0)),
            background: Some(|_| Some(Role::Background)),
            contrast_curve: Some(|_| curve(3.0, 3.0, 4.5, 7.0)),
            ..Def::new(neutral)
        },
        Role::SurfaceDim => Def {
            tone: Some(|s| {
                if s.is_dark {
                    6.0
                } else {
                    ContrastCurve::new(87.0, 87.0, 80.0, 75.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::SurfaceBright => Def {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(24.0, 24.0, 29.0, 34.0).get(s.contrast_level)
                } else {
                    98.0
                }
            }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::SurfaceContainerLowest => Def {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(4.0, 4.0, 2.0, 0.0).get(s.contrast_level)
                } else {
                    100.0
                }
            }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::SurfaceContainerLow => Def {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(10.0, 10.0, 11.0, 12.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(96.0, 96.0, 96.0, 95.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::SurfaceContainer => Def {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(12.0, 12.0, 16.0, 20.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(94.0, 94.0, 92.0, 90.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::SurfaceContainerHigh => Def {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(17.0, 17.0, 21.0, 25.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(92.0, 92.0, 88.0, 85.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::SurfaceContainerHighest => Def {
            tone: Some(|s| {
                if s.is_dark {
                    ContrastCurve::new(22.0, 22.0, 26.0, 30.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(90.0, 90.0, 84.0, 80.0).get(s.contrast_level)
                }
            }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::OnSurface => Def {
            tone: Some(|s| dark_light(s, 90.0, 10.0)),
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(neutral)
        },
        Role::SurfaceVariant => Def {
            tone: Some(|s| dark_light(s, 30.0, 90.0)),
            is_background: true,
            ..Def::new(neutral_variant)
        },
        Role::OnSurfaceVariant => Def {
            tone: Some(|s| dark_light(s, 80.0, 30.0)),
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(neutral_variant)
        },
        Role::InverseSurface => Def {
            tone: Some(|s| dark_light(s, 90.0, 20.0)),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::InverseOnSurface => Def {
            tone: Some(|s| dark_light(s, 20.0, 95.0)),
            background: Some(|_| Some(Role::InverseSurface)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(neutral)
        },
        Role::Outline => Def {
            tone: Some(|s| dark_light(s, 60.0, 50.0)),
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.5, 3.0, 4.5, 7.0)),
            ..Def::new(neutral_variant)
        },
        Role::OutlineVariant => Def {
            tone: Some(|s| dark_light(s, 30.0, 80.0)),
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            ..Def::new(neutral_variant)
        },
        Role::Shadow | Role::Scrim => Def {
            tone: Some(|_| 0.0),
            ..Def::new(neutral)
        },
        Role::SurfaceTint => Def {
            tone: Some(|s| dark_light(s, 80.0, 40.0)),
            is_background: true,
            ..Def::new(primary)
        },
        Role::Primary => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 100.0, 0.0)
                } else {
                    dark_light(s, 80.0, 40.0)
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 7.0)),
            tone_delta_pair: Some(|_| nearer_pair(Role::PrimaryContainer, Role::Primary)),
            ..Def::new(primary)
        },
        Role::OnPrimary => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 10.0, 90.0)
                } else {
                    dark_light(s, 20.0, 100.0)
                }
            }),
            background: Some(|_| Some(Role::Primary)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(primary)
        },
        Role::PrimaryContainer => Def {
            tone: Some(|s| {
                if is_fidelity(s) {
                    s.source.tone()
                } else if is_monochrome(s) {
                    dark_light(s, 85.0, 25.0)
                } else {
                    dark_light(s, 30.0, 90.0)
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| nearer_pair(Role::PrimaryContainer, Role::Primary)),
            ..Def::new(primary)
        },
        Role::OnPrimaryContainer => Def {
            tone: Some(|s| {
                if is_fidelity(s) {
                    foreground_tone(dynamic::tone(Role::PrimaryContainer, s), 4.5)
                } else if is_monochrome(s) {
                    dark_light(s, 0.0, 100.0)
                } else {
                    dark_light(s, 90.0, 30.0)
                }
            }),
            background: Some(|_| Some(Role::PrimaryContainer)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(primary)
        },
        Role::InversePrimary => Def {
            tone: Some(|s| dark_light(s, 40.0, 80.0)),
            background: Some(|_| Some(Role::InverseSurface)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 7.0)),
            ..Def::new(primary)
        },
        Role::Secondary => Def {
            tone: Some(|s| dark_light(s, 80.0, 40.0)),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 7.0)),
            tone_delta_pair: Some(|_| nearer_pair(Role::SecondaryContainer, Role::Secondary)),
            ..Def::new(secondary)
        },
        Role::OnSecondary => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 10.0, 100.0)
                } else {
                    dark_light(s, 20.0, 100.0)
                }
            }),
            background: Some(|_| Some(Role::Secondary)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(secondary)
        },
        Role::SecondaryContainer => Def {
            tone: Some(|s| {
                let initial = dark_light(s, 30.0, 90.0);
                if is_monochrome(s) {
                    dark_light(s, 30.0, 85.0)
                } else if !is_fidelity(s) {
                    initial
                } else {
                    find_desired_chroma_by_tone(
                        s.secondary_palette.hue,
                        s.secondary_palette.chroma,
                        initial,
                        !s.is_dark,
                    )
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| nearer_pair(Role::SecondaryContainer, Role::Secondary)),
            ..Def::new(secondary)
        },
        Role::OnSecondaryContainer => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 90.0, 10.0)
                } else if !is_fidelity(s) {
                    dark_light(s, 90.0, 30.0)
                } else {
                    foreground_tone(dynamic::tone(Role::SecondaryContainer, s), 4.5)
                }
            }),
            background: Some(|_| Some(Role::SecondaryContainer)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(secondary)
        },
        Role::Tertiary => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 90.0, 25.0)
                } else {
                    dark_light(s, 80.0, 40.0)
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 7.0)),
            tone_delta_pair: Some(|_| nearer_pair(Role::TertiaryContainer, Role::Tertiary)),
            ..Def::new(tertiary)
        },
        Role::OnTertiary => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 10.0, 90.0)
                } else {
                    dark_light(s, 20.0, 100.0)
                }
            }),
            background: Some(|_| Some(Role::Tertiary)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(tertiary)
        },
        Role::TertiaryContainer => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 60.0, 49.0)
                } else if !is_fidelity(s) {
                    dark_light(s, 30.0, 90.0)
                } else {
                    let proposed = s.tertiary_palette.get_hct(s.source.tone());
                    dislike::fix_if_disliked(proposed).tone()
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| nearer_pair(Role::TertiaryContainer, Role::Tertiary)),
            ..Def::new(tertiary)
        },
        Role::OnTertiaryContainer => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 0.0, 100.0)
                } else if !is_fidelity(s) {
                    dark_light(s, 90.0, 30.0)
                } else {
                    foreground_tone(dynamic::tone(Role::TertiaryContainer, s), 4.5)
                }
            }),
            background: Some(|_| Some(Role::TertiaryContainer)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(tertiary)
        },
        Role::Error => Def {
            tone: Some(|s| dark_light(s, 80.0, 40.0)),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 7.0)),
            tone_delta_pair: Some(|_| nearer_pair(Role::ErrorContainer, Role::Error)),
            ..Def::new(error)
        },
        Role::OnError => Def {
            tone: Some(|s| dark_light(s, 20.0, 100.0)),
            background: Some(|_| Some(Role::Error)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(error)
        },
        Role::ErrorContainer => Def {
            tone: Some(|s| dark_light(s, 30.0, 90.0)),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| nearer_pair(Role::ErrorContainer, Role::Error)),
            ..Def::new(error)
        },
        Role::OnErrorContainer => Def {
            tone: Some(|s| {
                if is_monochrome(s) {
                    dark_light(s, 90.0, 10.0)
                } else {
                    dark_light(s, 90.0, 30.0)
                }
            }),
            background: Some(|_| Some(Role::ErrorContainer)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(error)
        },
        Role::PrimaryFixed => Def {
            tone: Some(|s| if is_monochrome(s) { 40.0 } else { 90.0 }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| lighter_pair(Role::PrimaryFixed, Role::PrimaryFixedDim)),
            ..Def::new(primary)
        },
        Role::PrimaryFixedDim => Def {
            tone: Some(|s| if is_monochrome(s) { 30.0 } else { 80.0 }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| lighter_pair(Role::PrimaryFixed, Role::PrimaryFixedDim)),
            ..Def::new(primary)
        },
        Role::OnPrimaryFixed => Def {
            tone: Some(|s| if is_monochrome(s) { 100.0 } else { 10.0 }),
            background: Some(|_| Some(Role::PrimaryFixedDim)),
            second_background: Some(|_| Some(Role::PrimaryFixed)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(primary)
        },
        Role::OnPrimaryFixedVariant => Def {
            tone: Some(|s| if is_monochrome(s) { 90.0 } else { 30.0 }),
            background: Some(|_| Some(Role::PrimaryFixedDim)),
            second_background: Some(|_| Some(Role::PrimaryFixed)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(primary)
        },
        Role::SecondaryFixed => Def {
            tone: Some(|s| if is_monochrome(s) { 80.0 } else { 90.0 }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| lighter_pair(Role::SecondaryFixed, Role::SecondaryFixedDim)),
            ..Def::new(secondary)
        },
        Role::SecondaryFixedDim => Def {
            tone: Some(|s| if is_monochrome(s) { 70.0 } else { 80.0 }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| lighter_pair(Role::SecondaryFixed, Role::SecondaryFixedDim)),
            ..Def::new(secondary)
        },
        Role::OnSecondaryFixed => Def {
            tone: Some(|_| 10.0),
            background: Some(|_| Some(Role::SecondaryFixedDim)),
            second_background: Some(|_| Some(Role::SecondaryFixed)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(secondary)
        },
        Role::OnSecondaryFixedVariant => Def {
            tone: Some(|s| if is_monochrome(s) { 25.0 } else { 30.0 }),
            background: Some(|_| Some(Role::SecondaryFixedDim)),
            second_background: Some(|_| Some(Role::SecondaryFixed)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(secondary)
        },
        Role::TertiaryFixed => Def {
            tone: Some(|s| if is_monochrome(s) { 40.0 } else { 90.0 }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| lighter_pair(Role::TertiaryFixed, Role::TertiaryFixedDim)),
            ..Def::new(tertiary)
        },
        Role::TertiaryFixedDim => Def {
            tone: Some(|s| if is_monochrome(s) { 30.0 } else { 80.0 }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
            tone_delta_pair: Some(|_| lighter_pair(Role::TertiaryFixed, Role::TertiaryFixedDim)),
            ..Def::new(tertiary)
        },
        Role::OnTertiaryFixed => Def {
            tone: Some(|s| if is_monochrome(s) { 100.0 } else { 10.0 }),
            background: Some(|_| Some(Role::TertiaryFixedDim)),
            second_background: Some(|_| Some(Role::TertiaryFixed)),
            contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
            ..Def::new(tertiary)
        },
        Role::OnTertiaryFixedVariant => Def {
            tone: Some(|s| if is_monochrome(s) { 90.0 } else { 30.0 }),
            background: Some(|_| Some(Role::TertiaryFixedDim)),
            second_background: Some(|_| Some(Role::TertiaryFixed)),
            contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
            ..Def::new(tertiary)
        },
    }
}
