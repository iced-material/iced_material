// SPDX-License-Identifier: LGPL-3.0-only

use super::dynamic::{
    self, Constraint, ContrastCurve, Def, DynamicScheme, Polarity, Role, ToneDeltaPair, Variant,
    highest_surface, piecewise_hue, rotated_hue,
};
use super::hct::Hct;
use super::palette::TonalPalette;
use super::spec_2021;
use super::utils;

fn from(hue: f64, chroma: f64) -> TonalPalette {
    TonalPalette::from_hue_and_chroma(hue, chroma)
}

fn expressive_neutral_hue(source: Hct) -> f64 {
    rotated_hue(
        source,
        &[0.0, 71.0, 124.0, 253.0, 278.0, 300.0, 360.0],
        &[10.0, 0.0, 10.0, 0.0, 10.0, 0.0],
    )
}

fn expressive_neutral_chroma(source: Hct, is_dark: bool) -> f64 {
    let neutral_hue = expressive_neutral_hue(source);
    if is_dark {
        if Hct::is_yellow(neutral_hue) {
            6.0
        } else {
            14.0
        }
    } else {
        18.0
    }
}

fn vibrant_neutral_hue(source: Hct) -> f64 {
    rotated_hue(
        source,
        &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
        &[-14.0, 10.0, -14.0, 10.0, -14.0],
    )
}

pub fn palettes(variant: Variant, source: Hct, is_dark: bool) -> [TonalPalette; 6] {
    let hue = source.hue();
    let primary = match variant {
        Variant::Neutral => from(hue, if Hct::is_blue(hue) { 12.0 } else { 8.0 }),
        Variant::TonalSpot => from(hue, if is_dark { 26.0 } else { 32.0 }),
        Variant::Expressive => from(hue, if is_dark { 36.0 } else { 48.0 }),
        Variant::Vibrant => from(hue, 74.0),
        _ => spec_2021::primary_palette(variant, source),
    };
    let secondary = match variant {
        Variant::Neutral => from(hue, if Hct::is_blue(hue) { 6.0 } else { 4.0 }),
        Variant::TonalSpot => from(hue, 16.0),
        Variant::Expressive => from(
            rotated_hue(
                source,
                &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                &[-160.0, 155.0, -100.0, 96.0, -96.0, -156.0, -165.0, -160.0],
            ),
            if is_dark { 16.0 } else { 24.0 },
        ),
        Variant::Vibrant => from(
            rotated_hue(
                source,
                &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
                &[-14.0, 10.0, -14.0, 10.0, -14.0],
            ),
            56.0,
        ),
        _ => spec_2021::secondary_palette(variant, source),
    };
    let tertiary = match variant {
        Variant::Neutral => from(
            rotated_hue(
                source,
                &[0.0, 38.0, 105.0, 161.0, 204.0, 278.0, 333.0, 360.0],
                &[-32.0, 26.0, 10.0, -39.0, 24.0, -15.0, -32.0],
            ),
            20.0,
        ),
        Variant::TonalSpot => from(
            rotated_hue(
                source,
                &[0.0, 20.0, 71.0, 161.0, 333.0, 360.0],
                &[-40.0, 48.0, -32.0, 40.0, -32.0],
            ),
            28.0,
        ),
        Variant::Expressive => from(
            rotated_hue(
                source,
                &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                &[-165.0, 160.0, -105.0, 101.0, -101.0, -160.0, -170.0, -165.0],
            ),
            48.0,
        ),
        Variant::Vibrant => from(
            rotated_hue(
                source,
                &[0.0, 38.0, 71.0, 105.0, 140.0, 161.0, 253.0, 333.0, 360.0],
                &[-72.0, 35.0, 24.0, -24.0, 62.0, 50.0, 62.0, -72.0],
            ),
            56.0,
        ),
        _ => spec_2021::tertiary_palette(variant, source),
    };
    let neutral = match variant {
        Variant::Neutral => from(hue, 1.4),
        Variant::TonalSpot => from(hue, 5.0),
        Variant::Expressive => from(
            expressive_neutral_hue(source),
            expressive_neutral_chroma(source, is_dark),
        ),
        Variant::Vibrant => from(vibrant_neutral_hue(source), 28.0),
        _ => spec_2021::neutral_palette(variant, source),
    };
    let neutral_variant = match variant {
        Variant::Neutral => from(hue, 1.4 * 2.2),
        Variant::TonalSpot => from(hue, 5.0 * 1.7),
        Variant::Expressive => {
            let neutral_hue = expressive_neutral_hue(source);
            let neutral_chroma = expressive_neutral_chroma(source, is_dark);
            let factor = if (105.0..125.0).contains(&neutral_hue) {
                1.6
            } else {
                2.3
            };
            from(neutral_hue, neutral_chroma * factor)
        }
        Variant::Vibrant => from(vibrant_neutral_hue(source), 28.0 * 1.29),
        _ => spec_2021::neutral_variant_palette(variant, source),
    };
    let error_hue = piecewise_hue(
        source,
        &[0.0, 3.0, 13.0, 23.0, 33.0, 43.0, 153.0, 273.0, 360.0],
        &[12.0, 22.0, 32.0, 12.0, 22.0, 32.0, 22.0, 12.0],
    );
    let error = match variant {
        Variant::Neutral => from(error_hue, 50.0),
        Variant::TonalSpot => from(error_hue, 60.0),
        Variant::Expressive => from(error_hue, 64.0),
        Variant::Vibrant => from(error_hue, 80.0),
        _ => from(25.0, 84.0),
    };
    [
        primary,
        secondary,
        tertiary,
        neutral,
        neutral_variant,
        error,
    ]
}

fn find_best_tone_for_chroma(
    hue: f64,
    chroma: f64,
    mut tone: f64,
    by_decreasing_tone: bool,
) -> f64 {
    let mut answer = tone;
    let mut best = Hct::from(hue, chroma, answer);
    while best.chroma() < chroma {
        if !(0.0..=100.0).contains(&tone) {
            break;
        }
        tone += if by_decreasing_tone { -1.0 } else { 1.0 };
        let candidate = Hct::from(hue, chroma, tone);
        if best.chroma() < candidate.chroma() {
            best = candidate;
            answer = tone;
        }
    }
    answer
}

fn t_max_c(palette: TonalPalette, lower: f64, upper: f64) -> f64 {
    t_max_c_scaled(palette, lower, upper, 1.0)
}

fn t_max_c_scaled(palette: TonalPalette, lower: f64, upper: f64, chroma_multiplier: f64) -> f64 {
    let answer =
        find_best_tone_for_chroma(palette.hue, palette.chroma * chroma_multiplier, 100.0, true);
    utils::clamp_double(lower, upper, answer)
}

fn t_min_c(palette: TonalPalette, lower: f64, upper: f64) -> f64 {
    let answer = find_best_tone_for_chroma(palette.hue, palette.chroma, 0.0, false);
    utils::clamp_double(lower, upper, answer)
}

fn get_curve(default_contrast: f64) -> Option<ContrastCurve> {
    Some(if default_contrast == 1.5 {
        ContrastCurve::new(1.5, 1.5, 3.0, 5.5)
    } else if default_contrast == 3.0 {
        ContrastCurve::new(3.0, 3.0, 4.5, 7.0)
    } else if default_contrast == 4.5 {
        ContrastCurve::new(4.5, 4.5, 7.0, 11.0)
    } else if default_contrast == 6.0 {
        ContrastCurve::new(6.0, 6.0, 7.0, 11.0)
    } else if default_contrast == 7.0 {
        ContrastCurve::new(7.0, 7.0, 11.0, 21.0)
    } else if default_contrast == 9.0 {
        ContrastCurve::new(9.0, 9.0, 11.0, 21.0)
    } else if default_contrast == 11.0 {
        ContrastCurve::new(11.0, 11.0, 21.0, 21.0)
    } else if default_contrast == 21.0 {
        ContrastCurve::new(21.0, 21.0, 21.0, 21.0)
    } else {
        ContrastCurve::new(default_contrast, default_contrast, 7.0, 21.0)
    })
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

fn error(s: &DynamicScheme) -> TonalPalette {
    s.error_palette
}

fn yellow_neutral(s: &DynamicScheme) -> bool {
    Hct::is_yellow(s.neutral_palette.hue)
}

fn light_surface_tone(s: &DynamicScheme, yellow: f64, vibrant: f64, other: f64) -> f64 {
    if yellow_neutral(s) {
        yellow
    } else if s.variant == Variant::Vibrant {
        vibrant
    } else {
        other
    }
}

fn neutral_multiplier(
    s: &DynamicScheme,
    neutral: f64,
    tonal_spot: f64,
    expressive_yellow: f64,
    expressive: f64,
    vibrant: f64,
) -> f64 {
    match s.variant {
        Variant::Neutral => neutral,
        Variant::TonalSpot => tonal_spot,
        Variant::Expressive => {
            if yellow_neutral(s) {
                expressive_yellow
            } else {
                expressive
            }
        }
        Variant::Vibrant => vibrant,
        _ => 1.0,
    }
}

fn on_surface_multiplier(s: &DynamicScheme) -> f64 {
    let expressive_yellow = if s.is_dark { 3.0 } else { 2.3 };
    neutral_multiplier(s, 2.2, 1.7, expressive_yellow, 1.6, 1.0)
}

fn container_curve(s: &DynamicScheme) -> Option<ContrastCurve> {
    if s.contrast_level > 0.0 {
        get_curve(1.5)
    } else {
        None
    }
}

const fn accent_pair(container: Role, accent: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair::new(
        container,
        accent,
        5.0,
        Polarity::RelativeLighter,
        true,
        Constraint::Farther,
    ))
}

const fn fixed_dim_pair(dim: Role, fixed: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair::new(
        dim,
        fixed,
        5.0,
        Polarity::Darker,
        true,
        Constraint::Exact,
    ))
}

fn light_standard(s: &DynamicScheme) -> DynamicScheme {
    DynamicScheme {
        is_dark: false,
        contrast_level: 0.0,
        ..s.clone()
    }
}

fn surface() -> Def {
    Def {
        tone: Some(|s| {
            if s.is_dark {
                4.0
            } else {
                light_surface_tone(s, 99.0, 97.0, 98.0)
            }
        }),
        is_background: true,
        ..Def::new(neutral)
    }
}

fn surface_container_highest() -> Def {
    Def {
        tone: Some(|s| {
            if s.is_dark {
                15.0
            } else {
                light_surface_tone(s, 92.0, 88.0, 90.0)
            }
        }),
        is_background: true,
        chroma_multiplier: Some(|s| neutral_multiplier(s, 2.2, 1.7, 2.3, 1.6, 1.29)),
        ..Def::new(neutral)
    }
}

fn primary_def() -> Def {
    Def {
        tone: Some(|s| {
            let p = s.primary_palette;
            match s.variant {
                Variant::Neutral => {
                    if s.is_dark {
                        80.0
                    } else {
                        40.0
                    }
                }
                Variant::TonalSpot => {
                    if s.is_dark {
                        80.0
                    } else {
                        t_max_c(p, 0.0, 100.0)
                    }
                }
                Variant::Expressive => {
                    let upper = if s.is_dark {
                        if Hct::is_cyan(p.hue) { 88.0 } else { 98.0 }
                    } else if Hct::is_yellow(p.hue) {
                        25.0
                    } else {
                        98.0
                    };
                    t_max_c(p, 0.0, upper)
                }
                _ => t_max_c(p, 0.0, if Hct::is_cyan(p.hue) { 88.0 } else { 98.0 }),
            }
        }),
        is_background: true,
        background: Some(highest_surface),
        contrast_curve: Some(|_| get_curve(4.5)),
        tone_delta_pair: Some(|_| accent_pair(Role::PrimaryContainer, Role::Primary)),
        ..Def::new(primary)
    }
}

fn on_surface() -> Def {
    Def {
        tone: Some(|s| {
            if s.variant == Variant::Vibrant {
                t_max_c_scaled(s.neutral_palette, 0.0, 100.0, 1.1)
            } else {
                dynamic::get_tone(
                    if s.is_dark {
                        Role::SurfaceBright
                    } else {
                        Role::SurfaceDim
                    },
                    s,
                )
            }
        }),
        chroma_multiplier: Some(on_surface_multiplier),
        background: Some(highest_surface),
        contrast_curve: Some(|s| {
            if s.is_dark {
                get_curve(11.0)
            } else {
                get_curve(9.0)
            }
        }),
        ..Def::new(neutral)
    }
}

pub fn def(role: Role) -> Def {
    match role {
        Role::Surface | Role::Background => surface(),
        Role::SurfaceDim => Def {
            tone: Some(|s| {
                if s.is_dark {
                    4.0
                } else {
                    light_surface_tone(s, 90.0, 85.0, 87.0)
                }
            }),
            is_background: true,
            chroma_multiplier: Some(|s| {
                if s.is_dark {
                    1.0
                } else {
                    neutral_multiplier(s, 2.5, 1.7, 2.7, 1.75, 1.36)
                }
            }),
            ..Def::new(neutral)
        },
        Role::SurfaceBright => Def {
            tone: Some(|s| {
                if s.is_dark {
                    18.0
                } else {
                    light_surface_tone(s, 99.0, 97.0, 98.0)
                }
            }),
            is_background: true,
            chroma_multiplier: Some(|s| {
                if s.is_dark {
                    neutral_multiplier(s, 2.5, 1.7, 2.7, 1.75, 1.36)
                } else {
                    1.0
                }
            }),
            ..Def::new(neutral)
        },
        Role::SurfaceContainerLowest => Def {
            tone: Some(|s| if s.is_dark { 0.0 } else { 100.0 }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::SurfaceContainerLow => Def {
            tone: Some(|s| {
                if s.is_dark {
                    6.0
                } else {
                    light_surface_tone(s, 98.0, 95.0, 96.0)
                }
            }),
            is_background: true,
            chroma_multiplier: Some(|s| neutral_multiplier(s, 1.3, 1.25, 1.3, 1.15, 1.08)),
            ..Def::new(neutral)
        },
        Role::SurfaceContainer => Def {
            tone: Some(|s| {
                if s.is_dark {
                    9.0
                } else {
                    light_surface_tone(s, 96.0, 92.0, 94.0)
                }
            }),
            is_background: true,
            chroma_multiplier: Some(|s| neutral_multiplier(s, 1.6, 1.4, 1.6, 1.3, 1.15)),
            ..Def::new(neutral)
        },
        Role::SurfaceContainerHigh => Def {
            tone: Some(|s| {
                if s.is_dark {
                    12.0
                } else {
                    light_surface_tone(s, 94.0, 90.0, 92.0)
                }
            }),
            is_background: true,
            chroma_multiplier: Some(|s| neutral_multiplier(s, 1.9, 1.5, 1.95, 1.45, 1.22)),
            ..Def::new(neutral)
        },
        Role::SurfaceContainerHighest | Role::SurfaceVariant => surface_container_highest(),
        Role::OnSurface => on_surface(),
        Role::OnBackground => Def {
            tone: Some(|s| dynamic::get_tone(Role::OnSurface, s)),
            ..on_surface()
        },
        Role::OnSurfaceVariant => Def {
            chroma_multiplier: Some(on_surface_multiplier),
            background: Some(highest_surface),
            contrast_curve: Some(|s| {
                if s.is_dark {
                    get_curve(6.0)
                } else {
                    get_curve(4.5)
                }
            }),
            ..Def::new(neutral)
        },
        Role::Outline => Def {
            chroma_multiplier: Some(on_surface_multiplier),
            background: Some(highest_surface),
            contrast_curve: Some(|_| get_curve(3.0)),
            ..Def::new(neutral)
        },
        Role::OutlineVariant => Def {
            chroma_multiplier: Some(on_surface_multiplier),
            background: Some(highest_surface),
            contrast_curve: Some(|_| get_curve(1.5)),
            ..Def::new(neutral)
        },
        Role::InverseSurface => Def {
            tone: Some(|s| if s.is_dark { 98.0 } else { 4.0 }),
            is_background: true,
            ..Def::new(neutral)
        },
        Role::InverseOnSurface => Def {
            background: Some(|_| Some(Role::InverseSurface)),
            contrast_curve: Some(|_| get_curve(7.0)),
            ..Def::new(neutral)
        },
        Role::Primary | Role::SurfaceTint => primary_def(),
        Role::OnPrimary => Def {
            background: Some(|_| Some(Role::Primary)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(primary)
        },
        Role::PrimaryContainer => Def {
            tone: Some(|s| {
                let p = s.primary_palette;
                let cyan = Hct::is_cyan(p.hue);
                match s.variant {
                    Variant::Neutral => {
                        if s.is_dark {
                            30.0
                        } else {
                            90.0
                        }
                    }
                    Variant::TonalSpot => {
                        if s.is_dark {
                            t_min_c(p, 35.0, 93.0)
                        } else {
                            t_max_c(p, 0.0, 90.0)
                        }
                    }
                    Variant::Expressive => {
                        if s.is_dark {
                            t_min_c(p, 30.0, 93.0)
                        } else {
                            t_max_c(p, 78.0, if cyan { 88.0 } else { 90.0 })
                        }
                    }
                    _ => {
                        if s.is_dark {
                            t_min_c(p, 66.0, 93.0)
                        } else {
                            t_max_c(p, 66.0, if cyan { 88.0 } else { 93.0 })
                        }
                    }
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(container_curve),
            ..Def::new(primary)
        },
        Role::OnPrimaryContainer => Def {
            background: Some(|_| Some(Role::PrimaryContainer)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(primary)
        },
        Role::PrimaryFixed => Def {
            tone: Some(|s| dynamic::get_tone(Role::PrimaryContainer, &light_standard(s))),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(container_curve),
            ..Def::new(primary)
        },
        Role::PrimaryFixedDim => Def {
            tone: Some(|s| dynamic::get_tone(Role::PrimaryFixed, s)),
            is_background: true,
            tone_delta_pair: Some(|_| fixed_dim_pair(Role::PrimaryFixedDim, Role::PrimaryFixed)),
            ..Def::new(primary)
        },
        Role::OnPrimaryFixed => Def {
            background: Some(|_| Some(Role::PrimaryFixedDim)),
            contrast_curve: Some(|_| get_curve(7.0)),
            ..Def::new(primary)
        },
        Role::OnPrimaryFixedVariant => Def {
            background: Some(|_| Some(Role::PrimaryFixedDim)),
            contrast_curve: Some(|_| get_curve(4.5)),
            ..Def::new(primary)
        },
        Role::InversePrimary => Def {
            tone: Some(|s| t_max_c(s.primary_palette, 0.0, 100.0)),
            background: Some(|_| Some(Role::InverseSurface)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(primary)
        },
        Role::Secondary => Def {
            tone: Some(|s| {
                let p = s.secondary_palette;
                match s.variant {
                    Variant::Neutral => {
                        if s.is_dark {
                            t_min_c(p, 0.0, 98.0)
                        } else {
                            t_max_c(p, 0.0, 100.0)
                        }
                    }
                    Variant::Vibrant => t_max_c(p, 0.0, if s.is_dark { 90.0 } else { 98.0 }),
                    _ => {
                        if s.is_dark {
                            80.0
                        } else {
                            t_max_c(p, 0.0, 100.0)
                        }
                    }
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| get_curve(4.5)),
            tone_delta_pair: Some(|_| accent_pair(Role::SecondaryContainer, Role::Secondary)),
            ..Def::new(secondary)
        },
        Role::OnSecondary => Def {
            background: Some(|_| Some(Role::Secondary)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(secondary)
        },
        Role::SecondaryContainer => Def {
            tone: Some(|s| {
                let p = s.secondary_palette;
                match s.variant {
                    Variant::Vibrant => {
                        if s.is_dark {
                            t_min_c(p, 30.0, 40.0)
                        } else {
                            t_max_c(p, 84.0, 90.0)
                        }
                    }
                    Variant::Expressive => {
                        if s.is_dark {
                            15.0
                        } else {
                            t_max_c(p, 90.0, 95.0)
                        }
                    }
                    _ => {
                        if s.is_dark {
                            25.0
                        } else {
                            90.0
                        }
                    }
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(container_curve),
            ..Def::new(secondary)
        },
        Role::OnSecondaryContainer => Def {
            background: Some(|_| Some(Role::SecondaryContainer)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(secondary)
        },
        Role::SecondaryFixed => Def {
            tone: Some(|s| dynamic::get_tone(Role::SecondaryContainer, &light_standard(s))),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(container_curve),
            ..Def::new(secondary)
        },
        Role::SecondaryFixedDim => Def {
            tone: Some(|s| dynamic::get_tone(Role::SecondaryFixed, s)),
            is_background: true,
            tone_delta_pair: Some(|_| {
                fixed_dim_pair(Role::SecondaryFixedDim, Role::SecondaryFixed)
            }),
            ..Def::new(secondary)
        },
        Role::OnSecondaryFixed => Def {
            background: Some(|_| Some(Role::SecondaryFixedDim)),
            contrast_curve: Some(|_| get_curve(7.0)),
            ..Def::new(secondary)
        },
        Role::OnSecondaryFixedVariant => Def {
            background: Some(|_| Some(Role::SecondaryFixedDim)),
            contrast_curve: Some(|_| get_curve(4.5)),
            ..Def::new(secondary)
        },
        Role::Tertiary => Def {
            tone: Some(|s| {
                let p = s.tertiary_palette;
                match s.variant {
                    Variant::Expressive | Variant::Vibrant => {
                        let upper = if Hct::is_cyan(p.hue) {
                            88.0
                        } else if s.is_dark {
                            98.0
                        } else {
                            100.0
                        };
                        t_max_c(p, 0.0, upper)
                    }
                    _ => {
                        if s.is_dark {
                            t_max_c(p, 0.0, 98.0)
                        } else {
                            t_max_c(p, 0.0, 100.0)
                        }
                    }
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| get_curve(4.5)),
            tone_delta_pair: Some(|_| accent_pair(Role::TertiaryContainer, Role::Tertiary)),
            ..Def::new(tertiary)
        },
        Role::OnTertiary => Def {
            background: Some(|_| Some(Role::Tertiary)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(tertiary)
        },
        Role::TertiaryContainer => Def {
            tone: Some(|s| {
                let p = s.tertiary_palette;
                match s.variant {
                    Variant::Neutral => {
                        if s.is_dark {
                            t_max_c(p, 0.0, 93.0)
                        } else {
                            t_max_c(p, 0.0, 96.0)
                        }
                    }
                    Variant::TonalSpot => t_max_c(p, 0.0, if s.is_dark { 93.0 } else { 100.0 }),
                    Variant::Expressive => {
                        let upper = if Hct::is_cyan(p.hue) {
                            88.0
                        } else if s.is_dark {
                            93.0
                        } else {
                            100.0
                        };
                        t_max_c(p, 75.0, upper)
                    }
                    _ => {
                        if s.is_dark {
                            t_max_c(p, 0.0, 93.0)
                        } else {
                            t_max_c(p, 72.0, 100.0)
                        }
                    }
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(container_curve),
            ..Def::new(tertiary)
        },
        Role::OnTertiaryContainer => Def {
            background: Some(|_| Some(Role::TertiaryContainer)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(tertiary)
        },
        Role::TertiaryFixed => Def {
            tone: Some(|s| dynamic::get_tone(Role::TertiaryContainer, &light_standard(s))),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(container_curve),
            ..Def::new(tertiary)
        },
        Role::TertiaryFixedDim => Def {
            tone: Some(|s| dynamic::get_tone(Role::TertiaryFixed, s)),
            is_background: true,
            tone_delta_pair: Some(|_| fixed_dim_pair(Role::TertiaryFixedDim, Role::TertiaryFixed)),
            ..Def::new(tertiary)
        },
        Role::OnTertiaryFixed => Def {
            background: Some(|_| Some(Role::TertiaryFixedDim)),
            contrast_curve: Some(|_| get_curve(7.0)),
            ..Def::new(tertiary)
        },
        Role::OnTertiaryFixedVariant => Def {
            background: Some(|_| Some(Role::TertiaryFixedDim)),
            contrast_curve: Some(|_| get_curve(4.5)),
            ..Def::new(tertiary)
        },
        Role::Error => Def {
            tone: Some(|s| {
                if s.is_dark {
                    t_min_c(s.error_palette, 0.0, 98.0)
                } else {
                    t_max_c(s.error_palette, 0.0, 100.0)
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(|_| get_curve(4.5)),
            tone_delta_pair: Some(|_| accent_pair(Role::ErrorContainer, Role::Error)),
            ..Def::new(error)
        },
        Role::OnError => Def {
            background: Some(|_| Some(Role::Error)),
            contrast_curve: Some(|_| get_curve(6.0)),
            ..Def::new(error)
        },
        Role::ErrorContainer => Def {
            tone: Some(|s| {
                if s.is_dark {
                    t_min_c(s.error_palette, 30.0, 93.0)
                } else {
                    t_max_c(s.error_palette, 0.0, 90.0)
                }
            }),
            is_background: true,
            background: Some(highest_surface),
            contrast_curve: Some(container_curve),
            ..Def::new(error)
        },
        Role::OnErrorContainer => Def {
            background: Some(|_| Some(Role::ErrorContainer)),
            contrast_curve: Some(|_| get_curve(4.5)),
            ..Def::new(error)
        },
        Role::Shadow | Role::Scrim => spec_2021::def(role),
    }
}
