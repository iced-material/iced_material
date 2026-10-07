// SPDX-License-Identifier: LGPL-3.0-only

//! Radio button groups with arrow key navigation.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::{Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::text::Label;
use crate::draw::{focus_ring, surface};
use crate::interaction::Tokens;
use crate::motion::Transition;
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::Status;
use crate::widget::selection::{self, State, centered};

/// Radio button dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Width and height of the control, which is also the row height.
    pub target_size: f32,
    /// Diameter of the outer circle.
    pub icon_size: f32,
    /// Stroke width of the outer circle.
    pub ring_width: f32,
    /// Diameter of the selected dot.
    pub dot_size: f32,
    /// Diameter of the state layer.
    pub state_layer_size: f32,
    /// Space between the control and the label.
    pub label_space: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            target_size: 48.0,
            icon_size: 20.0,
            ring_width: 2.0,
            dot_size: 10.0,
            state_layer_size: 40.0,
            label_space: 4.0,
        }
    }
}

/// The appearance of a radio button in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Color of the circle and the dot.
    pub icon: Color,
    /// Label color.
    pub label: Color,
    /// Hover and focus state layer color.
    pub hover_layer: Color,
    /// Press ripple color.
    pub pressed_layer: Color,
}

/// The appearance catalog of radio groups.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, bool) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style {
        class(self, status, selected)
    }
}

/// The baseline radio button style.
pub fn style(theme: &Theme, status: Status, selected: bool) -> Style {
    let c = &theme.colors;
    if status == Status::Disabled {
        let disabled = alpha(c.on_surface, theme.disabled.content);
        return Style {
            icon: disabled,
            label: disabled,
            hover_layer: c.on_surface,
            pressed_layer: c.on_surface,
        };
    }
    let engaged = status != Status::Active;
    Style {
        icon: match (selected, engaged) {
            (true, _) => c.primary,
            (false, true) => c.on_surface,
            (false, false) => c.on_surface_variant,
        },
        label: c.on_surface,
        hover_layer: if selected { c.primary } else { c.on_surface },
        pressed_layer: if selected { c.on_surface } else { c.primary },
    }
}

/// A group of radio buttons with labels, one of which can be selected.
pub struct RadioGroup<'a, Message> {
    labels: Vec<String>,
    disabled: Vec<bool>,
    selected: Option<usize>,
    on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    metrics: Metrics,
    tokens: Tokens,
    enter: Transition,
    exit: Transition,
    type_style: TypeStyle,
    class: StyleFn<'a>,
}

/// A radio group. Without `on_select` every item is disabled.
pub fn radio_group<'a, Message>(
    theme: &Theme,
    labels: impl IntoIterator<Item = impl Into<String>>,
    selected: Option<usize>,
) -> RadioGroup<'a, Message> {
    let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
    let motion = &theme.motion;
    RadioGroup {
        disabled: vec![false; labels.len()],
        labels,
        selected,
        on_select: None,
        metrics: theme.components.radio,
        tokens: Tokens::new(theme),
        enter: Transition {
            duration: std::time::Duration::from_millis(300),
            easing: motion.easing.emphasized_decelerate,
        },
        exit: Transition {
            duration: motion.duration.short1,
            easing: motion.easing.linear,
        },
        type_style: theme.typography.body_large,
        class: Box::new(style),
    }
}

impl<'a, Message> RadioGroup<'a, Message> {
    /// Sets the message produced with the index of the chosen item.
    pub fn on_select(mut self, on_select: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(on_select));
        self
    }

    /// Disables one item.
    pub fn disable(mut self, index: usize) -> Self {
        if let Some(disabled) = self.disabled.get_mut(index) {
            *disabled = true;
        }
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, bool) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn enabled(&self, index: usize) -> bool {
        self.on_select.is_some() && !self.disabled[index]
    }

    fn row(&self, bounds: Rectangle, index: usize) -> Rectangle {
        let height = self.metrics.target_size;
        Rectangle::new(
            Point::new(bounds.x, bounds.y + index as f32 * height),
            Size::new(bounds.width, height),
        )
    }

    fn layer(&self, row: Rectangle) -> Rectangle {
        let s = self.metrics.state_layer_size;
        let control = Rectangle::new(
            row.position(),
            Size::new(self.metrics.target_size, self.metrics.target_size),
        );
        centered(control, Size::new(s, s))
    }

    fn roving(&self, state: &GroupState) -> Option<usize> {
        let n = self.labels.len();
        (0..n)
            .find(|i| state.items[*i].selection.press.interaction.focus.focused)
            .or_else(|| self.selected.filter(|i| *i < n && self.enabled(*i)))
            .or_else(|| (0..n).find(|i| self.enabled(*i)))
    }
}

struct Item {
    selection: State,
    label: Label,
}

#[derive(Default)]
struct GroupState {
    items: Vec<Item>,
}

impl GroupState {
    fn resize(&mut self, selected: Option<usize>, count: usize) {
        self.items.truncate(count);
        for i in self.items.len()..count {
            self.items.push(Item {
                selection: State::new(selected == Some(i)),
                label: Label::default(),
            });
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for RadioGroup<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GroupState>()
    }

    fn state(&self) -> tree::State {
        let mut state = GroupState::default();
        state.resize(self.selected, self.labels.len());
        tree::State::new(state)
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<GroupState>();
        state.resize(self.selected, self.labels.len());
        let m = &self.metrics;
        let mut width: f32 = m.target_size;
        for (label, item) in self.labels.iter().zip(&mut state.items) {
            let w = item.label.update(label, self.type_style).width;
            width = width.max(m.target_size + m.label_space + w);
        }
        layout::Node::new(Size::new(width, m.target_size * self.labels.len() as f32))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<GroupState>();
        if let Some(i) = self.roving(state) {
            let layer = self.layer(self.row(layout.bounds(), i));
            operation.focusable(
                None,
                layer,
                &mut state.items[i].selection.press.interaction.focus,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<GroupState>();
        state.resize(self.selected, self.labels.len());
        let bounds = layout.bounds();
        let n = self.labels.len();

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key:
                keyboard::Key::Named(
                    key
                    @ (Named::ArrowUp | Named::ArrowDown | Named::ArrowLeft | Named::ArrowRight),
                ),
            ..
        }) = event
            && let Some(current) =
                (0..n).find(|i| state.items[*i].selection.press.interaction.focus.focused)
        {
            let forward = matches!(key, Named::ArrowDown | Named::ArrowRight);
            let step = if forward { 1 } else { n - 1 };
            let mut next = (current + step) % n;
            while next != current && !self.enabled(next) {
                next = (next + step) % n;
            }
            if next != current {
                let old = &mut state.items[current].selection.press.interaction.focus;
                old.focused = false;
                old.visible = false;
                let focus = &mut state.items[next].selection.press.interaction.focus;
                focus.focused = true;
                focus.visible = true;
                focus.since = None;
                if let Some(on_select) = &self.on_select {
                    shell.publish(on_select(next));
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        for (i, item) in state.items.iter_mut().enumerate() {
            let enabled = self.on_select.is_some() && !self.disabled[i];
            let selected = self.selected == Some(i);
            let transition = if selected { self.enter } else { self.exit };
            let row = Rectangle {
                x: bounds.x,
                y: bounds.y + i as f32 * self.metrics.target_size,
                width: bounds.width,
                height: self.metrics.target_size,
            };
            let activation = item.selection.update(
                event,
                selected,
                transition,
                row,
                cursor,
                enabled,
                &self.tokens,
                shell,
            );
            if activation.is_some()
                && let Some(on_select) = &self.on_select
            {
                shell.publish(on_select(i));
            }
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
        let state = tree.state.downcast_ref::<GroupState>();
        let m = &self.metrics;
        for (i, item) in state.items.iter().enumerate() {
            let row = self.row(layout.bounds(), i);
            let control = Rectangle::new(row.position(), Size::new(m.target_size, m.target_size));
            let layer = self.layer(row);
            let enabled = self.enabled(i);
            let selected = self.selected == Some(i);
            let style = theme.style(
                &self.class,
                item.selection.press.status(enabled, false),
                selected,
            );
            if enabled {
                selection::draw_state_layer(
                    renderer,
                    theme,
                    &item.selection,
                    layer,
                    style.hover_layer,
                    style.pressed_layer,
                );
            }
            let icon = centered(control, Size::new(m.icon_size, m.icon_size));
            surface::outline(
                renderer,
                icon,
                Radius::from(m.icon_size / 2.0),
                m.ring_width,
                style.icon,
            );
            let t = item.selection.progress().clamp(0.0, 1.0);
            if t > 0.0 {
                let dot = m.dot_size * t;
                surface::fill(
                    renderer,
                    centered(control, Size::new(dot, dot)),
                    Radius::from(dot / 2.0),
                    alpha(style.icon, t),
                );
            }
            let label = item.label.size();
            item.label.draw(
                renderer,
                Point::new(
                    row.x + m.target_size + m.label_space,
                    row.center_y() - label.height / 2.0,
                ),
                style.label,
                *viewport,
            );
            focus_ring::draw(
                renderer,
                layer,
                Radius::from(layer.width / 2.0),
                theme.focus_ring.outward_offset,
                item.selection
                    .press
                    .interaction
                    .focus_ring_width(&self.tokens),
                theme.colors.secondary,
            );
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
        let bounds = layout.bounds();
        for i in 0..self.labels.len() {
            if self.enabled(i) && cursor.is_over(self.row(bounds, i)) {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

impl<'a, Message: Clone + 'a> From<RadioGroup<'a, Message>> for Element<'a, Message> {
    fn from(group: RadioGroup<'a, Message>) -> Self {
        IcedElement::new(group)
    }
}
