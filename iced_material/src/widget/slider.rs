// SPDX-License-Identifier: LGPL-3.0-only

//! Continuous, discrete and range sliders with an optional value label.

use std::ops::RangeInclusive;

use iced::advanced::Renderer as _;
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::mouse::{self, Cursor, ScrollDelta};
use iced::widget::canvas::{Fill, Frame, Path, Style as CanvasStyle};
use iced::{
    Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, Vector, window,
};

use crate::Element;
use crate::draw::text::Label;
use crate::draw::{focus_ring, shadow, surface};
use crate::interaction::{Interaction, Tokens};
use crate::motion::{Transition, Tween};
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::Status;

/// Slider dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of the widget, which is also the pointer target.
    pub target_height: f32,
    /// Minimum width.
    pub min_width: f32,
    /// Height of the active and inactive track.
    pub track_height: f32,
    /// Diameter of the handle.
    pub handle_size: f32,
    /// Diameter of the state layer.
    pub state_layer_size: f32,
    /// Elevation level of the handle.
    pub handle_elevation: f32,
    /// Diameter of a tick mark.
    pub tick_size: f32,
    /// Height and minimum width of the value label container.
    pub label_size: f32,
    /// Padding of the value label.
    pub label_padding: f32,
    /// Outline width of a handle that overlaps the other one.
    pub overlap_outline_width: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            target_height: 48.0,
            min_width: 200.0,
            track_height: 4.0,
            handle_size: 20.0,
            state_layer_size: 40.0,
            handle_elevation: 1.0,
            tick_size: 2.0,
            label_size: 28.0,
            label_padding: 4.0,
            overlap_outline_width: 1.0,
        }
    }
}

/// The appearance of a slider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Active track color.
    pub active_track: Color,
    /// Inactive track color.
    pub inactive_track: Color,
    /// Handle color.
    pub handle: Color,
    /// Tick color on the active track.
    pub active_tick: Color,
    /// Tick color on the inactive track.
    pub inactive_tick: Color,
    /// Hover state layer and press ripple color.
    pub state_layer: Color,
    /// Value label container color.
    pub label_container: Color,
    /// Value label text color.
    pub label_text: Color,
    /// Outline of the handle on top when handles overlap.
    pub overlap_outline: Color,
    /// Handle elevation level.
    pub elevation: f32,
}

/// The appearance catalog of sliders.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class. Only `Status::Disabled` and `Status::Active` are passed.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The baseline slider style.
pub fn style(theme: &Theme, status: Status) -> Style {
    let c = &theme.colors;
    if status == Status::Disabled {
        let content = alpha(c.on_surface, theme.disabled.content);
        return Style {
            active_track: content,
            inactive_track: alpha(c.on_surface, theme.disabled.container),
            handle: content,
            active_tick: content,
            inactive_tick: content,
            state_layer: c.primary,
            label_container: c.primary,
            label_text: c.on_primary,
            overlap_outline: c.on_primary,
            elevation: 0.0,
        };
    }
    Style {
        active_track: c.primary,
        inactive_track: c.surface_container_highest,
        handle: c.primary,
        active_tick: c.on_primary,
        inactive_tick: c.on_surface_variant,
        state_layer: c.primary,
        label_container: c.primary,
        label_text: c.on_primary,
        overlap_outline: c.on_primary,
        elevation: 1.0,
    }
}

enum Change<'a, Message> {
    Single(Box<dyn Fn(f32) -> Message + 'a>),
    Range(Box<dyn Fn(f32, f32) -> Message + 'a>),
}

/// A Material slider.
pub struct Slider<'a, Message> {
    min: f32,
    max: f32,
    values: [f32; 2],
    count: usize,
    step: Option<f32>,
    ticks: bool,
    labeled: bool,
    format: Option<Box<dyn Fn(f32) -> String + 'a>>,
    width: Length,
    on_change: Option<Change<'a, Message>>,
    metrics: Metrics,
    tokens: Tokens,
    label_transition: Transition,
    type_style: TypeStyle,
    class: StyleFn<'a>,
}

fn build<'a, Message>(
    theme: &Theme,
    range: RangeInclusive<f32>,
    values: [f32; 2],
    count: usize,
) -> Slider<'a, Message> {
    Slider {
        min: *range.start(),
        max: *range.end(),
        values,
        count,
        step: None,
        ticks: false,
        labeled: false,
        format: None,
        width: Length::Fill,
        on_change: None,
        metrics: theme.components.slider,
        tokens: Tokens::new(theme),
        label_transition: Transition {
            duration: theme.motion.duration.short2,
            easing: theme.motion.easing.emphasized,
        },
        type_style: theme.typography.label_medium,
        class: Box::new(style),
    }
}

/// A slider with one handle. Without a change handler it is disabled.
pub fn slider<'a, Message>(
    theme: &Theme,
    range: RangeInclusive<f32>,
    value: f32,
) -> Slider<'a, Message> {
    build(theme, range, [value, value], 1)
}

/// A slider with a start and an end handle. Without a change handler it is disabled.
pub fn range_slider<'a, Message>(
    theme: &Theme,
    range: RangeInclusive<f32>,
    values: (f32, f32),
) -> Slider<'a, Message> {
    build(theme, range, [values.0, values.1], 2)
}

impl<'a, Message> Slider<'a, Message> {
    /// Sets the message produced with the new value of a single slider.
    pub fn on_change(mut self, on_change: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_change = Some(Change::Single(Box::new(on_change)));
        self
    }

    /// Sets the message produced with the new start and end of a range slider.
    pub fn on_range_change(mut self, on_change: impl Fn(f32, f32) -> Message + 'a) -> Self {
        self.on_change = Some(Change::Range(Box::new(on_change)));
        self
    }

    /// Snaps values to multiples of `step` from the minimum.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Shows a tick mark at every step.
    pub fn ticks(mut self, ticks: bool) -> Self {
        self.ticks = ticks;
        self
    }

    /// Shows the value in a label above a handle while it is hovered, focused or dragged.
    pub fn labeled(mut self, labeled: bool) -> Self {
        self.labeled = labeled;
        self
    }

    /// Replaces the text of the value label.
    pub fn format(mut self, format: impl Fn(f32) -> String + 'a) -> Self {
        self.format = Some(Box::new(format));
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn fraction(&self, value: f32) -> f32 {
        if self.max > self.min {
            ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn inset(&self) -> f32 {
        self.metrics.handle_size / 2.0
    }

    fn x_of(&self, bounds: Rectangle, value: f32) -> f32 {
        let inset = self.inset();
        bounds.x + inset + self.fraction(value) * (bounds.width - inset * 2.0)
    }

    fn snap(&self, value: f32) -> f32 {
        let value = value.clamp(self.min, self.max);
        match self.step {
            Some(step) if step > 0.0 => {
                (self.min + ((value - self.min) / step).round() * step).clamp(self.min, self.max)
            }
            _ => value,
        }
    }

    fn value_at(&self, bounds: Rectangle, x: f32) -> f32 {
        let inset = self.inset();
        let span = (bounds.width - inset * 2.0).max(1.0);
        let fraction = ((x - bounds.x - inset) / span).clamp(0.0, 1.0);
        self.snap(self.min + fraction * (self.max - self.min))
    }

    fn handle_box(&self, bounds: Rectangle, value: f32) -> Rectangle {
        let s = self.metrics.state_layer_size;
        Rectangle::new(
            Point::new(
                self.x_of(bounds, value) - s / 2.0,
                bounds.center_y() - s / 2.0,
            ),
            Size::new(s, s),
        )
    }

    fn key_step(&self) -> f32 {
        match self.step {
            Some(step) if step > 0.0 => step,
            _ => (self.max - self.min) / 100.0,
        }
    }

    fn text(&self, value: f32) -> String {
        if let Some(format) = &self.format {
            return format(value);
        }
        let decimals = match self.step {
            Some(step) if step > 0.0 => (0..=6)
                .find(|d| {
                    let scaled = step * 10f32.powi(*d);
                    (scaled.round() - scaled).abs() < 1e-3
                })
                .map_or(6, |d| d as usize),
            _ => match self.max - self.min {
                span if span >= 10.0 => 0,
                span if span >= 1.0 => 1,
                _ => 2,
            },
        };
        format!("{value:.decimals$}")
    }

    fn publish(&self, index: usize, value: f32, shell: &mut Shell<'_, Message>) {
        let mut values = self.values;
        let value = self.snap(value);
        if self.count == 2 {
            if index == 0 {
                values[0] = value.min(values[1]);
            } else {
                values[1] = value.max(values[0]);
            }
        } else {
            values[0] = value;
        }
        if values == self.values {
            return;
        }
        match &self.on_change {
            Some(Change::Single(f)) => shell.publish(f(values[0])),
            Some(Change::Range(f)) => shell.publish(f(values[0], values[1])),
            None => {}
        }
    }
}

struct Handle {
    interaction: Interaction,
    label: Tween,
    text: Label,
}

impl Default for Handle {
    fn default() -> Self {
        Handle {
            interaction: Interaction::default(),
            label: Tween::new(0.0),
            text: Label::default(),
        }
    }
}

#[derive(Default)]
struct State {
    handles: [Handle; 2],
    dragging: Option<usize>,
    grab: f32,
    on_top: usize,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Slider<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            on_top: self.count - 1,
            ..State::default()
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.metrics.target_height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        for i in 0..self.count {
            let text = self.text(self.values[i]);
            state.handles[i].text.update(&text, self.type_style);
        }
        let height = self.metrics.target_height;
        layout::Node::new(limits.resolve(
            self.width,
            Length::Fixed(height),
            Size::new(self.metrics.min_width, height),
        ))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_change.is_none() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        for i in 0..self.count {
            let b = self.handle_box(layout.bounds(), self.values[i]);
            operation.focusable(None, b, &mut state.handles[i].interaction.focus);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let enabled = self.on_change.is_some();
        let n = self.count;

        for i in 0..n {
            let b = self.handle_box(bounds, self.values[i]);
            let c = if state.dragging == Some(i) {
                Cursor::Available(b.center())
            } else {
                cursor
            };
            let h = &mut state.handles[i];
            h.interaction
                .update(event, b, c, enabled, &self.tokens, shell);
            let shown = enabled
                && (h.interaction.hovered
                    || h.interaction.focus.focused
                    || state.dragging == Some(i));
            h.label.go(
                if shown { 1.0 } else { 0.0 },
                self.label_transition,
                h.interaction.now,
            );
            if let Event::Window(window::Event::RedrawRequested(now)) = event
                && h.label.tick(*now)
            {
                shell.request_redraw();
            }
        }
        if !enabled {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if state.dragging.is_none() =>
            {
                if let Some(position) = cursor.position_over(bounds) {
                    let on_handle: Vec<usize> = (0..n)
                        .filter(|i| state.handles[*i].interaction.pressed)
                        .collect();
                    let chosen = if on_handle.is_empty() {
                        let nearest = (0..n)
                            .min_by(|a, b| {
                                let da = (self.x_of(bounds, self.values[*a]) - position.x).abs();
                                let db = (self.x_of(bounds, self.values[*b]) - position.x).abs();
                                da.total_cmp(&db).then_with(|| {
                                    if position.x < self.x_of(bounds, self.values[*a]) {
                                        std::cmp::Ordering::Less
                                    } else {
                                        std::cmp::Ordering::Greater
                                    }
                                })
                            })
                            .unwrap_or(0);
                        let h = &mut state.handles[nearest];
                        h.interaction.pressed = true;
                        h.interaction.focus.focused = true;
                        h.interaction.focus.visible = false;
                        let half = self.metrics.state_layer_size / 2.0;
                        h.interaction.ripple.press(
                            Point::new(half, half),
                            self.tokens.state.pressed,
                            &self.tokens.ripple,
                            h.interaction.now,
                        );
                        state.grab = 0.0;
                        self.publish(nearest, self.value_at(bounds, position.x), shell);
                        nearest
                    } else {
                        let top = if on_handle.contains(&state.on_top) {
                            state.on_top
                        } else {
                            on_handle[0]
                        };
                        state.grab = position.x - self.x_of(bounds, self.values[top]);
                        top
                    };
                    for i in 0..n {
                        if i != chosen {
                            let h = &mut state.handles[i];
                            h.interaction.pressed = false;
                            h.interaction.ripple.release();
                            h.interaction.focus.focused = false;
                            h.interaction.focus.visible = false;
                        }
                    }
                    state.dragging = Some(chosen);
                    state.on_top = chosen;
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(i) = state.dragging {
                    self.publish(i, self.value_at(bounds, position.x - state.grab), shell);
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = None;
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if let Some(position) = cursor.position_over(bounds) {
                    let y = match delta {
                        ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. } => *y,
                    };
                    if y != 0.0 {
                        let target = (0..n)
                            .find(|i| state.handles[*i].interaction.focus.focused)
                            .unwrap_or_else(|| {
                                if n == 1 {
                                    0
                                } else {
                                    let mid = (self.values[0] + self.values[1]) / 2.0;
                                    usize::from(position.x >= self.x_of(bounds, mid))
                                }
                            });
                        let direction = if y > 0.0 { 1.0 } else { -1.0 };
                        self.publish(
                            target,
                            self.values[target] + direction * self.key_step(),
                            shell,
                        );
                        shell.capture_event();
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) => {
                let focused: Vec<usize> = (0..n)
                    .filter(|i| state.handles[*i].interaction.focus.focused)
                    .collect();
                if let Some(&i) = focused
                    .iter()
                    .find(|i| **i == state.on_top)
                    .or(focused.first())
                {
                    let step = self.key_step();
                    let page = step.max((self.max - self.min) / 10.0);
                    let current = self.values[i];
                    let target = match key {
                        Named::ArrowLeft | Named::ArrowDown => Some(current - step),
                        Named::ArrowRight | Named::ArrowUp => Some(current + step),
                        Named::PageDown => Some(current - page),
                        Named::PageUp => Some(current + page),
                        Named::Home => Some(self.min),
                        Named::End => Some(self.max),
                        _ => None,
                    };
                    if let Some(target) = target {
                        state.on_top = i;
                        self.publish(i, target, shell);
                        shell.capture_event();
                        shell.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let m = &self.metrics;
        let bounds = layout.bounds();
        let enabled = self.on_change.is_some();
        let style = theme.style(
            &self.class,
            if enabled {
                Status::Active
            } else {
                Status::Disabled
            },
        );
        let w = bounds.width;
        let cy = bounds.center_y();
        let n = self.count;

        let cap = m.track_height / 2.0;
        let track = |from: f32, to: f32, color: Color, renderer: &mut Renderer| {
            let round = |at_edge: bool| if at_edge { cap } else { 0.0 };
            surface::fill(
                renderer,
                Rectangle::new(
                    Point::new(bounds.x + from, cy - m.track_height / 2.0),
                    Size::new(to - from, m.track_height),
                ),
                Radius {
                    top_left: round(from <= 0.0),
                    bottom_left: round(from <= 0.0),
                    top_right: round(to >= w),
                    bottom_right: round(to >= w),
                },
                color,
            );
        };
        let left = if n == 2 && self.fraction(self.values[0]) > 0.0 {
            self.x_of(bounds, self.values[0]) - bounds.x
        } else {
            0.0
        };
        let right = if self.fraction(self.values[n - 1]) < 1.0 {
            self.x_of(bounds, self.values[n - 1]) - bounds.x
        } else {
            w
        };
        if right > left {
            if left > 0.0 {
                track(0.0, left, style.inactive_track, renderer);
            }
            if right < w {
                track(right, w, style.inactive_track, renderer);
            }
            track(left, right, style.active_track, renderer);
        } else {
            track(0.0, w, style.inactive_track, renderer);
        }

        if self.ticks
            && let Some(step) = self.step.filter(|s| *s > 0.0)
        {
            let count = ((self.max - self.min) / step).round() as usize;
            for i in 0..=count {
                let x = self.x_of(bounds, self.min + i as f32 * step);
                let active = x - bounds.x >= left && x - bounds.x <= right;
                let size = m.tick_size;
                surface::fill(
                    renderer,
                    Rectangle::new(
                        Point::new(x - size / 2.0, cy - size / 2.0),
                        Size::new(size, size),
                    ),
                    Radius::from(size / 2.0),
                    if active {
                        style.active_tick
                    } else {
                        style.inactive_tick
                    },
                );
            }
        }

        let overlapping = n == 2
            && (self.x_of(bounds, self.values[1]) - self.x_of(bounds, self.values[0])).abs()
                < m.handle_size;
        for i in 0..n {
            let h = &state.handles[i];
            let now = h.interaction.now;
            let layer = self.handle_box(bounds, self.values[i]);
            let radius = Radius::from(layer.width / 2.0);
            if enabled {
                surface::state_layer(
                    renderer,
                    layer,
                    radius,
                    style.state_layer,
                    h.interaction.hover.value(now),
                );
                h.interaction
                    .ripple
                    .draw(renderer, layer, radius, style.state_layer, theme, now);
            }
            let size = m.handle_size;
            let handle = Rectangle::new(
                Point::new(layer.center_x() - size / 2.0, layer.center_y() - size / 2.0),
                Size::new(size, size),
            );
            let round = Radius::from(size / 2.0);
            shadow::draw(
                renderer,
                handle,
                round,
                style.elevation.min(m.handle_elevation),
                theme.colors.shadow,
                &theme.elevation,
            );
            surface::fill(renderer, handle, round, style.handle);
            if overlapping && state.on_top == i {
                surface::outline(
                    renderer,
                    handle,
                    round,
                    m.overlap_outline_width,
                    style.overlap_outline,
                );
            }
            focus_ring::draw(
                renderer,
                layer,
                radius,
                theme.focus_ring.outward_offset,
                h.interaction.focus_ring_width(&self.tokens),
                theme.colors.secondary,
            );
        }

        if self.labeled {
            for i in 0..n {
                let h = &state.handles[i];
                let scale = h.label.value(h.interaction.now).clamp(0.0, 1.0);
                if scale > 0.0 {
                    let x = self.x_of(bounds, self.values[i]);
                    let bottom = cy - m.state_layer_size / 2.0;
                    self.draw_label(renderer, &style, h, x, bottom, scale, *viewport);
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_change.is_some() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<Message> Slider<'_, Message> {
    #[allow(clippy::too_many_arguments)]
    fn draw_label(
        &self,
        renderer: &mut Renderer,
        style: &Style,
        handle: &Handle,
        x: f32,
        bottom: f32,
        scale: f32,
        viewport: Rectangle,
    ) {
        let m = &self.metrics;
        let text = handle.text.size();
        let width = (text.width + m.label_padding * 2.0).max(m.label_size) * scale;
        let height = m.label_size * scale;
        let pill = Rectangle::new(
            Point::new(x - width / 2.0, bottom - height),
            Size::new(width, height),
        );
        let tip = m.label_size / 10.0;
        let half_diagonal = m.label_size / 2.0 * std::f32::consts::SQRT_2 / 2.0;
        let center = Point::new(x, bottom + (tip - m.label_size / 4.0) * scale);
        let reach = half_diagonal * scale;
        let region = Rectangle::new(
            Point::new(x - reach - 1.0, center.y - reach - 1.0),
            Size::new(reach * 2.0 + 2.0, reach * 2.0 + 2.0),
        );
        let mut frame = Frame::new(renderer, region.size());
        let local = Point::new(center.x - region.x, center.y - region.y);
        let diamond = Path::new(|b| {
            b.move_to(Point::new(local.x, local.y - reach));
            b.line_to(Point::new(local.x + reach, local.y));
            b.line_to(Point::new(local.x, local.y + reach));
            b.line_to(Point::new(local.x - reach, local.y));
            b.close();
        });
        frame.fill(
            &diamond,
            Fill {
                style: CanvasStyle::Solid(style.label_container),
                ..Fill::default()
            },
        );
        let geometry = frame.into_geometry();
        renderer.with_translation(Vector::new(region.x, region.y), |renderer| {
            renderer.draw_geometry(geometry);
        });
        surface::fill(
            renderer,
            pill,
            Radius::from(height / 2.0),
            style.label_container,
        );
        if scale > 0.1 {
            handle.text.draw(
                renderer,
                Point::new(x - text.width / 2.0, pill.center_y() - text.height / 2.0),
                alpha(style.label_text, scale * scale),
                viewport,
            );
        }
    }
}

impl<'a, Message: Clone + 'a> From<Slider<'a, Message>> for Element<'a, Message> {
    fn from(slider: Slider<'a, Message>) -> Self {
        IcedElement::new(slider)
    }
}
