// SPDX-License-Identifier: LGPL-3.0-only

//! Interaction state tokens: state layers, disabled opacities, focus ring and ripple.

use std::time::Duration;

use iced::Color;

/// Opacity of the state layer drawn over a container for each interaction state.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateLayers {
    pub hover: f32,
    pub focus: f32,
    pub pressed: f32,
    pub dragged: f32,
}

impl Default for StateLayers {
    fn default() -> Self {
        StateLayers {
            hover: 0.08,
            focus: 0.12,
            pressed: 0.12,
            dragged: 0.16,
        }
    }
}

/// Opacities used for disabled components, applied to `on_surface`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disabled {
    /// Labels, icons and input text.
    pub content: f32,
    /// Containers.
    pub container: f32,
    /// Outlines.
    pub outline: f32,
    /// Filled text field container.
    pub field_container: f32,
}

impl Default for Disabled {
    fn default() -> Self {
        Disabled {
            content: 0.38,
            container: 0.12,
            outline: 0.12,
            field_container: 0.04,
        }
    }
}

/// Keyboard focus indicator tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusRing {
    /// Resting stroke width.
    pub width: f32,
    /// Stroke width at the peak of the focus animation.
    pub active_width: f32,
    /// Gap between the component edge and the ring.
    pub outward_offset: f32,
    /// Inset of rings drawn inside a component.
    pub inward_offset: f32,
    /// Total duration of the grow and shrink animation.
    pub duration: Duration,
}

impl Default for FocusRing {
    fn default() -> Self {
        FocusRing {
            width: 3.0,
            active_width: 8.0,
            outward_offset: 2.0,
            inward_offset: 0.0,
            duration: Duration::from_millis(600),
        }
    }
}

/// Press ripple tokens, as implemented by Material web.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ripple {
    /// Duration of the grow animation.
    pub press_grow: Duration,
    /// Minimum time the pressed ripple stays before fading out.
    pub minimum_press: Duration,
    /// Initial ripple size as a fraction of the larger container side.
    pub initial_origin_scale: f32,
    /// Extra radius added beyond the container diagonal.
    pub padding: f32,
    /// Minimum width of the soft edge.
    pub soft_edge_minimum_size: f32,
    /// Soft edge width as a fraction of the larger container side.
    pub soft_edge_container_ratio: f32,
    /// Width of the gradient from the solid center to the transparent edge.
    pub soft_edge_falloff: f32,
    /// Minimum solid fraction of the ripple radius.
    pub solid_fraction: f32,
    /// Fade in duration of the pressed ripple.
    pub fade_in: Duration,
    /// Fade out duration of the pressed ripple.
    pub fade_out: Duration,
    /// Transition duration of the hover state layer.
    pub hover_transition: Duration,
}

impl Default for Ripple {
    fn default() -> Self {
        Ripple {
            press_grow: Duration::from_millis(450),
            minimum_press: Duration::from_millis(225),
            initial_origin_scale: 0.2,
            padding: 10.0,
            soft_edge_minimum_size: 75.0,
            soft_edge_container_ratio: 0.35,
            soft_edge_falloff: 70.0,
            solid_fraction: 0.65,
            fade_in: Duration::from_millis(105),
            fade_out: Duration::from_millis(375),
            hover_transition: Duration::from_millis(15),
        }
    }
}

/// Composites `layer` at `opacity` over an opaque or translucent `base` color.
pub fn overlay(base: Color, layer: Color, opacity: f32) -> Color {
    let a = layer.a * opacity;
    let out_a = a + base.a * (1.0 - a);
    if out_a == 0.0 {
        return Color::TRANSPARENT;
    }
    let mix = |l: f32, b: f32| (l * a + b * base.a * (1.0 - a)) / out_a;
    Color {
        r: mix(layer.r, base.r),
        g: mix(layer.g, base.g),
        b: mix(layer.b, base.b),
        a: out_a,
    }
}

/// Returns `color` with its alpha multiplied by `opacity`.
pub fn alpha(color: Color, opacity: f32) -> Color {
    Color {
        a: color.a * opacity,
        ..color
    }
}
