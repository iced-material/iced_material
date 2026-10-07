// SPDX-License-Identifier: LGPL-3.0-only

use iced::Color;
use material_iced::color::{
    ColorScheme, CustomColor, SchemeOptions, harmonize, matugen, source_from_image,
};

#[test]
fn matugen_round_trip() {
    let light = ColorScheme::default();
    let dark = ColorScheme::new(SchemeOptions {
        dark: true,
        ..SchemeOptions::default()
    });
    let json = matugen::export(&light, &dark, true);
    let (light_in, dark_in) = matugen::import(&json).unwrap();
    assert_eq!(light_in, light);
    assert_eq!(dark_in, dark);
}

#[test]
fn matugen_accepts_plain_strings_and_reports_missing_roles() {
    let light = ColorScheme::default();
    let json = matugen::export(&light, &light, false)
        .replace("{\n        \"color\": ", "")
        .replace("\"\n      }", "\"");
    assert!(!json.contains("\"color\""));
    assert_eq!(matugen::import(&json).unwrap().0, light);
    let err = matugen::import(r#"{"colors": {}}"#).unwrap_err();
    assert_eq!(err, matugen::MatugenError::MissingRole("primary", "light"));
}

#[test]
fn image_source_picks_dominant_chromatic_color() {
    let mut rgba = Vec::new();
    for i in 0..4096 {
        let pixel: [u8; 4] = if i % 4 == 0 {
            [240, 240, 240, 255]
        } else {
            [30, 110, 200, 255]
        };
        rgba.extend_from_slice(&pixel);
    }
    let source = source_from_image(&rgba);
    assert_eq!(source.into_rgba8(), [30, 110, 200, 255]);
}

#[test]
fn custom_color_harmonizes_toward_source() {
    let source = Color::from_rgb8(0x67, 0x50, 0xA4);
    let red = Color::from_rgb8(0xFF, 0x00, 0x00);
    assert_ne!(harmonize(red, source), red);
    let group = CustomColor::new(red, source, true, false);
    assert!(material_iced::color::contrast_ratio(group.on_color, group.color) >= 4.5);
}
