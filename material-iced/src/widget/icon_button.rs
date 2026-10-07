// SPDX-License-Identifier: LGPL-3.0-only

//! Icon buttons: standard, filled, filled tonal and outlined, with optional toggle.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::widget::svg;
use iced::{Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::icon::tinted;
use crate::interaction::Tokens;
use crate::shape::Shape;
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::pressable::{self, Status, Style};

/// Icon button dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Width and height of the container and state layer.
    pub size: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Minimum width and height of the pointer target.
    pub target_size: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            size: 40.0,
            icon_size: 24.0,
            target_size: 48.0,
        }
    }
}

/// Toggle state passed to icon button styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toggle {
    /// The button does not toggle.
    None,
    /// A toggle button that is not selected.
    Unselected,
    /// A toggle button that is selected.
    Selected,
}

/// The appearance catalog of icon buttons.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, toggle: Toggle) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, Toggle) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(standard_style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, toggle: Toggle) -> Style {
        class(self, status, toggle)
    }
}

fn disabled_icon(theme: &Theme) -> Color {
    alpha(theme.colors.on_surface, theme.disabled.content)
}

/// Standard icon button style.
pub fn standard_style(theme: &Theme, status: Status, toggle: Toggle) -> Style {
    let c = &theme.colors;
    if status == Status::Disabled {
        let icon = disabled_icon(theme);
        return Style::content(theme, icon, icon, c.on_surface);
    }
    let color = if toggle == Toggle::Selected {
        c.primary
    } else {
        c.on_surface_variant
    };
    Style::content(theme, color, color, color)
}

/// Filled icon button style.
pub fn filled_style(theme: &Theme, status: Status, toggle: Toggle) -> Style {
    let c = &theme.colors;
    if status == Status::Disabled {
        let icon = disabled_icon(theme);
        return Style {
            container: alpha(c.on_surface, theme.disabled.container),
            ..Style::content(theme, icon, icon, c.on_surface)
        };
    }
    let (container, content) = match toggle {
        Toggle::Unselected => (c.surface_container_highest, c.primary),
        Toggle::None | Toggle::Selected => (c.primary, c.on_primary),
    };
    Style {
        container,
        ..Style::content(theme, content, content, content)
    }
}

/// Filled tonal icon button style.
pub fn filled_tonal_style(theme: &Theme, status: Status, toggle: Toggle) -> Style {
    let c = &theme.colors;
    if status == Status::Disabled {
        let icon = disabled_icon(theme);
        return Style {
            container: alpha(c.on_surface, theme.disabled.container),
            ..Style::content(theme, icon, icon, c.on_surface)
        };
    }
    let (container, content) = match toggle {
        Toggle::Unselected => (c.surface_container_highest, c.on_surface_variant),
        Toggle::None | Toggle::Selected => (c.secondary_container, c.on_secondary_container),
    };
    Style {
        container,
        ..Style::content(theme, content, content, content)
    }
}

/// Outlined icon button style.
pub fn outlined_style(theme: &Theme, status: Status, toggle: Toggle) -> Style {
    let c = &theme.colors;
    if status == Status::Disabled {
        let icon = disabled_icon(theme);
        return if toggle == Toggle::Selected {
            Style {
                container: alpha(c.on_surface, theme.disabled.container),
                ..Style::content(theme, icon, icon, c.on_surface)
            }
        } else {
            Style {
                outline: alpha(c.on_surface, theme.disabled.outline),
                outline_width: 1.0,
                ..Style::content(theme, icon, icon, c.on_surface)
            }
        };
    }
    if toggle == Toggle::Selected {
        return Style {
            container: c.inverse_surface,
            ..Style::content(
                theme,
                c.inverse_on_surface,
                c.inverse_on_surface,
                c.inverse_on_surface,
            )
        };
    }
    Style {
        outline: c.outline,
        outline_width: 1.0,
        pressed_layer: c.on_surface,
        ..Style::content(
            theme,
            c.on_surface_variant,
            c.on_surface_variant,
            c.on_surface_variant,
        )
    }
}

/// A Material icon button.
pub struct IconButton<'a, Message> {
    icon: svg::Handle,
    selected_icon: Option<svg::Handle>,
    selected: Option<bool>,
    on_press: Option<Message>,
    metrics: Metrics,
    tokens: Tokens,
    shape: Shape,
    class: StyleFn<'a>,
}

impl<'a, Message> IconButton<'a, Message> {
    fn new(theme: &Theme, icon: svg::Handle, class: StyleFn<'a>) -> Self {
        IconButton {
            icon,
            selected_icon: None,
            selected: None,
            on_press: None,
            metrics: theme.components.icon_button,
            tokens: Tokens::new(theme),
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

    /// Makes the button a toggle with the given selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Icon shown while selected.
    pub fn selected_icon(mut self, icon: svg::Handle) -> Self {
        self.selected_icon = Some(icon);
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, Toggle) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn toggle(&self) -> Toggle {
        match self.selected {
            None => Toggle::None,
            Some(false) => Toggle::Unselected,
            Some(true) => Toggle::Selected,
        }
    }
}

/// A standard icon button.
pub fn standard<'a, Message>(theme: &Theme, icon: svg::Handle) -> IconButton<'a, Message> {
    IconButton::new(theme, icon, Box::new(standard_style))
}

/// A filled icon button.
pub fn filled<'a, Message>(theme: &Theme, icon: svg::Handle) -> IconButton<'a, Message> {
    IconButton::new(theme, icon, Box::new(filled_style))
}

/// A filled tonal icon button.
pub fn filled_tonal<'a, Message>(theme: &Theme, icon: svg::Handle) -> IconButton<'a, Message> {
    IconButton::new(theme, icon, Box::new(filled_tonal_style))
}

/// An outlined icon button.
pub fn outlined<'a, Message>(theme: &Theme, icon: svg::Handle) -> IconButton<'a, Message> {
    IconButton::new(theme, icon, Box::new(outlined_style))
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for IconButton<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<pressable::State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(pressable::State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.metrics.size),
            Length::Fixed(self.metrics.size),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.metrics.size, self.metrics.size))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_press.is_some() {
            let state = tree.state.downcast_mut::<pressable::State>();
            operation.focusable(None, layout.bounds(), &mut state.interaction.focus);
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
        let state = tree.state.downcast_mut::<pressable::State>();
        let hit = pressable::hit_area(layout.bounds(), self.metrics.target_size);
        let enabled = self.on_press.is_some();
        if state
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
        let state = tree.state.downcast_ref::<pressable::State>();
        let bounds = layout.bounds();
        let radius = self.shape.radius(bounds.size());
        let toggle = self.toggle();
        let (style, elevation) = state.style(self.on_press.is_some(), false, |status| {
            theme.style(&self.class, status, toggle)
        });
        pressable::draw_container(renderer, theme, bounds, radius, &style, elevation, state);
        let icon = match (&self.selected_icon, toggle) {
            (Some(icon), Toggle::Selected) => icon,
            _ => &self.icon,
        };
        let size = self.metrics.icon_size;
        renderer.draw_svg(
            tinted(icon.clone(), style.icon, 1.0),
            Rectangle::new(
                Point::new(
                    bounds.center_x() - size / 2.0,
                    bounds.center_y() - size / 2.0,
                ),
                Size::new(size, size),
            ),
            bounds.intersection(viewport).unwrap_or(bounds),
        );
        pressable::draw_focus(renderer, theme, bounds, radius, state, &self.tokens);
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let hit = pressable::hit_area(layout.bounds(), self.metrics.target_size);
        if self.on_press.is_some() && cursor.is_over(hit) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<IconButton<'a, Message>> for Element<'a, Message> {
    fn from(button: IconButton<'a, Message>) -> Self {
        IcedElement::new(button)
    }
}
