// SPDX-License-Identifier: LGPL-3.0-only

//! The keyboard focus indicator.

use std::time::Duration;

use iced::advanced::Renderer as _;
use iced::advanced::renderer::Quad;
use iced::border::{Border, Radius};
use iced::{Color, Rectangle, Renderer, Shadow};

use crate::motion::Easing;
use crate::state::FocusRing;

/// Ring width `elapsed` after focus became visible.
///
/// The ring grows to the active width in the first quarter of the duration
/// and shrinks to the resting width in the rest, as Material web does.
pub fn width(tokens: &FocusRing, easing: &Easing, elapsed: Duration) -> f32 {
    let grow = tokens.duration.mul_f32(0.25);
    if elapsed < grow {
        tokens.active_width * easing.apply(elapsed.as_secs_f32() / grow.as_secs_f32())
    } else if elapsed < tokens.duration {
        let shrink = tokens.duration - grow;
        let p = easing.apply((elapsed - grow).as_secs_f32() / shrink.as_secs_f32());
        tokens.active_width + (tokens.width - tokens.active_width) * p
    } else {
        tokens.width
    }
}

/// Draws an outward ring of `width` around a shape with corner `radius`.
pub fn draw(
    renderer: &mut Renderer,
    bounds: Rectangle,
    radius: Radius,
    offset: f32,
    width: f32,
    color: Color,
) {
    if width <= 0.0 {
        return;
    }
    let grow = offset + width;
    let r = |value: f32| value + grow;
    renderer.fill_quad(
        Quad {
            bounds: bounds.expand(grow),
            border: Border {
                color,
                width,
                radius: Radius {
                    top_left: r(radius.top_left),
                    top_right: r(radius.top_right),
                    bottom_right: r(radius.bottom_right),
                    bottom_left: r(radius.bottom_left),
                },
            },
            shadow: Shadow::default(),
            snap: false,
        },
        Color::TRANSPARENT,
    );
}
