// SPDX-License-Identifier: LGPL-3.0-only

//! Standard and modal side sheets.

use iced::widget::{Space, column, container, row};
use iced::{Alignment, Length, Padding};

use crate::Element;
use crate::icon::symbol;
use crate::theme::Theme;
use crate::widget::layer::{Edge, edge_modal};
use crate::widget::panel::{Panel, Surface, text};
use crate::widget::{button, divider, icon_button};

/// Side sheet dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Width of the sheet.
    pub width: f32,
    /// Padding around the content.
    pub padding: f32,
    /// Space between the header, the content and the actions.
    pub gap: f32,
    /// Space between actions.
    pub actions_gap: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            width: 256.0,
            padding: 24.0,
            gap: 16.0,
            actions_gap: 8.0,
        }
    }
}

/// A side sheet with a title, content and actions.
pub struct SideSheet<'a, Message> {
    title: String,
    content: Element<'a, Message>,
    actions: Vec<(String, Message)>,
    on_close: Option<Message>,
    width: Option<f32>,
}

/// A side sheet.
pub fn side_sheet<'a, Message>(
    title: impl Into<String>,
    content: impl Into<Element<'a, Message>>,
) -> SideSheet<'a, Message> {
    SideSheet {
        title: title.into(),
        content: content.into(),
        actions: Vec::new(),
        on_close: None,
        width: None,
    }
}

impl<'a, Message> SideSheet<'a, Message> {
    /// Adds a close button to the header and sets the message it produces.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Adds a text button at the end of the actions row.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.actions.push((label.into(), message));
        self
    }

    /// Sets the width. Material 3 allows 256 to 400 dp.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl<'a, Message: Clone + 'a> SideSheet<'a, Message> {
    fn panel(self, theme: &Theme, modal: bool) -> Element<'a, Message> {
        let m = theme.components.side_sheet;
        let c = &theme.colors;
        let width = self.width.unwrap_or(m.width);
        let mut header = row![
            container(text(
                self.title,
                theme.typography.title_large,
                c.on_surface_variant
            ))
            .width(Length::Fill)
        ]
        .align_y(Alignment::Center);
        if let Some(message) = self.on_close {
            header =
                header.push(icon_button::standard(theme, symbol::close(false)).on_press(message));
        }
        let mut body = column![header, container(self.content).height(Length::Fill)]
            .spacing(m.gap)
            .height(Length::Fill);
        if !self.actions.is_empty() {
            let mut actions = row![Space::new().width(Length::Fill)].spacing(m.actions_gap);
            for (label, message) in self.actions {
                actions = actions.push(button::text(theme, label).on_press(message));
            }
            body = body.push(actions);
        }
        Panel::new(
            body.into(),
            Surface {
                color: if modal {
                    c.surface_container_low
                } else {
                    c.surface
                },
                text: c.on_surface,
                shape: if modal {
                    theme.shape.large_start
                } else {
                    theme.shape.none
                },
                elevation: if modal { 1.0 } else { 0.0 },
                padding: Padding::new(m.padding),
                min_width: width,
                max_width: width,
            },
        )
        .fill_height()
        .into()
    }

    /// Builds a standard sheet, which sits beside the content and has a divider on its start edge.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        row![
            divider::vertical(theme).color(theme.colors.outline),
            self.panel(theme, false)
        ]
        .into()
    }

    /// Builds a modal sheet over `base` that slides in from the end edge while `open` is true.
    pub fn modal(
        self,
        theme: &Theme,
        base: impl Into<Element<'a, Message>>,
        open: bool,
        on_dismiss: Option<Message>,
    ) -> Element<'a, Message> {
        edge_modal(
            theme,
            base.into(),
            self.panel(theme, true),
            open,
            Edge::End,
            on_dismiss,
            true,
        )
    }
}
