// SPDX-License-Identifier: LGPL-3.0-only

//! Floating action buttons: small, regular, large and extended.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::widget::svg;
use iced::{Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::text::Label;
use crate::icon::tinted;
use crate::interaction::Tokens;
use crate::shape::Shape;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::{self, Status, Style};

/// FAB dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Container size and icon size of the small FAB.
    pub small: (f32, f32),
    /// Container size and icon size of the regular FAB.
    pub regular: (f32, f32),
    /// Container size and icon size of the large FAB.
    pub large: (f32, f32),
    /// Height of the extended FAB.
    pub extended_height: f32,
    /// Start and end padding of the extended FAB with an icon.
    pub extended_padding_icon: (f32, f32),
    /// Start and end padding of the extended FAB without an icon.
    pub extended_padding: (f32, f32),
    /// Space between icon and label of the extended FAB.
    pub extended_icon_label_space: f32,
    /// Icon size of the extended FAB.
    pub extended_icon_size: f32,
    /// Minimum width and height of the pointer target.
    pub target_size: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            small: (40.0, 24.0),
            regular: (56.0, 24.0),
            large: (96.0, 36.0),
            extended_height: 56.0,
            extended_padding_icon: (16.0, 20.0),
            extended_padding: (20.0, 20.0),
            extended_icon_label_space: 12.0,
            extended_icon_size: 24.0,
            target_size: 48.0,
        }
    }
}

/// FAB color set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Primary container.
    Primary,
    /// Secondary container.
    Secondary,
    /// Tertiary container.
    Tertiary,
    /// Surface container with a primary icon.
    Surface,
}

/// FAB size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FabSize {
    /// 40 dp.
    Small,
    /// 56 dp.
    Regular,
    /// 96 dp.
    Large,
}

/// The appearance catalog of FABs.
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
        Box::new(|theme, status| style(theme, status, Color::Primary, false))
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// FAB style for a color set. Lowered FABs rest at level 1 instead of level 3.
pub fn style(theme: &Theme, status: Status, color: Color, lowered: bool) -> Style {
    let c = &theme.colors;
    let (container, content) = match color {
        Color::Primary => (c.primary_container, c.on_primary_container),
        Color::Secondary => (c.secondary_container, c.on_secondary_container),
        Color::Tertiary => (c.tertiary_container, c.on_tertiary_container),
        Color::Surface => (
            if lowered {
                c.surface_container_low
            } else {
                c.surface_container_high
            },
            c.primary,
        ),
    };
    let (rest, hover) = if lowered { (1.0, 2.0) } else { (3.0, 4.0) };
    Style {
        container,
        elevation: if status == Status::Hovered {
            hover
        } else {
            rest
        },
        ..Style::content(theme, content, content, content)
    }
}

/// A Material floating action button.
pub struct Fab<'a, Message> {
    icon: Option<svg::Handle>,
    label: Option<String>,
    size: FabSize,
    color: Color,
    lowered: bool,
    on_press: Option<Message>,
    metrics: Metrics,
    tokens: Tokens,
    type_style: TypeStyle,
    shape: Shape,
    class: Option<StyleFn<'a>>,
}

impl<'a, Message> Fab<'a, Message> {
    fn new(theme: &Theme, icon: Option<svg::Handle>, label: Option<String>, size: FabSize) -> Self {
        let shape = match (size, label.is_some()) {
            (_, true) | (FabSize::Regular, false) => theme.shape.large,
            (FabSize::Small, false) => theme.shape.medium,
            (FabSize::Large, false) => theme.shape.extra_large,
        };
        Fab {
            icon,
            label,
            size,
            color: Color::Primary,
            lowered: false,
            on_press: None,
            metrics: theme.components.fab,
            tokens: Tokens::new(theme),
            type_style: theme.typography.label_large,
            shape,
            class: None,
        }
    }

    /// Sets the message produced on press.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the color set.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Uses the lowered elevation.
    pub fn lowered(mut self, lowered: bool) -> Self {
        self.lowered = lowered;
        self
    }

    /// Adds an icon to an extended FAB.
    pub fn icon(mut self, icon: svg::Handle) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
        self.class = Some(Box::new(style));
        self
    }

    fn icon_size(&self) -> (f32, f32) {
        match self.size {
            FabSize::Small => self.metrics.small,
            FabSize::Regular => self.metrics.regular,
            FabSize::Large => self.metrics.large,
        }
    }

    fn padding(&self) -> (f32, f32) {
        if self.icon.is_some() {
            self.metrics.extended_padding_icon
        } else {
            self.metrics.extended_padding
        }
    }
}

/// A regular FAB.
pub fn fab<'a, Message>(theme: &Theme, icon: svg::Handle) -> Fab<'a, Message> {
    Fab::new(theme, Some(icon), None, FabSize::Regular)
}

/// A small FAB.
pub fn small<'a, Message>(theme: &Theme, icon: svg::Handle) -> Fab<'a, Message> {
    Fab::new(theme, Some(icon), None, FabSize::Small)
}

/// A large FAB.
pub fn large<'a, Message>(theme: &Theme, icon: svg::Handle) -> Fab<'a, Message> {
    Fab::new(theme, Some(icon), None, FabSize::Large)
}

/// An extended FAB with a label.
pub fn extended<'a, Message>(theme: &Theme, label: impl Into<String>) -> Fab<'a, Message> {
    Fab::new(theme, None, Some(label.into()), FabSize::Regular)
}

#[derive(Default)]
struct State {
    press: pressable::State,
    label: Label,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Fab<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
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
        let state = tree.state.downcast_mut::<State>();
        match &self.label {
            Some(label) => {
                let label = state.label.update(label, self.type_style);
                let (start, end) = self.padding();
                let icon = if self.icon.is_some() {
                    self.metrics.extended_icon_size + self.metrics.extended_icon_label_space
                } else {
                    0.0
                };
                layout::Node::new(Size::new(
                    start + icon + label.width + end,
                    self.metrics.extended_height,
                ))
            }
            None => {
                let (size, _) = self.icon_size();
                layout::Node::new(Size::new(size, size))
            }
        }
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
        let hit = pressable::hit_area(layout.bounds(), self.metrics.target_size);
        let enabled = self.on_press.is_some();
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
        let (style, elevation) = state.press.style(true, false, |status| match &self.class {
            Some(class) => theme.style(class, status),
            None => style(theme, status, self.color, self.lowered),
        });
        pressable::draw_container(
            renderer,
            theme,
            bounds,
            radius,
            &style,
            elevation,
            &state.press,
        );
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let draw_icon = |renderer: &mut Renderer, icon: &svg::Handle, x: f32, size: f32| {
            renderer.draw_svg(
                tinted(icon.clone(), style.icon, 1.0),
                Rectangle::new(
                    Point::new(x, bounds.center_y() - size / 2.0),
                    Size::new(size, size),
                ),
                clip,
            );
        };
        match &self.label {
            Some(_) => {
                let (start, _) = self.padding();
                let mut x = bounds.x + start;
                if let Some(icon) = &self.icon {
                    draw_icon(renderer, icon, x, self.metrics.extended_icon_size);
                    x += self.metrics.extended_icon_size + self.metrics.extended_icon_label_space;
                }
                let size = state.label.size();
                state.label.draw(
                    renderer,
                    Point::new(x, bounds.center_y() - size.height / 2.0),
                    style.label,
                    clip,
                );
            }
            None => {
                if let Some(icon) = &self.icon {
                    let (_, size) = self.icon_size();
                    draw_icon(renderer, icon, bounds.center_x() - size / 2.0, size);
                }
            }
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
        let hit = pressable::hit_area(layout.bounds(), self.metrics.target_size);
        if self.on_press.is_some() && cursor.is_over(hit) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<Fab<'a, Message>> for Element<'a, Message> {
    fn from(fab: Fab<'a, Message>) -> Self {
        IcedElement::new(fab)
    }
}
