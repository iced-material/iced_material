// SPDX-License-Identifier: LGPL-3.0-only

//! The container, state layers and focus ring shared by pressable components.

use std::cell::Cell;

use iced::advanced::Shell;
use iced::border::Radius;
use iced::mouse::Cursor;
use iced::time::Instant;
use iced::{Color, Event, Rectangle, Renderer, window};

use crate::draw::{focus_ring, shadow, surface};
use crate::interaction::{Activation, Interaction, Tokens};
use crate::motion::{Transition, Tween};
use crate::theme::Theme;

/// Interaction status of a pressable component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Enabled and idle.
    Active,
    /// The pointer is over the component.
    Hovered,
    /// The component has keyboard focus.
    Focused,
    /// The component is pressed.
    Pressed,
    /// The component is being dragged.
    Dragged,
    /// The component is disabled.
    Disabled,
}

const STATUSES: usize = 6;

impl Status {
    fn index(self) -> usize {
        self as usize
    }
}

/// The appearance of a pressable component in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Container color. Transparent for components without a container.
    pub container: Color,
    /// Outline color.
    pub outline: Color,
    /// Outline width. Zero draws no outline.
    pub outline_width: f32,
    /// Label color.
    pub label: Color,
    /// Icon color.
    pub icon: Color,
    /// Color of the hover state layer.
    pub hover_layer: Color,
    /// Color of the press ripple.
    pub pressed_layer: Color,
    /// Elevation level from 0 to 5.
    pub elevation: f32,
    /// Shadow color.
    pub shadow: Color,
}

impl Style {
    /// A style with no container, outline or elevation, using one color for both state layers.
    pub fn content(theme: &Theme, label: Color, icon: Color, state_layer: Color) -> Style {
        Style {
            container: Color::TRANSPARENT,
            outline: Color::TRANSPARENT,
            outline_width: 0.0,
            label,
            icon,
            hover_layer: state_layer,
            pressed_layer: state_layer,
            elevation: 0.0,
            shadow: theme.colors.shadow,
        }
    }
}

pub(crate) struct State {
    pub interaction: Interaction,
    pub elevation: Cell<Tween>,
    pub elevations: Cell<Option<[f32; STATUSES]>>,
    pub status: Status,
}

impl Default for State {
    fn default() -> Self {
        State {
            interaction: Interaction::default(),
            elevation: Cell::new(Tween::new(0.0)),
            elevations: Cell::new(None),
            status: Status::Active,
        }
    }
}

impl State {
    pub fn status(&self, enabled: bool, dragged: bool) -> Status {
        let i = &self.interaction;
        if !enabled {
            Status::Disabled
        } else if dragged {
            Status::Dragged
        } else if i.pressed {
            Status::Pressed
        } else if i.focus.visible {
            Status::Focused
        } else if i.hovered {
            Status::Hovered
        } else {
            Status::Active
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update<Message>(
        &mut self,
        event: &Event,
        hit: Rectangle,
        cursor: Cursor,
        enabled: bool,
        dragged: bool,
        tokens: &Tokens,
        shell: &mut Shell<'_, Message>,
    ) -> Option<Activation> {
        let activation = self
            .interaction
            .update(event, hit, cursor, enabled, tokens, shell);
        let status = self.status(enabled, dragged);
        if status != self.status {
            self.status = status;
            if let Some(elevations) = self.elevations.get() {
                let mut tween = self.elevation.get();
                tween.go(
                    elevations[status.index()],
                    Transition {
                        duration: tokens.elevation,
                        easing: tokens.emphasized,
                    },
                    self.interaction.now,
                );
                self.elevation.set(tween);
            }
            shell.request_redraw();
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let mut tween = self.elevation.get();
            if tween.tick(*now) {
                shell.request_redraw();
            }
            self.elevation.set(tween);
        }
        activation
    }

    pub fn now(&self) -> Option<Instant> {
        self.interaction.now
    }

    pub fn style<F: Fn(Status) -> Style>(
        &self,
        enabled: bool,
        dragged: bool,
        style: F,
    ) -> (Style, f32) {
        let all = [
            Status::Active,
            Status::Hovered,
            Status::Focused,
            Status::Pressed,
            Status::Dragged,
            Status::Disabled,
        ]
        .map(|s| style(s).elevation);
        self.elevations.set(Some(all));
        let current = style(self.status(enabled, dragged));
        let tween = self.elevation.get();
        if tween.target() != current.elevation {
            self.elevation.set(Tween::new(current.elevation));
            return (current, current.elevation);
        }
        (current, tween.value(self.now()))
    }
}

pub(crate) fn draw_container(
    renderer: &mut Renderer,
    theme: &Theme,
    bounds: Rectangle,
    radius: Radius,
    style: &Style,
    elevation: f32,
    state: &State,
) {
    shadow::draw(
        renderer,
        bounds,
        radius,
        elevation,
        style.shadow,
        &theme.elevation,
    );
    surface::fill(renderer, bounds, radius, style.container);
    let now = state.now();
    if state.status != Status::Disabled {
        surface::state_layer(
            renderer,
            bounds,
            radius,
            style.hover_layer,
            state.interaction.hover.value(now),
        );
        state
            .interaction
            .ripple
            .draw(renderer, bounds, radius, style.pressed_layer, theme, now);
    }
    surface::outline(renderer, bounds, radius, style.outline_width, style.outline);
}

pub(crate) fn draw_focus(
    renderer: &mut Renderer,
    theme: &Theme,
    bounds: Rectangle,
    radius: Radius,
    state: &State,
    tokens: &Tokens,
) {
    let width = state.interaction.focus_ring_width(tokens);
    focus_ring::draw(
        renderer,
        bounds,
        radius,
        theme.focus_ring.outward_offset,
        width,
        theme.colors.secondary,
    );
}

pub(crate) fn hit_area(bounds: Rectangle, minimum: f32) -> Rectangle {
    let dx = (minimum - bounds.width).max(0.0) / 2.0;
    let dy = (minimum - bounds.height).max(0.0) / 2.0;
    Rectangle {
        x: bounds.x - dx,
        y: bounds.y - dy,
        width: bounds.width + dx * 2.0,
        height: bounds.height + dy * 2.0,
    }
}
