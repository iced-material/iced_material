// SPDX-License-Identifier: LGPL-3.0-only

//! Filled and outlined containers with per-corner radii.

use iced::advanced::Renderer as _;
use iced::advanced::renderer::Quad;
use iced::border::{Border, Radius};
use iced::{Color, Rectangle, Renderer, Shadow};

use crate::state::alpha;

/// Fills `bounds` with `color`.
pub fn fill(renderer: &mut Renderer, bounds: Rectangle, radius: Radius, color: Color) {
    if color.a == 0.0 {
        return;
    }
    renderer.fill_quad(
        Quad {
            bounds,
            border: Border {
                radius,
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: true,
        },
        color,
    );
}

/// Strokes the inside edge of `bounds`.
pub fn outline(
    renderer: &mut Renderer,
    bounds: Rectangle,
    radius: Radius,
    width: f32,
    color: Color,
) {
    if color.a == 0.0 || width == 0.0 {
        return;
    }
    renderer.fill_quad(
        Quad {
            bounds,
            border: Border {
                color,
                width,
                radius,
            },
            shadow: Shadow::default(),
            snap: true,
        },
        Color::TRANSPARENT,
    );
}

/// Draws a state layer of `color` at `opacity` over `bounds`.
pub fn state_layer(
    renderer: &mut Renderer,
    bounds: Rectangle,
    radius: Radius,
    color: Color,
    opacity: f32,
) {
    if opacity > 0.0 {
        fill(renderer, bounds, radius, alpha(color, opacity));
    }
}
