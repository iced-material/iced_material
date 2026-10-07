// SPDX-License-Identifier: LGPL-3.0-only

//! Top app bars: small, center-aligned, medium and large.
//!
//! The collapse of a medium or large bar while content scrolls is driven by
//! the application through [`TopAppBar::collapse`].

use iced::widget::{Space, column, container, row};
use iced::{Alignment, Length, Padding};

use crate::Element;
use crate::icon::symbol;
use crate::motion::Easing;
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::icon_button;
use crate::widget::panel::{Panel, Surface, text};
use crate::widget::pressable::Style;

/// Top app bar dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of the small and center-aligned bars and of the top row of the others.
    pub height: f32,
    /// Height of a fully expanded medium bar.
    pub medium_height: f32,
    /// Height of a fully expanded large bar.
    pub large_height: f32,
    /// Space between the edge and the first or last icon button.
    pub edge_padding: f32,
    /// Space between icon buttons.
    pub icon_gap: f32,
    /// Width of an icon button.
    pub icon_button: f32,
    /// Start padding of a title without a navigation icon, and of expanded titles.
    pub title_start: f32,
    /// Space below the title of a medium bar.
    pub medium_title_bottom: f32,
    /// Space below the title of a large bar.
    pub large_title_bottom: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            height: 64.0,
            medium_height: 112.0,
            large_height: 152.0,
            edge_padding: 8.0,
            icon_gap: 8.0,
            icon_button: 40.0,
            title_start: 16.0,
            medium_title_bottom: 24.0,
            large_title_bottom: 28.0,
        }
    }
}

/// Top app bar type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Title at the start of one row.
    Small,
    /// Title centered in one row.
    CenterAligned,
    /// A row of icons and a larger title below it.
    Medium,
    /// A row of icons and the largest title below it.
    Large,
}

/// A top app bar.
pub struct TopAppBar<'a, Message> {
    kind: Kind,
    title: String,
    navigation: Option<Element<'a, Message>>,
    actions: Vec<Element<'a, Message>>,
    scrolled: bool,
    collapse: f32,
}

fn new<'a, Message>(kind: Kind, title: impl Into<String>) -> TopAppBar<'a, Message> {
    TopAppBar {
        kind,
        title: title.into(),
        navigation: None,
        actions: Vec::new(),
        scrolled: false,
        collapse: 0.0,
    }
}

/// A small top app bar.
pub fn small<'a, Message>(title: impl Into<String>) -> TopAppBar<'a, Message> {
    new(Kind::Small, title)
}

/// A center-aligned top app bar.
pub fn center_aligned<'a, Message>(title: impl Into<String>) -> TopAppBar<'a, Message> {
    new(Kind::CenterAligned, title)
}

/// A medium top app bar.
pub fn medium<'a, Message>(title: impl Into<String>) -> TopAppBar<'a, Message> {
    new(Kind::Medium, title)
}

/// A large top app bar.
pub fn large<'a, Message>(title: impl Into<String>) -> TopAppBar<'a, Message> {
    new(Kind::Large, title)
}

/// An icon button for the navigation slot of a top app bar.
pub fn navigation_button<'a, Message: Clone + 'a>(
    theme: &Theme,
    icon: iced::widget::svg::Handle,
    message: Message,
) -> Element<'a, Message> {
    let color = theme.colors.on_surface;
    icon_button::standard(theme, icon)
        .on_press(message)
        .style(move |theme: &Theme, _, _| Style::content(theme, color, color, color))
        .into()
}

/// The menu button of a top app bar.
pub fn menu_button<'a, Message: Clone + 'a>(
    theme: &Theme,
    message: Message,
) -> Element<'a, Message> {
    navigation_button(theme, symbol::menu(false), message)
}

/// An icon button for the action slots of a top app bar.
pub fn action_button<'a, Message: Clone + 'a>(
    theme: &Theme,
    icon: iced::widget::svg::Handle,
    message: Message,
) -> Element<'a, Message> {
    icon_button::standard(theme, icon).on_press(message).into()
}

impl<'a, Message> TopAppBar<'a, Message> {
    /// Sets the element before the title, usually a [`menu_button`] or a back button.
    pub fn navigation(mut self, navigation: impl Into<Element<'a, Message>>) -> Self {
        self.navigation = Some(navigation.into());
        self
    }

    /// Adds an element at the end, usually an [`action_button`].
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Uses the container color and elevation of a bar over scrolled content.
    pub fn scrolled(mut self, scrolled: bool) -> Self {
        self.scrolled = scrolled;
        self
    }

    /// Sets how far a medium or large bar is collapsed, from 0 (expanded) to 1 (a small bar).
    pub fn collapse(mut self, collapse: f32) -> Self {
        self.collapse = collapse.clamp(0.0, 1.0);
        self
    }
}

impl<'a, Message: Clone + 'a> TopAppBar<'a, Message> {
    /// Builds the bar.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        let m = theme.components.top_app_bar;
        let c = &theme.colors;
        let t = &theme.typography;
        let actions = self.actions.len();
        let slot = |count: usize| {
            if count == 0 {
                m.title_start
            } else {
                m.edge_padding * 2.0
                    + m.icon_button * count as f32
                    + m.icon_gap * (count - 1) as f32
            }
        };
        let left_width = if self.navigation.is_some() {
            m.edge_padding * 2.0 + m.icon_button
        } else {
            m.title_start
        };
        let right_width = slot(actions);
        let mut action_row = row![].spacing(m.icon_gap).align_y(Alignment::Center);
        for action in self.actions {
            action_row = action_row.push(action);
        }
        let navigation: Element<'a, Message> = match self.navigation {
            Some(navigation) => container(navigation)
                .padding(Padding::ZERO.left(m.edge_padding))
                .width(Length::Fixed(left_width))
                .center_y(Length::Fixed(m.height))
                .into(),
            None => Space::new().width(Length::Fixed(left_width)).into(),
        };
        let actions_slot = container(action_row)
            .padding(Padding::ZERO.right(m.edge_padding))
            .align_right(Length::Fixed(right_width))
            .center_y(Length::Fixed(m.height));
        let top_title = |alpha_value: f32| {
            text(
                self.title.clone(),
                t.title_large,
                alpha(c.on_surface, alpha_value),
            )
        };
        let (content, height): (Element<'a, Message>, f32) = match self.kind {
            Kind::Small => (
                row![
                    navigation,
                    container(top_title(1.0))
                        .center_y(Length::Fixed(m.height))
                        .width(Length::Fill),
                    actions_slot
                ]
                .into(),
                m.height,
            ),
            Kind::CenterAligned => {
                let side = left_width.max(right_width);
                let left: Element<'a, Message> =
                    container(navigation).width(Length::Fixed(side)).into();
                (
                    row![
                        left,
                        container(top_title(1.0))
                            .center(Length::Fill)
                            .height(Length::Fixed(m.height)),
                        container(actions_slot).align_right(Length::Fixed(side))
                    ]
                    .into(),
                    m.height,
                )
            }
            Kind::Medium | Kind::Large => {
                let (expanded, style, bottom) = if self.kind == Kind::Medium {
                    (m.medium_height, t.headline_small, m.medium_title_bottom)
                } else {
                    (m.large_height, t.headline_medium, m.large_title_bottom)
                };
                let height = expanded + (m.height - expanded) * self.collapse;
                let fade = Easing::CubicBezier(0.8, 0.0, 0.8, 0.15).apply(self.collapse);
                let top = row![
                    navigation,
                    container(top_title(fade))
                        .center_y(Length::Fixed(m.height))
                        .width(Length::Fill),
                    actions_slot
                ];
                let title = container(text(
                    self.title.clone(),
                    style,
                    alpha(c.on_surface, 1.0 - self.collapse),
                ))
                .padding(Padding {
                    top: 0.0,
                    right: m.title_start,
                    bottom,
                    left: m.title_start,
                });
                (
                    column![top, Space::new().height(Length::Fill), title]
                        .height(Length::Fixed(height))
                        .into(),
                    height,
                )
            }
        };
        let (color, elevation) = if self.scrolled {
            (c.surface_container, 2.0)
        } else {
            (c.surface, 0.0)
        };
        Panel::new(
            container(content).height(Length::Fixed(height)).into(),
            Surface {
                color,
                text: c.on_surface,
                shape: theme.shape.none,
                elevation,
                padding: Padding::ZERO,
                min_width: 0.0,
                max_width: f32::INFINITY,
            },
        )
        .fill_width()
        .into()
    }
}
