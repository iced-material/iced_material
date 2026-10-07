// SPDX-License-Identifier: LGPL-3.0-only

//! Basic dialogs shown in a modal layer over the content of a window.

use iced::widget::{column, container, row, svg};
use iced::{Alignment, Length, Padding};

use crate::Element;
use crate::icon::icon;
use crate::theme::Theme;
use crate::widget::button;
use crate::widget::layer::{Layered, Mode, Placement};
use crate::widget::panel::{Panel, Surface, text};

/// Dialog dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Minimum width.
    pub min_width: f32,
    /// Maximum width.
    pub max_width: f32,
    /// Padding around the headline and the supporting text.
    pub padding: f32,
    /// Space above the headline of a dialog with an icon.
    pub icon_headline_top: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Space between actions.
    pub actions_gap: f32,
    /// Top padding of the actions.
    pub actions_top: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            min_width: 280.0,
            max_width: 560.0,
            padding: 24.0,
            icon_headline_top: 16.0,
            icon_size: 24.0,
            actions_gap: 8.0,
            actions_top: 16.0,
        }
    }
}

/// A basic dialog: optional icon, headline, supporting text or content, and actions.
pub struct Dialog<'a, Message> {
    icon: Option<svg::Handle>,
    headline: Option<String>,
    supporting: Option<String>,
    content: Option<Element<'a, Message>>,
    actions: Vec<(String, Message)>,
}

/// An empty dialog.
pub fn dialog<'a, Message>() -> Dialog<'a, Message> {
    Dialog {
        icon: None,
        headline: None,
        supporting: None,
        content: None,
        actions: Vec::new(),
    }
}

impl<'a, Message> Dialog<'a, Message> {
    /// Adds an icon above the headline. The headline is then centered.
    pub fn icon(mut self, icon: svg::Handle) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the headline.
    pub fn headline(mut self, headline: impl Into<String>) -> Self {
        self.headline = Some(headline.into());
        self
    }

    /// Sets the supporting text.
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Sets arbitrary content below the headline.
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Adds a text button at the end of the actions row.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.actions.push((label.into(), message));
        self
    }
}

impl<'a, Message: Clone + 'a> Dialog<'a, Message> {
    fn build(self, theme: &Theme) -> Element<'a, Message> {
        let m = theme.components.dialog;
        let c = &theme.colors;
        let t = &theme.typography;
        let has_icon = self.icon.is_some();
        let mut body = column![].width(Length::Fill);
        if let Some(handle) = self.icon {
            body = body.push(
                container(icon(handle, m.icon_size, c.secondary))
                    .center_x(Length::Fill)
                    .padding(Padding::ZERO.top(m.padding)),
            );
        }
        if let Some(headline) = self.headline {
            let top = if has_icon {
                m.icon_headline_top
            } else {
                m.padding
            };
            let headline = text(headline, t.headline_small, c.on_surface);
            body = body.push(
                container(headline)
                    .width(Length::Fill)
                    .align_x(if has_icon {
                        Alignment::Center
                    } else {
                        Alignment::Start
                    })
                    .padding(Padding {
                        top,
                        right: m.padding,
                        bottom: 0.0,
                        left: m.padding,
                    }),
            );
        }
        let content = self.content.or_else(|| {
            self.supporting
                .map(|s| text(s, t.body_medium, c.on_surface_variant))
        });
        if let Some(content) = content {
            body = body.push(container(content).padding(m.padding));
        }
        if !self.actions.is_empty() {
            let mut actions = row![].spacing(m.actions_gap);
            for (label, message) in self.actions {
                actions = actions.push(button::text(theme, label).on_press(message));
            }
            body = body.push(
                container(actions)
                    .width(Length::Fill)
                    .align_x(Alignment::End)
                    .padding(Padding {
                        top: m.actions_top,
                        right: m.padding,
                        bottom: m.padding,
                        left: m.padding,
                    }),
            );
        }
        Panel::new(
            body.into(),
            Surface {
                color: c.surface_container_high,
                text: c.on_surface_variant,
                shape: theme.shape.extra_large,
                elevation: 3.0,
                padding: Padding::ZERO,
                min_width: m.min_width,
                max_width: m.max_width,
            },
        )
        .into()
    }
}

/// Shows `dialog` over `base` while `open` is true.
///
/// The dialog opens and closes with an animation, closes on Escape and keeps
/// keyboard focus inside itself. `on_dismiss` is published for Escape and,
/// with `dismiss_on_scrim`, for a click outside the dialog.
pub struct Modal<'a, Message> {
    base: Element<'a, Message>,
    dialog: Dialog<'a, Message>,
    open: bool,
    on_dismiss: Option<Message>,
    dismiss_on_scrim: bool,
}

/// A modal dialog layer.
pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    dialog: Dialog<'a, Message>,
    open: bool,
) -> Modal<'a, Message> {
    Modal {
        base: base.into(),
        dialog,
        open,
        on_dismiss: None,
        dismiss_on_scrim: false,
    }
}

impl<'a, Message: Clone + 'a> Modal<'a, Message> {
    /// Sets the message produced when the user asks to close the dialog.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Also dismisses on a click outside the dialog.
    pub fn dismiss_on_scrim(mut self, dismiss: bool) -> Self {
        self.dismiss_on_scrim = dismiss;
        self
    }

    /// Builds the layer.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        Layered::new(
            theme,
            self.base,
            self.dialog.build(theme),
            Mode::Modal {
                open: self.open,
                on_dismiss: self.on_dismiss,
                dismiss_on_scrim: self.dismiss_on_scrim,
            },
            Placement::Center,
        )
        .into()
    }
}
