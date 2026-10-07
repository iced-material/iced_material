// SPDX-License-Identifier: LGPL-3.0-only

//! Horizontal and vertical dividers.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::border::Radius;
use iced::{Color, Element as IcedElement, Length, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::surface;
use crate::theme::Theme;

/// Divider dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Line thickness.
    pub thickness: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics { thickness: 1.0 }
    }
}

/// A Material divider.
pub struct Divider {
    vertical: bool,
    inset: (f32, f32),
    thickness: f32,
    color: Color,
}

/// A horizontal divider filling the available width.
pub fn horizontal(theme: &Theme) -> Divider {
    Divider {
        vertical: false,
        inset: (0.0, 0.0),
        thickness: theme.components.divider.thickness,
        color: theme.colors.outline_variant,
    }
}

/// A vertical divider filling the available height.
pub fn vertical(theme: &Theme) -> Divider {
    Divider {
        vertical: true,
        ..horizontal(theme)
    }
}

impl Divider {
    /// Leaves space before and after the line, along its length.
    pub fn inset(mut self, start: f32, end: f32) -> Self {
        self.inset = (start, end);
        self
    }

    /// Replaces the color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Divider {
    fn size(&self) -> Size<Length> {
        if self.vertical {
            Size::new(Length::Fixed(self.thickness), Length::Fill)
        } else {
            Size::new(Length::Fill, Length::Fixed(self.thickness))
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = <Self as Widget<Message, Theme, Renderer>>::size(self);
        layout::Node::new(limits.resolve(size.width, size.height, Size::ZERO))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let (start, end) = self.inset;
        let line = if self.vertical {
            Rectangle {
                y: b.y + start,
                height: (b.height - start - end).max(0.0),
                ..b
            }
        } else {
            Rectangle {
                x: b.x + start,
                width: (b.width - start - end).max(0.0),
                ..b
            }
        };
        surface::fill(renderer, line, Radius::default(), self.color);
    }
}

impl<'a, Message: 'a> From<Divider> for Element<'a, Message> {
    fn from(divider: Divider) -> Self {
        IcedElement::new(divider)
    }
}
