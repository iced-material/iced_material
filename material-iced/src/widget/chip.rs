// SPDX-License-Identifier: LGPL-3.0-only

//! Chips: assist, filter, input and suggestion.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::widget::svg;
use iced::{Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::surface;
use crate::draw::text::Label;
use crate::icon::{symbol, tinted};
use crate::interaction::{Interaction, Tokens};
use crate::shape::Shape;
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::{self, Status};

/// Chip dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Container height.
    pub height: f32,
    /// Start and end padding without icons.
    pub padding: f32,
    /// Start padding with a leading icon, end padding with a trailing icon.
    pub icon_padding: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Space between icons and the label.
    pub icon_label_space: f32,
    /// Diameter of the state layer of the trailing remove action.
    pub trailing_action_size: f32,
    /// Minimum height of the pointer target.
    pub target_height: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            height: 32.0,
            padding: 16.0,
            icon_padding: 8.0,
            icon_size: 18.0,
            icon_label_space: 8.0,
            trailing_action_size: 24.0,
            target_height: 48.0,
        }
    }
}

/// Chip type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Assist chip.
    Assist,
    /// Filter chip.
    Filter,
    /// Input chip.
    Input,
    /// Suggestion chip.
    Suggestion,
}

/// What a chip style depends on besides the status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variant {
    /// Chip type.
    pub kind: Kind,
    /// Selection of filter and input chips.
    pub selected: bool,
    /// Elevated instead of flat. Input chips are always flat.
    pub elevated: bool,
}

/// The appearance of a chip in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Container, outline, label, leading icon and state layers.
    pub base: pressable::Style,
    /// Trailing icon color.
    pub trailing_icon: Color,
}

/// The appearance catalog of chips.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, variant: Variant) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, Variant) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, variant: Variant) -> Style {
        class(self, status, variant)
    }
}

/// The baseline chip style.
pub fn style(theme: &Theme, status: Status, variant: Variant) -> Style {
    let c = &theme.colors;
    let Variant {
        kind,
        selected,
        elevated,
    } = variant;
    let selected = selected && matches!(kind, Kind::Filter | Kind::Input);
    let elevated = elevated && kind != Kind::Input;

    if status == Status::Disabled {
        let content = alpha(c.on_surface, theme.disabled.content);
        let filled = elevated || selected;
        return Style {
            base: pressable::Style {
                container: if filled {
                    alpha(c.on_surface, theme.disabled.container)
                } else {
                    Color::TRANSPARENT
                },
                outline: alpha(c.on_surface, theme.disabled.outline),
                outline_width: if filled { 0.0 } else { 1.0 },
                ..pressable::Style::content(theme, content, content, c.on_surface)
            },
            trailing_icon: content,
        };
    }

    let (label, leading, trailing, hover, pressed) = match (kind, selected) {
        (Kind::Assist, _) => (
            c.on_surface,
            c.primary,
            c.primary,
            c.on_surface,
            c.on_surface,
        ),
        (Kind::Suggestion, _) => (
            c.on_surface_variant,
            c.primary,
            c.primary,
            c.on_surface_variant,
            c.on_surface_variant,
        ),
        (Kind::Filter, false) => (
            c.on_surface_variant,
            c.primary,
            c.on_surface_variant,
            c.on_surface_variant,
            c.on_secondary_container,
        ),
        (Kind::Filter, true) => (
            c.on_secondary_container,
            c.on_secondary_container,
            c.on_secondary_container,
            c.on_secondary_container,
            c.on_surface_variant,
        ),
        (Kind::Input, false) => (
            c.on_surface_variant,
            c.on_surface_variant,
            c.on_surface_variant,
            c.on_surface_variant,
            c.on_surface_variant,
        ),
        (Kind::Input, true) => (
            c.on_secondary_container,
            c.primary,
            c.on_secondary_container,
            c.on_secondary_container,
            c.on_secondary_container,
        ),
    };
    let container = if selected {
        c.secondary_container
    } else if elevated {
        c.surface_container_low
    } else {
        Color::TRANSPARENT
    };
    let elevation = if elevated {
        match status {
            Status::Hovered => 2.0,
            Status::Dragged => 4.0,
            _ => 1.0,
        }
    } else {
        match status {
            Status::Hovered if selected && kind == Kind::Filter => 1.0,
            Status::Dragged => 4.0,
            _ => 0.0,
        }
    };
    let outlined = !elevated && !selected;
    Style {
        base: pressable::Style {
            container,
            outline: c.outline,
            outline_width: if outlined { 1.0 } else { 0.0 },
            label,
            icon: leading,
            hover_layer: hover,
            pressed_layer: pressed,
            elevation,
            shadow: c.shadow,
        },
        trailing_icon: trailing,
    }
}

/// A Material chip.
pub struct Chip<'a, Message> {
    label: String,
    kind: Kind,
    selected: bool,
    elevated: bool,
    icon: Option<svg::Handle>,
    trailing_icon: Option<svg::Handle>,
    on_press: Option<Message>,
    on_remove: Option<Message>,
    metrics: Metrics,
    tokens: Tokens,
    type_style: TypeStyle,
    shape: Shape,
    class: StyleFn<'a>,
}

impl<'a, Message> Chip<'a, Message> {
    fn new(theme: &Theme, kind: Kind, label: String) -> Self {
        Chip {
            label,
            kind,
            selected: false,
            elevated: false,
            icon: None,
            trailing_icon: None,
            on_press: None,
            on_remove: None,
            metrics: theme.components.chip,
            tokens: Tokens::new(theme),
            type_style: theme.typography.label_large,
            shape: theme.shape.small,
            class: Box::new(style),
        }
    }

    /// Sets the message produced on press. Without one the chip is disabled.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message produced on press, or disables the chip with `None`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Sets the selection of a filter or input chip.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Uses the elevated container instead of the outline.
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    /// Adds a leading icon. A selected filter chip shows a checkmark instead.
    pub fn icon(mut self, icon: svg::Handle) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Adds a trailing icon to a filter chip, drawn as part of the chip.
    pub fn trailing_icon(mut self, icon: svg::Handle) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Adds the remove action of an input chip.
    pub fn on_remove(mut self, message: Message) -> Self {
        self.on_remove = Some(message);
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, Variant) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn leading(&self) -> Option<svg::Handle> {
        if self.kind == Kind::Filter && self.selected {
            Some(symbol::check(false))
        } else {
            self.icon.clone()
        }
    }

    fn trailing(&self) -> Option<svg::Handle> {
        if self.on_remove.is_some() {
            Some(symbol::close(false))
        } else {
            self.trailing_icon.clone()
        }
    }

    fn variant(&self) -> Variant {
        Variant {
            kind: self.kind,
            selected: self.selected,
            elevated: self.elevated,
        }
    }

    fn remove_bounds(&self, bounds: Rectangle) -> Rectangle {
        let m = &self.metrics;
        let center = Point::new(
            bounds.x + bounds.width - m.icon_padding - m.icon_size / 2.0,
            bounds.center_y(),
        );
        let size = m.trailing_action_size;
        Rectangle::new(
            Point::new(center.x - size / 2.0, center.y - size / 2.0),
            Size::new(size, size),
        )
    }
}

/// An assist chip.
pub fn assist<'a, Message>(theme: &Theme, label: impl Into<String>) -> Chip<'a, Message> {
    Chip::new(theme, Kind::Assist, label.into())
}

/// A filter chip.
pub fn filter<'a, Message>(theme: &Theme, label: impl Into<String>) -> Chip<'a, Message> {
    Chip::new(theme, Kind::Filter, label.into())
}

/// An input chip.
pub fn input<'a, Message>(theme: &Theme, label: impl Into<String>) -> Chip<'a, Message> {
    Chip::new(theme, Kind::Input, label.into())
}

/// A suggestion chip.
pub fn suggestion<'a, Message>(theme: &Theme, label: impl Into<String>) -> Chip<'a, Message> {
    Chip::new(theme, Kind::Suggestion, label.into())
}

#[derive(Default)]
pub(crate) struct State {
    press: pressable::State,
    remove: Interaction,
    label: Label,
    pub enabled: bool,
    pub removable: bool,
}

impl State {
    pub(crate) fn is_focused(&self) -> bool {
        self.press.interaction.focus.focused || self.remove.focus.focused
    }

    pub(crate) fn focus(&mut self, trailing: bool) {
        let focus = if trailing && self.removable {
            &mut self.remove.focus
        } else {
            &mut self.press.interaction.focus
        };
        focus.focused = true;
        focus.visible = true;
        focus.since = None;
    }

    pub(crate) fn unfocus(&mut self) {
        for focus in [&mut self.press.interaction.focus, &mut self.remove.focus] {
            focus.focused = false;
            focus.visible = false;
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Chip<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Fixed(self.metrics.height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let m = &self.metrics;
        let label = state.label.update(&self.label, self.type_style);
        let mut width = label.width;
        width += match self.leading() {
            Some(_) => m.icon_padding + m.icon_size + m.icon_label_space,
            None => m.padding,
        };
        width += match self.trailing() {
            Some(_) => m.icon_label_space + m.icon_size + m.icon_padding,
            None => m.padding,
        };
        layout::Node::new(Size::new(width, m.height))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let enabled = self.on_press.is_some();
        if enabled {
            operation.focusable(None, bounds, &mut state.press.interaction.focus);
            if self.on_remove.is_some() {
                operation.focusable(None, self.remove_bounds(bounds), &mut state.remove.focus);
            }
        }
        state.enabled = enabled;
        state.removable = self.on_remove.is_some();
        operation.custom(None, bounds, state);
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
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let enabled = self.on_press.is_some();

        if self.on_remove.is_some()
            && let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key @ (Named::ArrowLeft | Named::ArrowRight)),
                ..
            }) = event
        {
            let primary = state.press.interaction.focus.focused;
            let trailing = state.remove.focus.focused;
            if (primary && *key == Named::ArrowRight) || (trailing && *key == Named::ArrowLeft) {
                state.unfocus();
                state.focus(primary);
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        if let Some(message) = &self.on_remove {
            let remove = self.remove_bounds(bounds);
            if state
                .remove
                .update(event, remove, cursor, enabled, &self.tokens, shell)
                .is_some()
            {
                shell.publish(message.clone());
                return;
            }
            if shell.is_event_captured() {
                return;
            }
        }

        let hit = pressable::hit_area(bounds, self.metrics.target_height);
        if state
            .press
            .update(event, hit, cursor, enabled, false, &self.tokens, shell)
            .is_some()
            && let Some(message) = &self.on_press
        {
            shell.publish(message.clone());
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
        let bounds = layout.bounds();
        let radius = self.shape.radius(bounds.size());
        let enabled = self.on_press.is_some();
        let variant = self.variant();
        let (base, elevation) = state.press.style(enabled, false, |status| {
            theme.style(&self.class, status, variant).base
        });
        let trailing_color = theme
            .style(&self.class, state.press.status(enabled, false), variant)
            .trailing_icon;
        pressable::draw_container(
            renderer,
            theme,
            bounds,
            radius,
            &base,
            elevation,
            &state.press,
        );

        let m = &self.metrics;
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let draw_icon = |renderer: &mut Renderer, icon: svg::Handle, x: f32, color: Color| {
            renderer.draw_svg(
                tinted(icon, color, 1.0),
                Rectangle::new(
                    Point::new(x, bounds.center_y() - m.icon_size / 2.0),
                    Size::new(m.icon_size, m.icon_size),
                ),
                clip,
            );
        };
        let mut x = bounds.x;
        match self.leading() {
            Some(icon) => {
                draw_icon(renderer, icon, x + m.icon_padding, base.icon);
                x += m.icon_padding + m.icon_size + m.icon_label_space;
            }
            None => x += m.padding,
        }
        let label = state.label.size();
        state.label.draw(
            renderer,
            Point::new(x, bounds.center_y() - label.height / 2.0),
            base.label,
            clip,
        );
        if let Some(icon) = self.trailing() {
            let x = bounds.x + bounds.width - m.icon_padding - m.icon_size;
            if self.on_remove.is_some() && enabled {
                let remove = self.remove_bounds(bounds);
                let round = Radius::from(remove.width / 2.0);
                surface::state_layer(
                    renderer,
                    remove,
                    round,
                    base.hover_layer,
                    state.remove.hover.value(state.remove.now),
                );
                state.remove.ripple.draw(
                    renderer,
                    remove,
                    round,
                    base.pressed_layer,
                    theme,
                    state.remove.now,
                );
            }
            draw_icon(renderer, icon, x, trailing_color);
        }

        pressable::draw_focus(renderer, theme, bounds, radius, &state.press, &self.tokens);
        if self.on_remove.is_some() {
            let remove = self.remove_bounds(bounds);
            let width = state.remove.focus_ring_width(&self.tokens);
            crate::draw::focus_ring::draw(
                renderer,
                remove,
                Radius::from(remove.width / 2.0),
                theme.focus_ring.outward_offset,
                width,
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
        let hit = pressable::hit_area(layout.bounds(), self.metrics.target_height);
        if self.on_press.is_some() && cursor.is_over(hit) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<Chip<'a, Message>> for Element<'a, Message> {
    fn from(chip: Chip<'a, Message>) -> Self {
        IcedElement::new(chip)
    }
}
