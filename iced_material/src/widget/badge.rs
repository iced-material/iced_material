// SPDX-License-Identifier: LGPL-3.0-only

//! Small and large badges, and placement of a badge on an icon.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::{
    Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, Vector, mouse,
};

use crate::Element;
use crate::draw::surface;
use crate::draw::text::Label;
use crate::shape::Shape;
use crate::theme::Theme;
use crate::typography::TypeStyle;

/// Badge dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Diameter of the small badge.
    pub small_size: f32,
    /// Height and minimum width of the large badge.
    pub large_size: f32,
    /// Horizontal padding of the large badge label.
    pub large_padding: f32,
    /// Offset of the small badge from the horizontal center and the top of the anchor.
    pub small_offset: Vector,
    /// Offset of the large badge from the horizontal center and the top of the anchor.
    pub large_offset: Vector,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            small_size: 6.0,
            large_size: 16.0,
            large_padding: 4.0,
            small_offset: Vector::new(6.0, 4.0),
            large_offset: Vector::new(2.0, 1.0),
        }
    }
}

/// A Material badge.
pub struct Badge {
    value: Option<String>,
    metrics: Metrics,
    type_style: TypeStyle,
    shape: Shape,
    color: iced::Color,
    label_color: iced::Color,
}

/// A small badge without a label.
pub fn small(theme: &Theme) -> Badge {
    Badge {
        value: None,
        metrics: theme.components.badge,
        type_style: theme.typography.label_small,
        shape: theme.shape.full,
        color: theme.colors.error,
        label_color: theme.colors.on_error,
    }
}

/// A large badge with a short label such as a count.
pub fn large(theme: &Theme, value: impl Into<String>) -> Badge {
    Badge {
        value: Some(value.into()),
        ..small(theme)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Badge {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Label>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Label::default())
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
        let m = &self.metrics;
        match &self.value {
            None => layout::Node::new(Size::new(m.small_size, m.small_size)),
            Some(value) => {
                let label = tree.state.downcast_mut::<Label>();
                let width = label.update(value, self.type_style).width + m.large_padding * 2.0;
                layout::Node::new(Size::new(width.max(m.large_size), m.large_size))
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        surface::fill(
            renderer,
            bounds,
            self.shape.radius(bounds.size()),
            self.color,
        );
        if self.value.is_some() {
            let label = tree.state.downcast_ref::<Label>();
            let size = label.size();
            label.draw(
                renderer,
                Point::new(
                    bounds.center_x() - size.width / 2.0,
                    bounds.center_y() - size.height / 2.0,
                ),
                self.label_color,
                *viewport,
            );
        }
    }
}

impl<'a, Message: 'a> From<Badge> for Element<'a, Message> {
    fn from(badge: Badge) -> Self {
        IcedElement::new(badge)
    }
}

/// Places a badge on the top end of its anchor, usually an icon.
pub struct Badged<'a, Message> {
    anchor: Element<'a, Message>,
    badge: Badge,
}

/// Places `badge` on `anchor`.
pub fn badged<'a, Message>(
    anchor: impl Into<Element<'a, Message>>,
    badge: Badge,
) -> Badged<'a, Message> {
    Badged {
        anchor: anchor.into(),
        badge,
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Badged<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.anchor),
            Tree {
                tag: tree::Tag::of::<Label>(),
                state: tree::State::new(Label::default()),
                children: Vec::new(),
            },
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() == 2 {
            tree.children[0].diff(&self.anchor);
        } else {
            tree.children = Widget::<Message, Theme, Renderer>::children(self);
        }
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let anchor = self
            .anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let badge = Widget::<Message, Theme, Renderer>::layout(
            &mut self.badge,
            &mut tree.children[1],
            renderer,
            limits,
        );
        let offset = if self.badge.value.is_some() {
            self.badge.metrics.large_offset
        } else {
            self.badge.metrics.small_offset
        };
        let size = anchor.size();
        let badge = badge.move_to(Point::new(size.width / 2.0 + offset.x, offset.y));
        layout::Node::with_children(size, vec![anchor, badge])
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
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], child, renderer, operation);
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
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            children.next().unwrap(),
            cursor,
            viewport,
        );
        Widget::<Message, Theme, Renderer>::draw(
            &self.badge,
            &tree.children[1],
            renderer,
            theme,
            style,
            children.next().unwrap(),
            cursor,
            viewport,
        );
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
        self.anchor.as_widget().mouse_interaction(
            &tree.children[0],
            child,
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message: 'a> From<Badged<'a, Message>> for Element<'a, Message> {
    fn from(badged: Badged<'a, Message>) -> Self {
        IcedElement::new(badged)
    }
}
