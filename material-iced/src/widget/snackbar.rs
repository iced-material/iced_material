// SPDX-License-Identifier: LGPL-3.0-only

//! Snackbars with an optional action and a dismiss button.
//!
//! A snackbar is a plain widget. The application decides when to show it,
//! where to place it and when to hide it.

use iced::widget::{container, row};
use iced::{Alignment, Length, Padding};

use crate::Element;
use crate::icon::symbol;
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::panel::{Panel, Surface, text};
use crate::widget::pressable::Style;
use crate::widget::{button, icon_button};

/// Snackbar dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Start padding of the text.
    pub padding_start: f32,
    /// End padding when the snackbar has an action or a dismiss button.
    pub padding_end_buttons: f32,
    /// End padding when the snackbar has only text.
    pub padding_end: f32,
    /// Top and bottom padding of the text. One line of text gives a 48 dp container.
    pub text_vertical: f32,
    /// Maximum width.
    pub max_width: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            padding_start: 16.0,
            padding_end_buttons: 8.0,
            padding_end: 16.0,
            text_vertical: 14.0,
            max_width: 600.0,
        }
    }
}

/// A Material snackbar.
pub struct Snackbar<Message> {
    message: String,
    action: Option<(String, Message)>,
    on_dismiss: Option<Message>,
}

/// A snackbar with a message.
pub fn snackbar<Message>(message: impl Into<String>) -> Snackbar<Message> {
    Snackbar {
        message: message.into(),
        action: None,
        on_dismiss: None,
    }
}

impl<Message> Snackbar<Message> {
    /// Adds a text button after the message.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// Adds a close button and sets the message it produces.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }
}

impl<Message: Clone> Snackbar<Message> {
    /// Builds the snackbar widget.
    pub fn build<'a>(self, theme: &Theme) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let m = theme.components.snackbar;
        let c = &theme.colors;
        let has_buttons = self.action.is_some() || self.on_dismiss.is_some();
        let body = container(text(
            self.message,
            theme.typography.body_medium,
            c.inverse_on_surface,
        ))
        .padding(Padding::from([m.text_vertical, 0.0]))
        .width(Length::Fill);
        let mut content = row![body].align_y(Alignment::Center);
        if let Some((label, message)) = self.action {
            let accent = c.inverse_primary;
            content = content.push(button::text(theme, label).on_press(message).style(
                move |theme: &Theme, status| {
                    let color = if status == crate::widget::pressable::Status::Disabled {
                        alpha(theme.colors.inverse_on_surface, theme.disabled.content)
                    } else {
                        accent
                    };
                    Style::content(theme, color, color, color)
                },
            ));
        }
        if let Some(message) = self.on_dismiss {
            let color = c.inverse_on_surface;
            content = content.push(
                icon_button::standard(theme, symbol::close(false))
                    .on_press(message)
                    .style(move |theme: &Theme, _, _| Style::content(theme, color, color, color)),
            );
        }
        Panel::new(
            content.into(),
            Surface {
                color: c.inverse_surface,
                text: c.inverse_on_surface,
                shape: theme.shape.extra_small,
                elevation: 3.0,
                padding: Padding {
                    top: 0.0,
                    right: if has_buttons {
                        m.padding_end_buttons
                    } else {
                        m.padding_end
                    },
                    bottom: 0.0,
                    left: m.padding_start,
                },
                min_width: 0.0,
                max_width: m.max_width,
            },
        )
        .into()
    }
}
