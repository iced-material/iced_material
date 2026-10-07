// SPDX-License-Identifier: LGPL-3.0-only

//! Common buttons: elevated, filled, filled tonal, outlined and text.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::widget::svg;
use iced::{Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::text::Label;
use crate::icon::tinted;
use crate::interaction::{Activation, Tokens};
use crate::shape::Shape;
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::{self, Status, Style};

/// Horizontal padding of a button in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    /// Padding without icons, start and end.
    pub plain: (f32, f32),
    /// Padding with a leading icon.
    pub leading_icon: (f32, f32),
    /// Padding with a trailing icon.
    pub trailing_icon: (f32, f32),
}

/// Button dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Container height.
    pub height: f32,
    /// Minimum container width.
    pub min_width: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Space between icon and label.
    pub icon_label_space: f32,
    /// Padding of elevated, filled, filled tonal and outlined buttons.
    pub padding: Padding,
    /// Padding of text buttons.
    pub text_padding: Padding,
    /// Minimum height of the pointer target.
    pub target_height: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            height: 40.0,
            min_width: 64.0,
            icon_size: 18.0,
            icon_label_space: 8.0,
            padding: Padding {
                plain: (24.0, 24.0),
                leading_icon: (16.0, 24.0),
                trailing_icon: (24.0, 16.0),
            },
            text_padding: Padding {
                plain: (12.0, 12.0),
                leading_icon: (12.0, 16.0),
                trailing_icon: (16.0, 12.0),
            },
            target_height: 48.0,
        }
    }
}

/// The appearance catalog of buttons.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(filled_style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

fn disabled(theme: &Theme, container: bool) -> Style {
    let c = &theme.colors;
    let content = alpha(c.on_surface, theme.disabled.content);
    Style {
        container: if container {
            alpha(c.on_surface, theme.disabled.container)
        } else {
            iced::Color::TRANSPARENT
        },
        ..Style::content(theme, content, content, c.on_surface)
    }
}

fn levels(status: Status, active: f32, hovered: f32, pressed: f32) -> f32 {
    match status {
        Status::Hovered => hovered,
        Status::Pressed => pressed,
        Status::Disabled => 0.0,
        Status::Active | Status::Focused | Status::Dragged => active,
    }
}

/// Filled button style.
pub fn filled_style(theme: &Theme, status: Status) -> Style {
    if status == Status::Disabled {
        return disabled(theme, true);
    }
    let c = &theme.colors;
    Style {
        container: c.primary,
        elevation: levels(status, 0.0, 1.0, 0.0),
        ..Style::content(theme, c.on_primary, c.on_primary, c.on_primary)
    }
}

/// Elevated button style.
pub fn elevated_style(theme: &Theme, status: Status) -> Style {
    if status == Status::Disabled {
        return disabled(theme, true);
    }
    let c = &theme.colors;
    Style {
        container: c.surface_container_low,
        elevation: levels(status, 1.0, 2.0, 1.0),
        ..Style::content(theme, c.primary, c.primary, c.primary)
    }
}

/// Filled tonal button style.
pub fn filled_tonal_style(theme: &Theme, status: Status) -> Style {
    if status == Status::Disabled {
        return disabled(theme, true);
    }
    let c = &theme.colors;
    Style {
        container: c.secondary_container,
        elevation: levels(status, 0.0, 1.0, 0.0),
        ..Style::content(
            theme,
            c.on_secondary_container,
            c.on_secondary_container,
            c.on_secondary_container,
        )
    }
}

/// Outlined button style.
pub fn outlined_style(theme: &Theme, status: Status) -> Style {
    let c = &theme.colors;
    if status == Status::Disabled {
        return Style {
            outline: alpha(c.on_surface, theme.disabled.outline),
            outline_width: 1.0,
            ..disabled(theme, false)
        };
    }
    Style {
        outline: c.outline,
        outline_width: 1.0,
        ..Style::content(theme, c.primary, c.primary, c.primary)
    }
}

/// Text button style.
pub fn text_style(theme: &Theme, status: Status) -> Style {
    if status == Status::Disabled {
        return disabled(theme, false);
    }
    let c = &theme.colors;
    Style::content(theme, c.primary, c.primary, c.primary)
}

/// A Material common button.
pub struct Button<'a, Message> {
    label: String,
    leading_icon: Option<svg::Handle>,
    trailing_icon: Option<svg::Handle>,
    on_press: Option<Message>,
    width: Length,
    text: bool,
    metrics: Metrics,
    tokens: Tokens,
    type_style: TypeStyle,
    shape: Shape,
    class: StyleFn<'a>,
}

impl<'a, Message> Button<'a, Message> {
    fn new(theme: &Theme, label: impl Into<String>, class: StyleFn<'a>, text: bool) -> Self {
        Button {
            label: label.into(),
            leading_icon: None,
            trailing_icon: None,
            on_press: None,
            width: Length::Shrink,
            text,
            metrics: theme.components.button,
            tokens: Tokens::new(theme),
            type_style: theme.typography.label_large,
            shape: theme.shape.full,
            class,
        }
    }

    /// Sets the message produced on press. Without one the button is disabled.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message produced on press, or disables the button with `None`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Adds a leading icon.
    pub fn leading_icon(mut self, icon: svg::Handle) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Adds a trailing icon.
    pub fn trailing_icon(mut self, icon: svg::Handle) -> Self {
        self.trailing_icon = Some(icon);
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

    fn padding(&self) -> (f32, f32) {
        let padding = if self.text {
            self.metrics.text_padding
        } else {
            self.metrics.padding
        };
        if self.leading_icon.is_some() {
            padding.leading_icon
        } else if self.trailing_icon.is_some() {
            padding.trailing_icon
        } else {
            padding.plain
        }
    }
}

/// A filled button.
pub fn filled<'a, Message>(theme: &Theme, label: impl Into<String>) -> Button<'a, Message> {
    Button::new(theme, label, Box::new(filled_style), false)
}

/// An elevated button.
pub fn elevated<'a, Message>(theme: &Theme, label: impl Into<String>) -> Button<'a, Message> {
    Button::new(theme, label, Box::new(elevated_style), false)
}

/// A filled tonal button.
pub fn filled_tonal<'a, Message>(theme: &Theme, label: impl Into<String>) -> Button<'a, Message> {
    Button::new(theme, label, Box::new(filled_tonal_style), false)
}

/// An outlined button.
pub fn outlined<'a, Message>(theme: &Theme, label: impl Into<String>) -> Button<'a, Message> {
    Button::new(theme, label, Box::new(outlined_style), false)
}

/// A text button.
pub fn text<'a, Message>(theme: &Theme, label: impl Into<String>) -> Button<'a, Message> {
    Button::new(theme, label, Box::new(text_style), true)
}

#[derive(Default)]
struct State {
    press: pressable::State,
    label: Label,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Button<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.metrics.height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let label = state.label.update(&self.label, self.type_style);
        let icons = [&self.leading_icon, &self.trailing_icon]
            .iter()
            .filter(|i| i.is_some())
            .count() as f32;
        let (start, end) = self.padding();
        let content =
            label.width + icons * (self.metrics.icon_size + self.metrics.icon_label_space);
        let intrinsic = (start + content + end).max(self.metrics.min_width);
        let size = limits.resolve(
            self.width,
            Length::Fixed(self.metrics.height),
            Size::new(intrinsic, self.metrics.height),
        );
        layout::Node::new(size)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_press.is_some() {
            let state = tree.state.downcast_mut::<State>();
            operation.focusable(None, layout.bounds(), &mut state.press.interaction.focus);
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
        let state = tree.state.downcast_mut::<State>();
        let hit = pressable::hit_area(layout.bounds(), self.metrics.target_height);
        let enabled = self.on_press.is_some();
        if let Some(Activation::Pointer | Activation::Keyboard) =
            state
                .press
                .update(event, hit, cursor, enabled, false, &self.tokens, shell)
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
        let (style, elevation) = state
            .press
            .style(enabled, false, |status| theme.style(&self.class, status));
        pressable::draw_container(
            renderer,
            theme,
            bounds,
            radius,
            &style,
            elevation,
            &state.press,
        );

        let m = &self.metrics;
        let label = state.label.size();
        let (start, end) = self.padding();
        let icons = [&self.leading_icon, &self.trailing_icon]
            .iter()
            .filter(|i| i.is_some())
            .count() as f32;
        let content = label.width + icons * (m.icon_size + m.icon_label_space);
        let free = bounds.width - start - end - content;
        let mut x = bounds.x + start + free.max(0.0) / 2.0;
        let icon_y = bounds.center_y() - m.icon_size / 2.0;
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let draw_icon = |renderer: &mut Renderer, handle: &svg::Handle, x: f32| {
            renderer.draw_svg(
                tinted(handle.clone(), style.icon, 1.0),
                Rectangle::new(Point::new(x, icon_y), Size::new(m.icon_size, m.icon_size)),
                clip,
            );
        };
        if let Some(icon) = &self.leading_icon {
            draw_icon(renderer, icon, x);
            x += m.icon_size + m.icon_label_space;
        }
        let label_clip = Rectangle {
            x: bounds.x + start,
            width: (bounds.width - start - end).max(0.0),
            ..bounds
        }
        .intersection(&clip)
        .unwrap_or(clip);
        state.label.draw(
            renderer,
            Point::new(x, bounds.center_y() - label.height / 2.0),
            style.label,
            label_clip,
        );
        x += label.width;
        if let Some(icon) = &self.trailing_icon {
            draw_icon(renderer, icon, x + m.icon_label_space);
        }
        pressable::draw_focus(renderer, theme, bounds, radius, &state.press, &self.tokens);
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

impl<'a, Message: Clone + 'a> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(button: Button<'a, Message>) -> Self {
        IcedElement::new(button)
    }
}
