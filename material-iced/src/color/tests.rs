// SPDX-License-Identifier: LGPL-3.0-only

use super::dynamic::{DynamicScheme, Role, SpecVersion, Variant};
use super::hct::{Hct, solve_to_int};
use super::palette::TonalPalette;
use super::{blend, contrast, quantize, score};

fn data(name: &str) -> String {
    let path = format!("{}/tests/data/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn hex(text: &str) -> u32 {
    0xFF00_0000 | u32::from_str_radix(text, 16).unwrap()
}

fn variant(name: &str) -> Variant {
    match name {
        "monochrome" => Variant::Monochrome,
        "neutral" => Variant::Neutral,
        "tonal_spot" => Variant::TonalSpot,
        "vibrant" => Variant::Vibrant,
        "expressive" => Variant::Expressive,
        "fidelity" => Variant::Fidelity,
        "content" => Variant::Content,
        "rainbow" => Variant::Rainbow,
        "fruit_salad" => Variant::FruitSalad,
        other => panic!("unknown variant {other}"),
    }
}

#[test]
fn schemes_match_reference() {
    let text = data("schemes.txt");
    let mut failures = Vec::new();
    let mut checked = 0;
    for line in text.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        let spec = if fields[2] == "2025" {
            SpecVersion::Spec2025
        } else {
            SpecVersion::Spec2021
        };
        let scheme = DynamicScheme::new(
            Hct::from_int(hex(fields[0])),
            variant(fields[1]),
            fields[3] == "1",
            fields[4].parse().unwrap(),
            spec,
        );
        for (role, expected) in Role::ALL.iter().zip(&fields[5..]) {
            checked += 1;
            let actual = scheme.argb(*role) & 0xFF_FFFF;
            if actual != hex(expected) & 0xFF_FFFF {
                failures.push(format!(
                    "{} {}: expected {expected} got {actual:06x}",
                    fields[..5].join(" "),
                    role.name()
                ));
            }
        }
    }
    assert!(checked > 60_000);
    assert!(
        failures.is_empty(),
        "{} of {checked} mismatches, first: {:?}",
        failures.len(),
        &failures[..failures.len().min(10)]
    );
}

#[test]
fn hct_matches_reference() {
    for line in data("hct.txt").lines() {
        let f: Vec<&str> = line.split(' ').collect();
        let hct = Hct::from_int(hex(f[0]));
        let expected: Vec<f64> = f[1..].iter().map(|v| v.parse().unwrap()).collect();
        assert!((hct.hue() - expected[0]).abs() < 1e-9, "{line}");
        assert!((hct.chroma() - expected[1]).abs() < 1e-9, "{line}");
        assert!((hct.tone() - expected[2]).abs() < 1e-9, "{line}");
    }
}

#[test]
fn solver_matches_reference() {
    for line in data("solver.txt").lines() {
        let f: Vec<&str> = line.split(' ').collect();
        let v: Vec<f64> = f[..3].iter().map(|v| v.parse().unwrap()).collect();
        assert_eq!(solve_to_int(v[0], v[1], v[2]), hex(f[3]), "{line}");
    }
}

#[test]
fn palettes_match_reference() {
    for line in data("palettes.txt").lines() {
        let f: Vec<&str> = line.split(' ').collect();
        let source = Hct::from_int(hex(f[0]));
        let palette = TonalPalette::from_hue_and_chroma(source.hue(), source.chroma());
        assert_eq!(palette.key_color.to_int(), hex(f[1]), "key color {line}");
        for (tone, expected) in f[2..].iter().enumerate() {
            assert_eq!(
                palette.tone(tone as f64),
                hex(expected),
                "{} tone {tone}",
                f[0]
            );
        }
    }
}

#[test]
fn harmonize_matches_reference() {
    for line in data("harmonize.txt").lines() {
        let f: Vec<&str> = line.split(' ').collect();
        assert_eq!(blend::harmonize(hex(f[0]), hex(f[1])), hex(f[2]), "{line}");
    }
}

#[test]
fn contrast_matches_reference() {
    for line in data("contrast.txt").lines() {
        let v: Vec<f64> = line.split(' ').map(|v| v.parse().unwrap()).collect();
        assert!(
            (contrast::ratio_of_tones(v[0], v[1]) - v[2]).abs() < 1e-12,
            "{line}"
        );
        assert!(
            (contrast::lighter(v[0], 4.5) - v[3]).abs() < 1e-12,
            "{line}"
        );
        assert!((contrast::darker(v[0], 4.5) - v[4]).abs() < 1e-12, "{line}");
    }
}

#[test]
fn quantize_and_score_match_reference() {
    for line in data("quantize.txt").lines() {
        let parts: Vec<&str> = line.split('|').collect();
        let pixels: Vec<u32> = parts[0].split_whitespace().map(hex).collect();
        let expected_clusters: Vec<(u32, u32)> = parts[1]
            .split_whitespace()
            .map(|e| {
                let (color, count) = e.split_once(':').unwrap();
                (hex(color), count.parse().unwrap())
            })
            .collect();
        let expected_ranked: Vec<u32> = parts[2].split_whitespace().map(hex).collect();
        let clusters = quantize::celebi(&pixels, 128);
        assert_eq!(clusters, expected_clusters);
        assert_eq!(
            score::score(&clusters, 4, score::FALLBACK_COLOR, true),
            expected_ranked
        );
    }
}

#[test]
fn success_and_warning_match_reference() {
    use super::scheme::{ColorScheme, SchemeOptions};

    let rgb = |text: &str| {
        let v = hex(text);
        [(v >> 16) as u8, (v >> 8) as u8, v as u8, 255]
    };
    let mut checked = 0;
    for line in data("custom.txt").lines() {
        let f: Vec<&str> = line.split(' ').collect();
        let source = rgb(f[0]);
        let scheme = ColorScheme::new(SchemeOptions {
            source: iced::Color::from_rgb8(source[0], source[1], source[2]),
            variant: variant(f[1]),
            dark: f[3] == "1",
            contrast: f[4].parse().unwrap(),
            spec: if f[2] == "2025" {
                SpecVersion::Spec2025
            } else {
                SpecVersion::Spec2021
            },
        });
        assert_eq!(scheme.primary.into_rgba8(), rgb(f[5]), "{line}");
        for (group, start) in [(scheme.success, 6), (scheme.warning, 10)] {
            let actual = [
                group.color,
                group.on_color,
                group.color_container,
                group.on_color_container,
            ]
            .map(|c| c.into_rgba8());
            let expected = [
                rgb(f[start]),
                rgb(f[start + 1]),
                rgb(f[start + 2]),
                rgb(f[start + 3]),
            ];
            assert_eq!(actual, expected, "{line}");
            checked += 4;
        }
    }
    assert_eq!(checked, 832 * 8);
}
