// SPDX-License-Identifier: LGPL-3.0-only

use super::contrast;
use super::hct::Hct;
use super::palette::{TonalPalette, tone_of};
use super::spec_2021;
use super::spec_2025;
use super::utils::{self, js_round};

/// Scheme generation algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// Grayscale scheme.
    Monochrome,
    /// Near grayscale scheme with a hint of the source hue.
    Neutral,
    /// Low chroma scheme. This is the Material default.
    TonalSpot,
    /// High chroma scheme.
    Vibrant,
    /// Scheme with rotated hues.
    Expressive,
    /// Scheme that keeps the source color as the primary container.
    Fidelity,
    /// Scheme for content-derived colors, close to the source color.
    Content,
    /// Scheme with chromatic accents and grayscale neutrals.
    Rainbow,
    /// Scheme with hues rotated away from the source.
    FruitSalad,
}

impl Variant {
    /// All variants in a fixed order.
    pub const ALL: [Variant; 9] = [
        Variant::Monochrome,
        Variant::Neutral,
        Variant::TonalSpot,
        Variant::Vibrant,
        Variant::Expressive,
        Variant::Fidelity,
        Variant::Content,
        Variant::Rainbow,
        Variant::FruitSalad,
    ];
}

/// Version of the Material color specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpecVersion {
    /// The original Material 3 color specification.
    Spec2021,
    /// The 2025 color specification. Only TonalSpot, Neutral, Vibrant and Expressive use it.
    Spec2025,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicScheme {
    pub source: Hct,
    pub variant: Variant,
    pub is_dark: bool,
    pub contrast_level: f64,
    pub spec_version: SpecVersion,
    pub primary_palette: TonalPalette,
    pub secondary_palette: TonalPalette,
    pub tertiary_palette: TonalPalette,
    pub neutral_palette: TonalPalette,
    pub neutral_variant_palette: TonalPalette,
    pub error_palette: TonalPalette,
}

impl DynamicScheme {
    pub fn new(
        source: Hct,
        variant: Variant,
        is_dark: bool,
        contrast_level: f64,
        spec_version: SpecVersion,
    ) -> DynamicScheme {
        let spec_version = match variant {
            Variant::Expressive | Variant::Vibrant | Variant::TonalSpot | Variant::Neutral => {
                spec_version
            }
            _ => SpecVersion::Spec2021,
        };
        let palettes = match spec_version {
            SpecVersion::Spec2021 => spec_2021::palettes(variant, source),
            SpecVersion::Spec2025 => spec_2025::palettes(variant, source, is_dark),
        };
        DynamicScheme {
            source,
            variant,
            is_dark,
            contrast_level,
            spec_version,
            primary_palette: palettes[0],
            secondary_palette: palettes[1],
            tertiary_palette: palettes[2],
            neutral_palette: palettes[3],
            neutral_variant_palette: palettes[4],
            error_palette: palettes[5],
        }
    }

    pub fn argb(&self, role: Role) -> u32 {
        get_hct(role, self).to_int()
    }
}

pub fn piecewise_hue(source: Hct, breakpoints: &[f64], hues: &[f64]) -> f64 {
    let size = (breakpoints.len() - 1).min(hues.len());
    let source_hue = source.hue();
    for i in 0..size {
        if source_hue >= breakpoints[i] && source_hue < breakpoints[i + 1] {
            return utils::sanitize_degrees_double(hues[i]);
        }
    }
    source_hue
}

pub fn rotated_hue(source: Hct, breakpoints: &[f64], rotations: &[f64]) -> f64 {
    let mut rotation = piecewise_hue(source, breakpoints, rotations);
    if (breakpoints.len() - 1).min(rotations.len()) == 0 {
        rotation = 0.0;
    }
    utils::sanitize_degrees_double(source.hue() + rotation)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastCurve {
    pub low: f64,
    pub normal: f64,
    pub medium: f64,
    pub high: f64,
}

impl ContrastCurve {
    pub const fn new(low: f64, normal: f64, medium: f64, high: f64) -> ContrastCurve {
        ContrastCurve {
            low,
            normal,
            medium,
            high,
        }
    }

    pub fn get(&self, contrast_level: f64) -> f64 {
        if contrast_level <= -1.0 {
            self.low
        } else if contrast_level < 0.0 {
            utils::lerp(self.low, self.normal, contrast_level + 1.0)
        } else if contrast_level < 0.5 {
            utils::lerp(self.normal, self.medium, contrast_level / 0.5)
        } else if contrast_level < 1.0 {
            utils::lerp(self.medium, self.high, (contrast_level - 0.5) / 0.5)
        } else {
            self.high
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Darker,
    Lighter,
    Nearer,
    RelativeLighter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    Exact,
    Farther,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneDeltaPair {
    pub role_a: Role,
    pub role_b: Role,
    pub delta: f64,
    pub polarity: Polarity,
    pub stay_together: bool,
    pub constraint: Constraint,
}

impl ToneDeltaPair {
    pub const fn new(
        role_a: Role,
        role_b: Role,
        delta: f64,
        polarity: Polarity,
        stay_together: bool,
        constraint: Constraint,
    ) -> ToneDeltaPair {
        ToneDeltaPair {
            role_a,
            role_b,
            delta,
            polarity,
            stay_together,
            constraint,
        }
    }
}

macro_rules! roles {
    ($($variant:ident => $name:literal,)*) => {
        /// A Material color role.
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Role {
            $($variant,)*
        }

        impl Role {
            /// All roles in a fixed order.
            pub const ALL: [Role; [$($name,)*].len()] = [$(Role::$variant,)*];

            /// The token name of the role, for example `on_primary_container`.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Role::$variant => $name,)*
                }
            }
        }
    };
}

roles! {
    Primary => "primary",
    OnPrimary => "on_primary",
    PrimaryContainer => "primary_container",
    OnPrimaryContainer => "on_primary_container",
    Secondary => "secondary",
    OnSecondary => "on_secondary",
    SecondaryContainer => "secondary_container",
    OnSecondaryContainer => "on_secondary_container",
    Tertiary => "tertiary",
    OnTertiary => "on_tertiary",
    TertiaryContainer => "tertiary_container",
    OnTertiaryContainer => "on_tertiary_container",
    Error => "error",
    OnError => "on_error",
    ErrorContainer => "error_container",
    OnErrorContainer => "on_error_container",
    Background => "background",
    OnBackground => "on_background",
    Surface => "surface",
    OnSurface => "on_surface",
    SurfaceVariant => "surface_variant",
    OnSurfaceVariant => "on_surface_variant",
    SurfaceDim => "surface_dim",
    SurfaceBright => "surface_bright",
    SurfaceContainerLowest => "surface_container_lowest",
    SurfaceContainerLow => "surface_container_low",
    SurfaceContainer => "surface_container",
    SurfaceContainerHigh => "surface_container_high",
    SurfaceContainerHighest => "surface_container_highest",
    SurfaceTint => "surface_tint",
    Outline => "outline",
    OutlineVariant => "outline_variant",
    Shadow => "shadow",
    Scrim => "scrim",
    InverseSurface => "inverse_surface",
    InverseOnSurface => "inverse_on_surface",
    InversePrimary => "inverse_primary",
    PrimaryFixed => "primary_fixed",
    PrimaryFixedDim => "primary_fixed_dim",
    OnPrimaryFixed => "on_primary_fixed",
    OnPrimaryFixedVariant => "on_primary_fixed_variant",
    SecondaryFixed => "secondary_fixed",
    SecondaryFixedDim => "secondary_fixed_dim",
    OnSecondaryFixed => "on_secondary_fixed",
    OnSecondaryFixedVariant => "on_secondary_fixed_variant",
    TertiaryFixed => "tertiary_fixed",
    TertiaryFixedDim => "tertiary_fixed_dim",
    OnTertiaryFixed => "on_tertiary_fixed",
    OnTertiaryFixedVariant => "on_tertiary_fixed_variant",
}

type SchemeFn<T> = fn(&DynamicScheme) -> T;

#[derive(Clone, Copy)]
pub struct Def {
    pub palette: SchemeFn<TonalPalette>,
    pub tone: Option<SchemeFn<f64>>,
    pub is_background: bool,
    pub chroma_multiplier: Option<SchemeFn<f64>>,
    pub background: Option<SchemeFn<Option<Role>>>,
    pub second_background: Option<SchemeFn<Option<Role>>>,
    pub contrast_curve: Option<SchemeFn<Option<ContrastCurve>>>,
    pub tone_delta_pair: Option<SchemeFn<Option<ToneDeltaPair>>>,
}

impl Def {
    pub const fn new(palette: SchemeFn<TonalPalette>) -> Def {
        Def {
            palette,
            tone: None,
            is_background: false,
            chroma_multiplier: None,
            background: None,
            second_background: None,
            contrast_curve: None,
            tone_delta_pair: None,
        }
    }

    fn background(&self, s: &DynamicScheme) -> Option<Role> {
        self.background.and_then(|f| f(s))
    }

    fn second_background(&self, s: &DynamicScheme) -> Option<Role> {
        self.second_background.and_then(|f| f(s))
    }

    fn contrast_curve(&self, s: &DynamicScheme) -> Option<ContrastCurve> {
        self.contrast_curve.and_then(|f| f(s))
    }

    fn tone_delta_pair(&self, s: &DynamicScheme) -> Option<ToneDeltaPair> {
        self.tone_delta_pair.and_then(|f| f(s))
    }
}

pub fn def(role: Role, spec: SpecVersion) -> Def {
    match spec {
        SpecVersion::Spec2021 => spec_2021::def(role),
        SpecVersion::Spec2025 => spec_2025::def(role),
    }
}

pub fn tone(role: Role, s: &DynamicScheme) -> f64 {
    let d = def(role, s.spec_version);
    match d.tone {
        Some(f) => f(s),
        None => match d.background(s) {
            Some(bg) => get_tone(bg, s),
            None => 50.0,
        },
    }
}

pub fn get_hct(role: Role, s: &DynamicScheme) -> Hct {
    let d = def(role, s.spec_version);
    let palette = (d.palette)(s);
    let t = get_tone(role, s);
    match s.spec_version {
        SpecVersion::Spec2021 => palette.get_hct(t),
        SpecVersion::Spec2025 => {
            let multiplier = d.chroma_multiplier.map(|f| f(s)).unwrap_or(1.0);
            if multiplier == 1.0 {
                palette.get_hct(t)
            } else {
                Hct::from_int(tone_of(palette.hue, palette.chroma * multiplier, t))
            }
        }
    }
}

pub fn get_tone(role: Role, s: &DynamicScheme) -> f64 {
    match s.spec_version {
        SpecVersion::Spec2021 => get_tone_2021(role, s),
        SpecVersion::Spec2025 => get_tone_2025(role, s),
    }
}

pub fn foreground_tone(bg_tone: f64, ratio: f64) -> f64 {
    let lighter_tone = contrast::lighter_unsafe(bg_tone, ratio);
    let darker_tone = contrast::darker_unsafe(bg_tone, ratio);
    let lighter_ratio = contrast::ratio_of_tones(lighter_tone, bg_tone);
    let darker_ratio = contrast::ratio_of_tones(darker_tone, bg_tone);
    if tone_prefers_light_foreground(bg_tone) {
        let negligible = (lighter_ratio - darker_ratio).abs() < 0.1
            && lighter_ratio < ratio
            && darker_ratio < ratio;
        if lighter_ratio >= ratio || lighter_ratio >= darker_ratio || negligible {
            lighter_tone
        } else {
            darker_tone
        }
    } else if darker_ratio >= ratio || darker_ratio >= lighter_ratio {
        darker_tone
    } else {
        lighter_tone
    }
}

pub fn tone_prefers_light_foreground(tone: f64) -> bool {
    js_round(tone) < 60.0
}

fn second_background_tone(d: &Def, s: &DynamicScheme, answer: f64, desired_ratio: f64) -> f64 {
    let (Some(bg1), Some(bg2)) = (d.background(s), d.second_background(s)) else {
        return answer;
    };
    let bg_tone1 = get_tone(bg1, s);
    let bg_tone2 = get_tone(bg2, s);
    let upper = bg_tone1.max(bg_tone2);
    let lower = bg_tone1.min(bg_tone2);
    if contrast::ratio_of_tones(upper, answer) >= desired_ratio
        && contrast::ratio_of_tones(lower, answer) >= desired_ratio
    {
        return answer;
    }
    let light_option = contrast::lighter(upper, desired_ratio);
    let dark_option = contrast::darker(lower, desired_ratio);
    let mut availables = Vec::with_capacity(2);
    if light_option != -1.0 {
        availables.push(light_option);
    }
    if dark_option != -1.0 {
        availables.push(dark_option);
    }
    if tone_prefers_light_foreground(bg_tone1) || tone_prefers_light_foreground(bg_tone2) {
        return if light_option < 0.0 {
            100.0
        } else {
            light_option
        };
    }
    if availables.len() == 1 {
        return availables[0];
    }
    if dark_option < 0.0 { 0.0 } else { dark_option }
}

fn get_tone_2021(role: Role, s: &DynamicScheme) -> f64 {
    let d = def(role, s.spec_version);
    let decreasing_contrast = s.contrast_level < 0.0;
    if let Some(pair) = d.tone_delta_pair(s) {
        let a_is_nearer = pair.polarity == Polarity::Nearer
            || (pair.polarity == Polarity::Lighter && !s.is_dark)
            || (pair.polarity == Polarity::Darker && s.is_dark);
        let (nearer, farther) = if a_is_nearer {
            (pair.role_a, pair.role_b)
        } else {
            (pair.role_b, pair.role_a)
        };
        let am_nearer = role == nearer;
        let delta = pair.delta;
        let expansion_dir = if s.is_dark { 1.0 } else { -1.0 };
        let mut n_tone = tone(nearer, s);
        let mut f_tone = tone(farther, s);
        let nearer_def = def(nearer, s.spec_version);
        let farther_def = def(farther, s.spec_version);
        if let (Some(bg), Some(n_curve), Some(f_curve)) = (
            d.background(s),
            nearer_def.contrast_curve(s),
            farther_def.contrast_curve(s),
        ) {
            let bg_tone = get_tone(bg, s);
            let n_contrast = n_curve.get(s.contrast_level);
            let f_contrast = f_curve.get(s.contrast_level);
            if contrast::ratio_of_tones(bg_tone, n_tone) < n_contrast {
                n_tone = foreground_tone(bg_tone, n_contrast);
            }
            if contrast::ratio_of_tones(bg_tone, f_tone) < f_contrast {
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
            if decreasing_contrast {
                n_tone = foreground_tone(bg_tone, n_contrast);
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
        }
        if (f_tone - n_tone) * expansion_dir < delta {
            f_tone = utils::clamp_double(0.0, 100.0, n_tone + delta * expansion_dir);
            if (f_tone - n_tone) * expansion_dir < delta {
                n_tone = utils::clamp_double(0.0, 100.0, f_tone - delta * expansion_dir);
            }
        }
        if (50.0..60.0).contains(&n_tone) {
            if expansion_dir > 0.0 {
                n_tone = 60.0;
                f_tone = f_tone.max(n_tone + delta * expansion_dir);
            } else {
                n_tone = 49.0;
                f_tone = f_tone.min(n_tone + delta * expansion_dir);
            }
        } else if (50.0..60.0).contains(&f_tone) {
            if pair.stay_together {
                if expansion_dir > 0.0 {
                    n_tone = 60.0;
                    f_tone = f_tone.max(n_tone + delta * expansion_dir);
                } else {
                    n_tone = 49.0;
                    f_tone = f_tone.min(n_tone + delta * expansion_dir);
                }
            } else if expansion_dir > 0.0 {
                f_tone = 60.0;
            } else {
                f_tone = 49.0;
            }
        }
        return if am_nearer { n_tone } else { f_tone };
    }

    let mut answer = tone(role, s);
    let (Some(bg), Some(curve)) = (d.background(s), d.contrast_curve(s)) else {
        return answer;
    };
    let bg_tone = get_tone(bg, s);
    let desired_ratio = curve.get(s.contrast_level);
    if contrast::ratio_of_tones(bg_tone, answer) < desired_ratio {
        answer = foreground_tone(bg_tone, desired_ratio);
    }
    if decreasing_contrast {
        answer = foreground_tone(bg_tone, desired_ratio);
    }
    if d.is_background && (50.0..60.0).contains(&answer) {
        answer = if contrast::ratio_of_tones(49.0, bg_tone) >= desired_ratio {
            49.0
        } else {
            60.0
        };
    }
    second_background_tone(&d, s, answer, desired_ratio)
}

fn is_fixed_dim(role: Role) -> bool {
    matches!(
        role,
        Role::PrimaryFixedDim | Role::SecondaryFixedDim | Role::TertiaryFixedDim
    )
}

fn get_tone_2025(role: Role, s: &DynamicScheme) -> f64 {
    let d = def(role, s.spec_version);
    if let Some(pair) = d.tone_delta_pair(s) {
        let absolute_delta = if pair.polarity == Polarity::Darker
            || (pair.polarity == Polarity::RelativeLighter && s.is_dark)
        {
            -pair.delta
        } else {
            pair.delta
        };
        let am_role_a = role == pair.role_a;
        let self_role = if am_role_a { pair.role_a } else { pair.role_b };
        let ref_role = if am_role_a { pair.role_b } else { pair.role_a };
        let mut self_tone = tone(self_role, s);
        let ref_tone = get_tone(ref_role, s);
        let relative_delta = absolute_delta * if am_role_a { 1.0 } else { -1.0 };
        match pair.constraint {
            Constraint::Exact => {
                self_tone = utils::clamp_double(0.0, 100.0, ref_tone + relative_delta);
            }
            Constraint::Farther => {
                if relative_delta > 0.0 {
                    self_tone = utils::clamp_double(ref_tone + relative_delta, 100.0, self_tone);
                } else {
                    self_tone = utils::clamp_double(0.0, ref_tone + relative_delta, self_tone);
                }
            }
        }
        if let (Some(bg), Some(curve)) = (d.background(s), d.contrast_curve(s)) {
            let bg_tone = get_tone(bg, s);
            let self_contrast = curve.get(s.contrast_level);
            if !(contrast::ratio_of_tones(bg_tone, self_tone) >= self_contrast
                && s.contrast_level >= 0.0)
            {
                self_tone = foreground_tone(bg_tone, self_contrast);
            }
        }
        if d.is_background && !is_fixed_dim(role) {
            self_tone = if self_tone >= 57.0 {
                utils::clamp_double(65.0, 100.0, self_tone)
            } else {
                utils::clamp_double(0.0, 49.0, self_tone)
            };
        }
        return self_tone;
    }

    let mut answer = tone(role, s);
    let (Some(bg), Some(curve)) = (d.background(s), d.contrast_curve(s)) else {
        return answer;
    };
    let bg_tone = get_tone(bg, s);
    let desired_ratio = curve.get(s.contrast_level);
    if !(contrast::ratio_of_tones(bg_tone, answer) >= desired_ratio && s.contrast_level >= 0.0) {
        answer = foreground_tone(bg_tone, desired_ratio);
    }
    if d.is_background && !is_fixed_dim(role) {
        answer = if answer >= 57.0 {
            utils::clamp_double(65.0, 100.0, answer)
        } else {
            utils::clamp_double(0.0, 49.0, answer)
        };
    }
    second_background_tone(&d, s, answer, desired_ratio)
}

pub fn highest_surface(s: &DynamicScheme) -> Option<Role> {
    Some(if s.is_dark {
        Role::SurfaceBright
    } else {
        Role::SurfaceDim
    })
}
