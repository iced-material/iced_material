// SPDX-License-Identifier: LGPL-3.0-only

//! The two shadow layers of a Material elevation level.

use iced::advanced::Renderer as _;
use iced::advanced::renderer::Quad;
use iced::border::{Border, Radius};
use iced::{Color, Rectangle, Renderer, Shadow, Vector};

use crate::elevation::Elevation;
use crate::state::alpha;

/// Draws the key and ambient shadows of `level` under a shape.
///
/// Each layer is a transparent quad with an Iced shadow. Spread grows the
/// quad and its radii, as CSS `box-shadow` does.
pub fn draw(
    renderer: &mut Renderer,
    bounds: Rectangle,
    radius: Radius,
    level: f32,
    color: Color,
    elevation: &Elevation,
) {
    if level <= 0.0 {
        return;
    }
    for layer in elevation.shadows(level) {
        let spread = layer.spread;
        let grow = |r: f32| r + spread;
        renderer.fill_quad(
            Quad {
                bounds: bounds.expand(spread),
                border: Border {
                    radius: Radius {
                        top_left: grow(radius.top_left),
                        top_right: grow(radius.top_right),
                        bottom_right: grow(radius.bottom_right),
                        bottom_left: grow(radius.bottom_left),
                    },
                    ..Border::default()
                },
                shadow: Shadow {
                    color: alpha(color, layer.opacity),
                    offset: Vector::new(0.0, layer.y),
                    blur_radius: layer.blur,
                },
                snap: false,
            },
            Color::TRANSPARENT,
        );
    }
}
