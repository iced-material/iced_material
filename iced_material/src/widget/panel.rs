// SPDX-License-Identifier: LGPL-3.0-only

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay, renderer};
use iced::{
    Color, Element as IcedElement, Event, Length, Padding, Rectangle, Renderer, Size, Vector, mouse,
};

use crate::Element;
use crate::draw::{shadow, surface};
use crate::shape::Shape;
use crate::theme::Theme;

pub(crate) struct Surface {
    pub color: Color,
    pub text: Color,
    pub shape: Shape,
    pub elevation: f32,
    pub padding: Padding,
    pub min_width: f32,
    pub max_width: f32,
}

pub(crate) struct Panel<'a, Message> {
    content: Element<'a, Message>,
    surface: Surface,
    fill_height: bool,
    fill_width: bool,
}

impl<'a, Message> Panel<'a, Message> {
    pub fn new(content: Element<'a, Message>, surface: Surface) -> Self {
        Panel {
            content,
            surface,
            fill_height: false,
            fill_width: false,
        }
    }

    pub fn fill_width(mut self) -> Self {
        self.fill_width = true;
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.fill_height = true;
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Panel<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            if self.fill_width {
                Length::Fill
            } else {
                Length::Shrink
            },
            if self.fill_height {
                Length::Fill
            } else {
                Length::Shrink
            },
        )
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let s = &self.surface;
        let max = Size::new(limits.max().width.min(s.max_width), limits.max().height);
        let min = Size::new(
            limits.min().width.max(s.min_width).min(max.width),
            limits.min().height,
        );
        let limits = layout::Limits::new(min, max);
        let inner = limits.shrink(s.padding);
        let content = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &inner)
            .move_to(iced::Point::new(s.padding.left, s.padding.top));
        let fill_width = self.fill_width && limits.max().width.is_finite();
        let fill_height = self.fill_height && limits.max().height.is_finite();
        let size = limits.resolve(
            if fill_width {
                Length::Fill
            } else {
                Length::Shrink
            },
            if fill_height {
                Length::Fill
            } else {
                Length::Shrink
            },
            content.size().expand(s.padding),
        );
        layout::Node::with_children(size, vec![content])
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
        self.content
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
        let bounds = layout.bounds();
        let s = &self.surface;
        let radius = s.shape.radius(bounds.size());
        shadow::draw(
            renderer,
            bounds,
            radius,
            s.elevation,
            theme.colors.shadow,
            &theme.elevation,
        );
        surface::fill(renderer, bounds, radius, s.color);
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style { text_color: s.text },
            child,
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            child,
            cursor,
            viewport,
            renderer,
        )
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

impl<'a, Message: 'a> From<Panel<'a, Message>> for Element<'a, Message> {
    fn from(panel: Panel<'a, Message>) -> Self {
        IcedElement::new(panel)
    }
}

pub(crate) fn text<'a, Message: 'a>(
    content: impl Into<String>,
    style: crate::typography::TypeStyle,
    color: Color,
) -> Element<'a, Message> {
    iced::widget::text(content.into())
        .size(style.size)
        .line_height(iced::widget::text::LineHeight::Absolute(
            style.line_height.into(),
        ))
        .font(style.font)
        .style(move |_: &Theme| iced::widget::text::Style { color: Some(color) })
        .into()
}
