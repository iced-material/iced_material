// SPDX-License-Identifier: LGPL-3.0-only

//! Cards: elevated, filled and outlined, static or pressable.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay, renderer};
use iced::{
    Element as IcedElement, Event, Length, Padding, Rectangle, Renderer, Size, Vector, mouse,
};

use crate::Element;
use crate::interaction::Tokens;
use crate::shape::Shape;
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::pressable::{self, Status, Style};

/// Card type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Elevated card.
    Elevated,
    /// Filled card.
    Filled,
    /// Outlined card.
    Outlined,
}

/// The appearance catalog of cards.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, kind: Kind) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, Kind) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, kind: Kind) -> Style {
        class(self, status, kind)
    }
}

/// The baseline card style.
pub fn style(theme: &Theme, status: Status, kind: Kind) -> Style {
    let c = &theme.colors;
    let base = Style::content(theme, c.on_surface, c.primary, c.on_surface);
    let disabled = theme.disabled.content;
    match kind {
        Kind::Elevated => Style {
            container: if status == Status::Disabled {
                alpha(c.surface, disabled)
            } else {
                c.surface_container_low
            },
            elevation: match status {
                Status::Hovered => 2.0,
                Status::Dragged => 4.0,
                _ => 1.0,
            },
            ..base
        },
        Kind::Filled => Style {
            container: if status == Status::Disabled {
                alpha(c.surface_variant, disabled)
            } else {
                c.surface_container_highest
            },
            elevation: match status {
                Status::Hovered => 1.0,
                Status::Dragged => 3.0,
                _ => 0.0,
            },
            ..base
        },
        Kind::Outlined => Style {
            container: c.surface,
            outline: match status {
                Status::Focused => c.on_surface,
                Status::Disabled => alpha(c.outline, theme.disabled.outline),
                _ => c.outline_variant,
            },
            outline_width: 1.0,
            elevation: match status {
                Status::Hovered => 1.0,
                Status::Dragged => 3.0,
                _ => 0.0,
            },
            ..base
        },
    }
}

/// A Material card.
pub struct Card<'a, Message> {
    content: Element<'a, Message>,
    kind: Kind,
    on_press: Option<Message>,
    disabled: bool,
    width: Length,
    height: Length,
    padding: Padding,
    tokens: Tokens,
    shape: Shape,
    class: StyleFn<'a>,
}

impl<'a, Message> Card<'a, Message> {
    fn new(theme: &Theme, kind: Kind, content: Element<'a, Message>) -> Self {
        Card {
            content,
            kind,
            on_press: None,
            disabled: false,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::ZERO,
            tokens: Tokens::new(theme),
            shape: theme.shape.medium,
            class: Box::new(style),
        }
    }

    /// Makes the whole card pressable.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Draws the card with the disabled style and ignores presses.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the padding around the content.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, Kind) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn pressable(&self) -> bool {
        self.on_press.is_some() && !self.disabled
    }
}

/// An elevated card.
pub fn elevated<'a, Message>(
    theme: &Theme,
    content: impl Into<Element<'a, Message>>,
) -> Card<'a, Message> {
    Card::new(theme, Kind::Elevated, content.into())
}

/// A filled card.
pub fn filled<'a, Message>(
    theme: &Theme,
    content: impl Into<Element<'a, Message>>,
) -> Card<'a, Message> {
    Card::new(theme, Kind::Filled, content.into())
}

/// An outlined card.
pub fn outlined<'a, Message>(
    theme: &Theme,
    content: impl Into<Element<'a, Message>>,
) -> Card<'a, Message> {
    Card::new(theme, Kind::Outlined, content.into())
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Card<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<pressable::State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(pressable::State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let Some(child) = layout.children().next() else {
            return;
        };
        if self.pressable() {
            let state = tree.state.downcast_mut::<pressable::State>();
            operation.focusable(None, layout.bounds(), &mut state.interaction.focus);
        }
        operation.traverse(&mut |operation| {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], child, renderer, operation);
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let Some(child) = layout.children().next() else {
            return;
        };
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if self.on_press.is_none() || shell.is_event_captured() {
            return;
        }
        let state = tree.state.downcast_mut::<pressable::State>();
        let enabled = self.pressable();
        if state
            .update(
                event,
                layout.bounds(),
                cursor,
                enabled,
                false,
                &self.tokens,
                shell,
            )
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
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(child) = layout.children().next() else {
            return;
        };
        let state = tree.state.downcast_ref::<pressable::State>();
        let bounds = layout.bounds();
        let radius = self.shape.radius(bounds.size());
        let (card, elevation) = state.style(!self.disabled, false, |status| {
            theme.style(&self.class, status, self.kind)
        });
        pressable::draw_container(renderer, theme, bounds, radius, &card, elevation, state);
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: card.label,
            },
            child,
            cursor,
            viewport,
        );
        pressable::draw_focus(renderer, theme, bounds, radius, state, &self.tokens);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(child) = layout.children().next() else {
            return mouse::Interaction::default();
        };
        let content = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            child,
            cursor,
            viewport,
            renderer,
        );
        if content == mouse::Interaction::default()
            && self.pressable()
            && cursor.is_over(layout.bounds())
        {
            mouse::Interaction::Pointer
        } else {
            content
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let child = layout.children().next()?;
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            child,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<Card<'a, Message>> for Element<'a, Message> {
    fn from(card: Card<'a, Message>) -> Self {
        IcedElement::new(card)
    }
}
