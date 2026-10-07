// SPDX-License-Identifier: LGPL-3.0-only

//! The search bar and the docked search view.

use iced::widget::{Space, column, container, row, text_input};
use iced::{Alignment, Length, Padding};

use crate::Element;
use crate::icon::{icon, symbol};
use crate::theme::Theme;
use crate::widget::divider;
use crate::widget::panel::{Panel, Surface};

/// Search dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of the bar and of the header of the view.
    pub height: f32,
    /// Minimum width.
    pub min_width: f32,
    /// Maximum width.
    pub max_width: f32,
    /// Space before the leading icon.
    pub leading_padding: f32,
    /// Space between the leading icon and the text.
    pub icon_text_space: f32,
    /// Space after the last trailing element, and after the text without any.
    pub trailing_padding: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Height of the text.
    pub line_height: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            height: 56.0,
            min_width: 360.0,
            max_width: 720.0,
            leading_padding: 16.0,
            icon_text_space: 16.0,
            trailing_padding: 8.0,
            icon_size: 24.0,
            line_height: 24.0,
        }
    }
}

/// A search bar that turns into a search view when it has results to show.
pub struct Search<'a, Message> {
    value: String,
    placeholder: String,
    leading: iced::widget::svg::Handle,
    trailing: Vec<Element<'a, Message>>,
    results: Option<Element<'a, Message>>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
}

/// A search bar with a value.
pub fn search<'a, Message>(value: impl Into<String>) -> Search<'a, Message> {
    Search {
        value: value.into(),
        placeholder: String::new(),
        leading: symbol::search(false),
        trailing: Vec::new(),
        results: None,
        on_input: None,
        on_submit: None,
    }
}

impl<'a, Message> Search<'a, Message> {
    /// Sets the hint shown while the value is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the message produced with the new text.
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message produced on Enter.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Replaces the leading icon.
    pub fn leading_icon(mut self, icon: iced::widget::svg::Handle) -> Self {
        self.leading = icon;
        self
    }

    /// Adds an element at the end, such as a clear button or an avatar.
    pub fn trailing(mut self, element: impl Into<Element<'a, Message>>) -> Self {
        self.trailing.push(element.into());
        self
    }

    /// Shows the bar as a docked search view with `results` below a divider.
    pub fn results(mut self, results: impl Into<Element<'a, Message>>) -> Self {
        self.results = Some(results.into());
        self
    }
}

impl<'a, Message: Clone + 'a> Search<'a, Message> {
    /// Builds the bar or view.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        let m = theme.components.search;
        let c = &theme.colors;
        let mut body = theme.typography.body_large;
        body.tracking = 0.0;
        let mut input = text_input(&self.placeholder, &self.value)
            .font(body.font)
            .size(body.size)
            .line_height(iced::widget::text::LineHeight::Absolute(
                m.line_height.into(),
            ))
            .padding(0)
            .width(Length::Fill)
            .on_submit_maybe(self.on_submit);
        if let Some(on_input) = self.on_input {
            input = input.on_input(on_input);
        }
        let has_trailing = !self.trailing.is_empty();
        let mut header = row![
            Space::new().width(Length::Fixed(m.leading_padding)),
            icon(self.leading, m.icon_size, c.on_surface),
            Space::new().width(Length::Fixed(m.icon_text_space)),
            input,
        ]
        .align_y(Alignment::Center)
        .height(Length::Fixed(m.height));
        for element in self.trailing {
            header = header.push(element);
        }
        header = header.push(Space::new().width(Length::Fixed(if has_trailing {
            m.trailing_padding
        } else {
            m.leading_padding
        })));
        let content: Element<'a, Message> = match self.results {
            Some(results) => {
                column![header, divider::horizontal(theme).color(c.outline), results].into()
            }
            None => header.into(),
        };
        Panel::new(
            container(content).width(Length::Fill).into(),
            Surface {
                color: c.surface_container_high,
                text: c.on_surface,
                shape: theme.shape.extra_large,
                elevation: 3.0,
                padding: Padding::ZERO,
                min_width: m.min_width,
                max_width: m.max_width,
            },
        )
        .fill_width()
        .into()
    }
}
