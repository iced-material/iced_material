// SPDX-License-Identifier: LGPL-3.0-only

use std::fmt;

use iced::Color;
use serde_json::{Map, Value, json};

use super::dynamic::Role;
use super::scheme::ColorScheme;

/// Error returned when a Matugen JSON document cannot be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatugenError {
    /// The document is not valid JSON.
    Json(String),
    /// A color role is missing for the given mode.
    MissingRole(&'static str, &'static str),
    /// A color value is not a `#RRGGBB` or `#RRGGBBAA` string.
    InvalidColor(&'static str, String),
}

impl fmt::Display for MatugenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatugenError::Json(message) => write!(f, "invalid JSON: {message}"),
            MatugenError::MissingRole(role, mode) => {
                write!(f, "missing color role `{role}` for mode `{mode}`")
            }
            MatugenError::InvalidColor(role, value) => {
                write!(f, "invalid color `{value}` for role `{role}`")
            }
        }
    }
}

impl std::error::Error for MatugenError {}

fn parse_hex(hex: &str) -> Option<Color> {
    let digits = hex.strip_prefix('#')?;
    let byte = |i: usize| u8::from_str_radix(digits.get(i..i + 2)?, 16).ok();
    match digits.len() {
        6 => Some(Color::from_rgb8(byte(0)?, byte(2)?, byte(4)?)),
        8 => Some(Color::from_rgba8(
            byte(0)?,
            byte(2)?,
            byte(4)?,
            byte(6)? as f32 / 255.0,
        )),
        _ => None,
    }
}

fn format_hex(color: Color) -> String {
    let [r, g, b, a] = color.into_rgba8();
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

fn read_mode(colors: &Map<String, Value>, mode: &'static str) -> Result<ColorScheme, MatugenError> {
    let mut scheme = ColorScheme::default();
    for role in Role::ALL {
        let entry = colors
            .get(role.name())
            .and_then(|r| r.get(mode))
            .ok_or(MatugenError::MissingRole(role.name(), mode))?;
        let text = match entry {
            Value::String(text) => text.as_str(),
            Value::Object(object) => object
                .get("color")
                .and_then(Value::as_str)
                .ok_or(MatugenError::MissingRole(role.name(), mode))?,
            _ => return Err(MatugenError::MissingRole(role.name(), mode)),
        };
        let color = parse_hex(text)
            .ok_or_else(|| MatugenError::InvalidColor(role.name(), text.to_string()))?;
        scheme.set_role(role, color);
    }
    scheme.derive_custom_colors(mode == "dark");
    Ok(scheme)
}

/// Reads the light and dark schemes from Matugen JSON output.
///
/// The expected layout is the one Matugen 4 prints with `--json hex`:
/// `colors.<role>.<light|dark>.color`. Plain strings in place of the
/// `{ "color": ... }` objects are accepted too. Success and warning roles
/// are derived from the imported `primary`.
pub fn import(json: &str) -> Result<(ColorScheme, ColorScheme), MatugenError> {
    let document: Value =
        serde_json::from_str(json).map_err(|e| MatugenError::Json(e.to_string()))?;
    let colors = document
        .get("colors")
        .and_then(Value::as_object)
        .ok_or(MatugenError::Json("missing `colors` object".to_string()))?;
    Ok((read_mode(colors, "light")?, read_mode(colors, "dark")?))
}

/// Writes the light and dark schemes in the Matugen 4 JSON layout.
pub fn export(light: &ColorScheme, dark: &ColorScheme, dark_is_default: bool) -> String {
    let mut colors = Map::new();
    for role in Role::ALL {
        let light_hex = format_hex(light.role(role));
        let dark_hex = format_hex(dark.role(role));
        let default_hex = if dark_is_default {
            dark_hex.clone()
        } else {
            light_hex.clone()
        };
        colors.insert(
            role.name().to_string(),
            json!({
                "light": { "color": light_hex },
                "dark": { "color": dark_hex },
                "default": { "color": default_hex },
            }),
        );
    }
    serde_json::to_string_pretty(&json!({ "colors": colors })).unwrap_or_default()
}
