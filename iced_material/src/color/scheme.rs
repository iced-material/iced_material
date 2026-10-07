// SPDX-License-Identifier: LGPL-3.0-only

use iced::Color;

use super::custom::{CustomColor, SUCCESS_SEED, WARNING_SEED};
use super::dynamic::{DynamicScheme, Role, SpecVersion, Variant};
use super::hct::Hct;

/// Inputs for generating a [`ColorScheme`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchemeOptions {
    /// Source color the scheme is derived from.
    pub source: Color,
    /// Generation algorithm.
    pub variant: Variant,
    /// Generate the dark scheme instead of the light one.
    pub dark: bool,
    /// Contrast level from -1.0 to 1.0. 0.0 is standard, 0.5 medium, 1.0 high.
    pub contrast: f64,
    /// Color specification version.
    pub spec: SpecVersion,
}

impl Default for SchemeOptions {
    fn default() -> Self {
        SchemeOptions {
            source: Color::from_rgb8(0x67, 0x50, 0xA4),
            variant: Variant::TonalSpot,
            dark: false,
            contrast: 0.0,
            spec: SpecVersion::Spec2021,
        }
    }
}

pub(crate) fn argb(color: Color) -> u32 {
    let [r, g, b, _] = color.into_rgba8();
    0xFF00_0000 | (r as u32) << 16 | (g as u32) << 8 | b as u32
}

pub(crate) fn color(argb: u32) -> Color {
    Color::from_rgb8((argb >> 16) as u8, (argb >> 8) as u8, argb as u8)
}

macro_rules! color_scheme {
    ($($field:ident => $role:ident,)*) => {
        /// The resolved colors of all Material color roles.
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct ColorScheme {
            $(pub $field: Color,)*
            /// Success roles, harmonized toward `primary`.
            pub success: CustomColor,
            /// Warning roles, harmonized toward `primary`.
            pub warning: CustomColor,
        }

        impl ColorScheme {
            /// Returns the color of a role.
            pub fn role(&self, role: Role) -> Color {
                match role {
                    $(Role::$role => self.$field,)*
                }
            }

            /// Replaces the color of a role.
            pub fn set_role(&mut self, role: Role, color: Color) {
                match role {
                    $(Role::$role => self.$field = color,)*
                }
            }

            fn from_dynamic(scheme: &DynamicScheme) -> ColorScheme {
                let primary = color(scheme.argb(Role::Primary));
                ColorScheme {
                    $($field: color(scheme.argb(Role::$role)),)*
                    success: CustomColor::new(SUCCESS_SEED, primary, true, scheme.is_dark),
                    warning: CustomColor::new(WARNING_SEED, primary, true, scheme.is_dark),
                }
            }

            /// Interpolates every role between two schemes in sRGB space.
            pub fn mix(&self, other: &ColorScheme, amount: f32) -> ColorScheme {
                ColorScheme {
                    $($field: mix(self.$field, other.$field, amount),)*
                    success: self.success.mix(&other.success, amount, mix),
                    warning: self.warning.mix(&other.warning, amount, mix),
                }
            }
        }
    };
}

color_scheme! {
    primary => Primary,
    on_primary => OnPrimary,
    primary_container => PrimaryContainer,
    on_primary_container => OnPrimaryContainer,
    secondary => Secondary,
    on_secondary => OnSecondary,
    secondary_container => SecondaryContainer,
    on_secondary_container => OnSecondaryContainer,
    tertiary => Tertiary,
    on_tertiary => OnTertiary,
    tertiary_container => TertiaryContainer,
    on_tertiary_container => OnTertiaryContainer,
    error => Error,
    on_error => OnError,
    error_container => ErrorContainer,
    on_error_container => OnErrorContainer,
    background => Background,
    on_background => OnBackground,
    surface => Surface,
    on_surface => OnSurface,
    surface_variant => SurfaceVariant,
    on_surface_variant => OnSurfaceVariant,
    surface_dim => SurfaceDim,
    surface_bright => SurfaceBright,
    surface_container_lowest => SurfaceContainerLowest,
    surface_container_low => SurfaceContainerLow,
    surface_container => SurfaceContainer,
    surface_container_high => SurfaceContainerHigh,
    surface_container_highest => SurfaceContainerHighest,
    surface_tint => SurfaceTint,
    outline => Outline,
    outline_variant => OutlineVariant,
    shadow => Shadow,
    scrim => Scrim,
    inverse_surface => InverseSurface,
    inverse_on_surface => InverseOnSurface,
    inverse_primary => InversePrimary,
    primary_fixed => PrimaryFixed,
    primary_fixed_dim => PrimaryFixedDim,
    on_primary_fixed => OnPrimaryFixed,
    on_primary_fixed_variant => OnPrimaryFixedVariant,
    secondary_fixed => SecondaryFixed,
    secondary_fixed_dim => SecondaryFixedDim,
    on_secondary_fixed => OnSecondaryFixed,
    on_secondary_fixed_variant => OnSecondaryFixedVariant,
    tertiary_fixed => TertiaryFixed,
    tertiary_fixed_dim => TertiaryFixedDim,
    on_tertiary_fixed => OnTertiaryFixed,
    on_tertiary_fixed_variant => OnTertiaryFixedVariant,
}

fn mix(a: Color, b: Color, amount: f32) -> Color {
    Color {
        r: a.r + (b.r - a.r) * amount,
        g: a.g + (b.g - a.g) * amount,
        b: a.b + (b.b - a.b) * amount,
        a: a.a + (b.a - a.a) * amount,
    }
}

impl ColorScheme {
    /// Generates a scheme with the Material dynamic color algorithm.
    pub fn new(options: SchemeOptions) -> ColorScheme {
        let scheme = DynamicScheme::new(
            Hct::from_int(argb(options.source)),
            options.variant,
            options.dark,
            options.contrast,
            options.spec,
        );
        ColorScheme::from_dynamic(&scheme)
    }
}

impl ColorScheme {
    pub(crate) fn derive_custom_colors(&mut self, dark: bool) {
        self.success = CustomColor::new(SUCCESS_SEED, self.primary, true, dark);
        self.warning = CustomColor::new(WARNING_SEED, self.primary, true, dark);
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme::new(SchemeOptions::default())
    }
}
