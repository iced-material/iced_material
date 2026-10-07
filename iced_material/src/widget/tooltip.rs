// SPDX-License-Identifier: LGPL-3.0-only

//! Plain and rich tooltips that open after the pointer rests on their anchor.

use std::time::Duration;

use iced::widget::{column, container, row};
use iced::{Alignment, Padding};

use crate::Element;
use crate::theme::Theme;
use crate::widget::button;
use crate::widget::layer::{Layered, Mode, Placement};
use crate::widget::panel::{Panel, Surface, text};

/// Tooltip dimensions and timing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Time the pointer must rest on the anchor before a tooltip opens.
    pub delay: Duration,
    /// Space between the anchor and the tooltip.
    pub gap: f32,
    /// Minimum height of a plain tooltip.
    pub plain_min_height: f32,
    /// Minimum width of a plain tooltip.
    pub plain_min_width: f32,
    /// Maximum width of a plain tooltip.
    pub plain_max_width: f32,
    /// Horizontal padding of a plain tooltip.
    pub plain_padding_x: f32,
    /// Vertical padding of a plain tooltip.
    pub plain_padding_y: f32,
    /// Maximum width of a rich tooltip.
    pub rich_max_width: f32,
    /// Horizontal padding of a rich tooltip.
    pub rich_padding_x: f32,
    /// Top padding of a rich tooltip.
    pub rich_padding_top: f32,
    /// Bottom padding of a rich tooltip without actions.
    pub rich_padding_bottom: f32,
    /// Bottom padding of a rich tooltip with actions.
    pub rich_actions_padding_bottom: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            delay: Duration::from_millis(500),
            gap: 4.0,
            plain_min_height: 24.0,
            plain_min_width: 40.0,
            plain_max_width: 200.0,
            plain_padding_x: 8.0,
            plain_padding_y: 4.0,
            rich_max_width: 320.0,
            rich_padding_x: 16.0,
            rich_padding_top: 12.0,
            rich_padding_bottom: 16.0,
            rich_actions_padding_bottom: 8.0,
        }
    }
}

/// A plain tooltip with one short text.
pub fn plain<'a, Message: Clone + 'a>(
    theme: &Theme,
    anchor: impl Into<Element<'a, Message>>,
    content: impl Into<String>,
) -> Element<'a, Message> {
    let m = theme.components.tooltip;
    let c = &theme.colors;
    let body = text(content, theme.typography.body_small, c.inverse_on_surface);
    let body: Element<'a, Message> = container(body)
        .center_y(m.plain_min_height - m.plain_padding_y * 2.0)
        .into();
    let panel = Panel::new(
        body,
        Surface {
            color: c.inverse_surface,
            text: c.inverse_on_surface,
            shape: theme.shape.extra_small,
            elevation: 0.0,
            padding: Padding::from([m.plain_padding_y, m.plain_padding_x]),
            min_width: m.plain_min_width,
            max_width: m.plain_max_width,
        },
    );
    Layered::new(
        theme,
        anchor.into(),
        panel.into(),
        Mode::Tooltip { delay: m.delay },
        Placement::Above(m.gap),
    )
    .into()
}

/// A rich tooltip with an optional subhead and actions.
pub struct Rich<'a, Message> {
    anchor: Element<'a, Message>,
    supporting: String,
    subhead: Option<String>,
    actions: Vec<(String, Message)>,
}

/// A rich tooltip with supporting text.
pub fn rich<'a, Message: Clone + 'a>(
    anchor: impl Into<Element<'a, Message>>,
    supporting: impl Into<String>,
) -> Rich<'a, Message> {
    Rich {
        anchor: anchor.into(),
        supporting: supporting.into(),
        subhead: None,
        actions: Vec::new(),
    }
}

impl<'a, Message: Clone + 'a> Rich<'a, Message> {
    /// Adds a subhead above the supporting text.
    pub fn subhead(mut self, subhead: impl Into<String>) -> Self {
        self.subhead = Some(subhead.into());
        self
    }

    /// Adds a text button below the supporting text.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.actions.push((label.into(), message));
        self
    }

    /// Builds the anchor with its tooltip.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        let m = theme.components.tooltip;
        let c = &theme.colors;
        let t = &theme.typography;
        let mut body = column![].align_x(Alignment::Start);
        if let Some(subhead) = self.subhead {
            body = body.push(text(subhead, t.title_small, c.on_surface_variant));
        }
        body = body.push(text(self.supporting, t.body_medium, c.on_surface_variant));
        let has_actions = !self.actions.is_empty();
        if has_actions {
            let mut actions = row![].spacing(8);
            for (label, message) in self.actions {
                actions = actions.push(button::text(theme, label).on_press(message));
            }
            body = body.push(container(actions).padding(Padding::ZERO.top(8)));
        }
        let panel = Panel::new(
            body.into(),
            Surface {
                color: c.surface_container,
                text: c.on_surface_variant,
                shape: theme.shape.medium,
                elevation: 2.0,
                padding: Padding {
                    top: m.rich_padding_top,
                    right: m.rich_padding_x,
                    bottom: if has_actions {
                        m.rich_actions_padding_bottom
                    } else {
                        m.rich_padding_bottom
                    },
                    left: m.rich_padding_x,
                },
                min_width: 0.0,
                max_width: m.rich_max_width,
            },
        );
        Layered::new(
            theme,
            self.anchor,
            panel.into(),
            Mode::Tooltip { delay: m.delay },
            Placement::Above(m.gap),
        )
        .into()
    }
}
