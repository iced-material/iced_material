// SPDX-License-Identifier: LGPL-3.0-only

use std::time::Duration;

use iced::font::{Family, Weight};
use iced::{Color, Size};
use material_iced::color::{
    ColorScheme, Role, SchemeOptions, SpecVersion, Variant, contrast_ratio,
};
use material_iced::elevation::Elevation;
use material_iced::motion::{Easing, Motion};
use material_iced::shape::{Corner, ShapeScale};
use material_iced::state::{self, Disabled, FocusRing, StateLayers};
use material_iced::typography::TypeScale;

#[test]
fn type_scale_matches_tokens() {
    let t = TypeScale::default();
    let expected = [
        (t.display_large, 57.0, 64.0, Weight::Normal, -0.25),
        (t.display_medium, 45.0, 52.0, Weight::Normal, 0.0),
        (t.display_small, 36.0, 44.0, Weight::Normal, 0.0),
        (t.headline_large, 32.0, 40.0, Weight::Normal, 0.0),
        (t.headline_medium, 28.0, 36.0, Weight::Normal, 0.0),
        (t.headline_small, 24.0, 32.0, Weight::Normal, 0.0),
        (t.title_large, 22.0, 28.0, Weight::Normal, 0.0),
        (t.title_medium, 16.0, 24.0, Weight::Medium, 0.15),
        (t.title_small, 14.0, 20.0, Weight::Medium, 0.1),
        (t.body_large, 16.0, 24.0, Weight::Normal, 0.5),
        (t.body_medium, 14.0, 20.0, Weight::Normal, 0.25),
        (t.body_small, 12.0, 16.0, Weight::Normal, 0.4),
        (t.label_large, 14.0, 20.0, Weight::Medium, 0.1),
        (t.label_medium, 12.0, 16.0, Weight::Medium, 0.5),
        (t.label_small, 11.0, 16.0, Weight::Medium, 0.5),
        (t.label_large_prominent, 14.0, 20.0, Weight::Bold, 0.1),
        (t.label_medium_prominent, 12.0, 16.0, Weight::Bold, 0.5),
    ];
    for (style, size, line_height, weight, tracking) in expected {
        assert_eq!(style.size, size);
        assert_eq!(style.line_height, line_height);
        assert_eq!(style.font.weight, weight);
        assert_eq!(style.tracking, tracking);
        assert_eq!(style.font.family, Family::Name("Roboto"));
    }
}

#[test]
fn shape_scale_matches_tokens() {
    let s = ShapeScale::default();
    let size = Size::new(100.0, 40.0);
    let all = |shape: material_iced::shape::Shape| {
        let r = shape.radius(size);
        [r.top_left, r.top_right, r.bottom_right, r.bottom_left]
    };
    assert_eq!(all(s.none), [0.0; 4]);
    assert_eq!(all(s.extra_small), [4.0; 4]);
    assert_eq!(all(s.extra_small_top), [4.0, 4.0, 0.0, 0.0]);
    assert_eq!(all(s.small), [8.0; 4]);
    assert_eq!(all(s.medium), [12.0; 4]);
    assert_eq!(all(s.large), [16.0; 4]);
    assert_eq!(all(s.large_top), [16.0, 16.0, 0.0, 0.0]);
    assert_eq!(all(s.large_start), [16.0, 0.0, 0.0, 16.0]);
    assert_eq!(all(s.large_end), [0.0, 16.0, 16.0, 0.0]);
    assert_eq!(all(s.extra_large), [28.0; 4]);
    assert_eq!(all(s.extra_large_top), [28.0, 28.0, 0.0, 0.0]);
    assert_eq!(all(s.full), [20.0; 4]);
    assert_eq!(s.full.top_left, Corner::Full);
    let mid = s.small.lerp(&s.full, 0.5, size);
    assert_eq!(mid.top_left, 14.0);
}

#[test]
fn elevation_matches_material_web() {
    let e = Elevation::default();
    assert_eq!(e.levels, [0.0, 1.0, 3.0, 6.0, 8.0, 12.0]);
    let expected = [
        ([0.0, 0.0], [0.0, 0.0, 0.0]),
        ([1.0, 2.0], [1.0, 3.0, 1.0]),
        ([1.0, 2.0], [2.0, 6.0, 2.0]),
        ([1.0, 3.0], [4.0, 8.0, 3.0]),
        ([2.0, 3.0], [6.0, 10.0, 4.0]),
        ([4.0, 4.0], [8.0, 12.0, 6.0]),
    ];
    for (level, (key, ambient)) in expected.iter().enumerate() {
        let [k, a] = e.shadows(level as f32);
        assert_eq!(
            [k.y, k.blur, k.spread],
            [key[0], key[1], 0.0],
            "key level {level}"
        );
        assert_eq!([a.y, a.blur, a.spread], *ambient, "ambient level {level}");
        assert_eq!(k.opacity, 0.3);
        assert_eq!(a.opacity, 0.15);
    }
    let [k, a] = e.shadows(2.5);
    assert_eq!(k.blur, 2.5);
    assert_eq!(a.y, 3.0);
}

#[test]
fn durations_match_tokens() {
    let d = Motion::default().duration;
    let ms = |d: Duration| d.as_millis();
    assert_eq!(
        [ms(d.short1), ms(d.short2), ms(d.short3), ms(d.short4)],
        [50, 100, 150, 200]
    );
    assert_eq!(
        [ms(d.medium1), ms(d.medium2), ms(d.medium3), ms(d.medium4)],
        [250, 300, 350, 400]
    );
    assert_eq!(
        [ms(d.long1), ms(d.long2), ms(d.long3), ms(d.long4)],
        [450, 500, 550, 600]
    );
    assert_eq!(
        [
            ms(d.extra_long1),
            ms(d.extra_long2),
            ms(d.extra_long3),
            ms(d.extra_long4)
        ],
        [700, 800, 900, 1000]
    );
}

#[test]
fn easing_matches_reference_values() {
    let e = Motion::default().easing;
    let cases = [
        (
            e.standard,
            [
                (0.1, 0.15625),
                (0.25, 0.607_220_36),
                (0.5, 0.877_833_6),
                (0.75, 0.975_48),
                (0.9, 0.996_459_04),
            ],
        ),
        (
            e.emphasized_decelerate,
            [
                (0.1, 0.621_384_4),
                (0.25, 0.831_529_75),
                (0.5, 0.950_247_5),
                (0.75, 0.990_510_96),
                (0.9, 0.998_666_35),
            ],
        ),
        (
            e.emphasized,
            [
                (0.05, 0.020_608_49),
                (0.1, 0.093_479_87),
                (0.25, 0.772_831_2),
                (0.5, 0.950_612_5),
                (0.9, 0.998_829),
            ],
        ),
    ];
    for (easing, points) in cases {
        for (x, y) in points {
            assert!(
                (easing.apply(x) - y).abs() < 1e-4,
                "{easing:?} at {x}: {} vs {y}",
                easing.apply(x)
            );
        }
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
    }
    assert_eq!(
        e.emphasized_accelerate,
        Easing::CubicBezier(0.3, 0.0, 0.8, 0.15)
    );
    assert_eq!(e.linear.apply(0.3), 0.3);
}

#[test]
fn state_tokens_match() {
    let l = StateLayers::default();
    assert_eq!(
        [l.hover, l.focus, l.pressed, l.dragged],
        [0.08, 0.12, 0.12, 0.16]
    );
    let d = Disabled::default();
    assert_eq!(
        [d.content, d.container, d.outline, d.field_container],
        [0.38, 0.12, 0.12, 0.04]
    );
    let f = FocusRing::default();
    assert_eq!([f.width, f.active_width, f.outward_offset], [3.0, 8.0, 2.0]);
    let over = state::overlay(Color::WHITE, Color::BLACK, 0.12);
    assert!((over.r - 0.88).abs() < 1e-6 && over.a == 1.0);
}

fn schemes() -> Vec<ColorScheme> {
    let seeds = [
        0x6750A4u32,
        0xB3261E,
        0x0B57D0,
        0x1B6D00,
        0xFFDE3F,
        0x00796B,
        0x808080,
    ];
    let mut out = Vec::new();
    for seed in seeds {
        for variant in Variant::ALL {
            for spec in [SpecVersion::Spec2021, SpecVersion::Spec2025] {
                for dark in [false, true] {
                    for contrast in [0.0, 0.5, 1.0] {
                        out.push(ColorScheme::new(SchemeOptions {
                            source: Color::from_rgb8(
                                (seed >> 16) as u8,
                                (seed >> 8) as u8,
                                seed as u8,
                            ),
                            variant,
                            dark,
                            contrast,
                            spec,
                        }));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn text_roles_reach_wcag_contrast() {
    let pairs = [
        (Role::OnPrimary, Role::Primary),
        (Role::OnPrimaryContainer, Role::PrimaryContainer),
        (Role::OnSecondary, Role::Secondary),
        (Role::OnSecondaryContainer, Role::SecondaryContainer),
        (Role::OnTertiary, Role::Tertiary),
        (Role::OnTertiaryContainer, Role::TertiaryContainer),
        (Role::OnError, Role::Error),
        (Role::OnErrorContainer, Role::ErrorContainer),
        (Role::OnSurface, Role::Surface),
        (Role::OnSurface, Role::SurfaceContainerLowest),
        (Role::OnSurface, Role::SurfaceContainer),
        (Role::OnSurface, Role::SurfaceContainerHighest),
        (Role::OnSurfaceVariant, Role::Surface),
        (Role::OnSurfaceVariant, Role::SurfaceContainerHighest),
        (Role::InverseOnSurface, Role::InverseSurface),
        (Role::OnPrimaryFixed, Role::PrimaryFixed),
        (Role::OnPrimaryFixedVariant, Role::PrimaryFixed),
    ];
    let mut failures = Vec::new();
    for scheme in schemes() {
        for (text, background) in pairs {
            let ratio = contrast_ratio(scheme.role(text), scheme.role(background));
            if ratio < 4.5 {
                failures.push(format!(
                    "{} on {}: {ratio:.2}",
                    text.name(),
                    background.name()
                ));
            }
        }
        for (name, group) in [("success", scheme.success), ("warning", scheme.warning)] {
            for (text, background) in [
                (group.on_color, group.color),
                (group.on_color_container, group.color_container),
            ] {
                let ratio = contrast_ratio(text, background);
                if ratio < 4.5 {
                    failures.push(format!("{name}: {ratio:.2}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures: {:?}",
        failures.len(),
        &failures[..failures.len().min(20)]
    );
}

#[test]
fn outline_reaches_non_text_contrast() {
    let mut failures = Vec::new();
    for scheme in schemes() {
        for background in [Role::Surface, Role::SurfaceContainerHighest] {
            let ratio = contrast_ratio(scheme.outline, scheme.role(background));
            if ratio < 3.0 {
                failures.push(format!("outline on {}: {ratio:.2}", background.name()));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures: {:?}",
        failures.len(),
        &failures[..failures.len().min(20)]
    );
}

#[test]
fn non_text_roles_reach_three_to_one_on_the_surface() {
    let roles = [
        Role::Outline,
        Role::Primary,
        Role::Secondary,
        Role::Error,
        Role::OnSurfaceVariant,
    ];
    let mut failures: std::collections::BTreeMap<String, (usize, f64)> = Default::default();
    for scheme in schemes() {
        for role in roles {
            let ratio = contrast_ratio(scheme.role(role), scheme.role(Role::Surface));
            if ratio < 3.0 {
                let entry = failures
                    .entry(role.name().to_string())
                    .or_insert((0, 99.0f64));
                entry.0 += 1;
                entry.1 = entry.1.min(ratio);
            }
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}
