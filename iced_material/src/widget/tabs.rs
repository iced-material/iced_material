// SPDX-License-Identifier: LGPL-3.0-only

//! Primary and secondary tabs with a sliding active indicator.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::{
    Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse, window,
};

use crate::Element;
use crate::draw::text::Label;
use crate::draw::{focus_ring, surface};
use crate::icon::tinted;
use crate::interaction::Tokens;
use crate::motion::{Transition, Tween};
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::{self, Status};

/// Tab dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of a tab with an icon or a label.
    pub height: f32,
    /// Height of a primary tab with an icon and a label.
    pub icon_label_height: f32,
    /// Start and end padding of the label.
    pub padding: f32,
    /// Minimum width of a tab that is not stretched.
    pub min_width: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Space between the icon and the label.
    pub icon_label_space: f32,
    /// Height of the primary indicator.
    pub primary_indicator: f32,
    /// Height of the secondary indicator.
    pub secondary_indicator: f32,
    /// Minimum width of the primary indicator.
    pub primary_indicator_min_width: f32,
    /// Height of the divider below the tabs.
    pub divider: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            height: 48.0,
            icon_label_height: 64.0,
            padding: 16.0,
            min_width: 90.0,
            icon_size: 24.0,
            icon_label_space: 2.0,
            primary_indicator: 3.0,
            secondary_indicator: 2.0,
            primary_indicator_min_width: 24.0,
            divider: 1.0,
        }
    }
}

/// Primary or secondary tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Tabs at the top of a page, with a short indicator under the content.
    Primary,
    /// Tabs inside a page, with an indicator as wide as the tab.
    Secondary,
}

/// The appearance of a tab in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Label color.
    pub label: Color,
    /// Icon color.
    pub icon: Color,
    /// Container color.
    pub container: Color,
    /// Indicator color.
    pub indicator: Color,
    /// Divider color.
    pub divider: Color,
    /// Hover state layer color.
    pub hover_layer: Color,
    /// Press ripple color.
    pub pressed_layer: Color,
}

/// The appearance catalog of tabs.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class for a tab.
    fn style(&self, class: &Self::Class<'_>, kind: Kind, status: Status, selected: bool) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Kind, Status, bool) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, kind: Kind, status: Status, selected: bool) -> Style {
        class(self, kind, status, selected)
    }
}

/// The baseline tab style.
pub fn style(theme: &Theme, kind: Kind, _status: Status, selected: bool) -> Style {
    let c = &theme.colors;
    let (content, hover, pressed) = match (kind, selected) {
        (Kind::Primary, true) => (c.primary, c.primary, c.primary),
        (Kind::Primary, false) => (c.on_surface_variant, c.on_surface, c.primary),
        (Kind::Secondary, true) => (c.on_surface, c.on_surface, c.on_surface),
        (Kind::Secondary, false) => (c.on_surface_variant, c.on_surface, c.on_surface),
    };
    Style {
        label: content,
        icon: content,
        container: c.surface,
        indicator: c.primary,
        divider: c.outline_variant,
        hover_layer: hover,
        pressed_layer: pressed,
    }
}

/// One tab.
pub struct Tab {
    label: Option<String>,
    icon: Option<iced::widget::svg::Handle>,
    selected_icon: Option<iced::widget::svg::Handle>,
}

impl Tab {
    /// A tab with a label.
    pub fn new(label: impl Into<String>) -> Self {
        Tab {
            label: Some(label.into()),
            icon: None,
            selected_icon: None,
        }
    }

    /// A tab with only an icon.
    pub fn icon_only(icon: iced::widget::svg::Handle) -> Self {
        Tab {
            label: None,
            icon: Some(icon),
            selected_icon: None,
        }
    }

    /// Adds an icon above the label.
    pub fn icon(mut self, icon: iced::widget::svg::Handle) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the icon shown while the tab is selected.
    pub fn selected_icon(mut self, icon: iced::widget::svg::Handle) -> Self {
        self.selected_icon = Some(icon);
        self
    }
}

/// A row of tabs.
pub struct Tabs<'a, Message> {
    tabs: Vec<Tab>,
    selected: usize,
    on_select: Box<dyn Fn(usize) -> Message + 'a>,
    kind: Kind,
    stretch: bool,
    metrics: Metrics,
    tokens: Tokens,
    transition: Transition,
    label_style: TypeStyle,
    class: StyleFn<'a>,
}

fn new<'a, Message>(
    theme: &Theme,
    kind: Kind,
    tabs: Vec<Tab>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Tabs<'a, Message> {
    Tabs {
        tabs,
        selected,
        on_select: Box::new(on_select),
        kind,
        stretch: true,
        metrics: theme.components.tabs,
        tokens: Tokens::new(theme),
        transition: Transition {
            duration: theme.motion.duration.medium1,
            easing: theme.motion.easing.emphasized,
        },
        label_style: theme.typography.title_small,
        class: Box::new(style),
    }
}

/// Primary tabs.
pub fn primary<'a, Message>(
    theme: &Theme,
    tabs: Vec<Tab>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Tabs<'a, Message> {
    new(theme, Kind::Primary, tabs, selected, on_select)
}

/// Secondary tabs.
pub fn secondary<'a, Message>(
    theme: &Theme,
    tabs: Vec<Tab>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Tabs<'a, Message> {
    new(theme, Kind::Secondary, tabs, selected, on_select)
}

impl<'a, Message> Tabs<'a, Message> {
    /// Chooses between tabs that share the width (the default) and tabs sized to their content.
    pub fn stretch(mut self, stretch: bool) -> Self {
        self.stretch = stretch;
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Kind, Status, bool) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn row_height(&self) -> f32 {
        let both = self.kind == Kind::Primary
            && self
                .tabs
                .iter()
                .any(|t| t.icon.is_some() && t.label.is_some());
        let m = &self.metrics;
        (if both { m.icon_label_height } else { m.height }) + m.divider
    }
}

struct TabState {
    press: pressable::State,
    label: Label,
}

struct TabsState {
    items: Vec<TabState>,
    indicator: Tween,
    from: Rectangle,
    to: Rectangle,
    selected: usize,
    now: Option<iced::time::Instant>,
}

impl TabsState {
    fn current(&self) -> Rectangle {
        let t = self.indicator.value(self.now);
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Rectangle::new(
            Point::new(mix(self.from.x, self.to.x), mix(self.from.y, self.to.y)),
            Size::new(
                mix(self.from.width, self.to.width),
                mix(self.from.height, self.to.height),
            ),
        )
    }
}

impl<Message> Tabs<'_, Message> {
    fn rects(&self, bounds: Rectangle, state: &TabsState) -> Vec<Rectangle> {
        let m = &self.metrics;
        let n = self.tabs.len().max(1);
        let height = self.row_height() - m.divider;
        let mut x = bounds.x;
        self.tabs
            .iter()
            .zip(&state.items)
            .map(|(tab, item)| {
                let width = if self.stretch {
                    bounds.width / n as f32
                } else {
                    let content = if tab.label.is_some() {
                        item.label.size().width.max(if tab.icon.is_some() {
                            m.icon_size
                        } else {
                            0.0
                        })
                    } else {
                        m.icon_size
                    };
                    (content + m.padding * 2.0).max(m.min_width)
                };
                let r = Rectangle::new(Point::new(x, bounds.y), Size::new(width, height));
                x += width;
                r
            })
            .collect()
    }

    fn content_width(&self, index: usize, state: &TabsState) -> f32 {
        let m = &self.metrics;
        let tab = &self.tabs[index];
        let label = tab
            .label
            .as_ref()
            .map_or(0.0, |_| state.items[index].label.size().width);
        let icon = if tab.icon.is_some() { m.icon_size } else { 0.0 };
        label.max(icon)
    }

    fn indicator_rect(&self, rect: Rectangle, index: usize, state: &TabsState) -> Rectangle {
        let m = &self.metrics;
        match self.kind {
            Kind::Primary => {
                let width = self
                    .content_width(index, state)
                    .max(m.primary_indicator_min_width);
                Rectangle::new(
                    Point::new(
                        rect.center_x() - width / 2.0,
                        rect.y + rect.height - m.primary_indicator,
                    ),
                    Size::new(width, m.primary_indicator),
                )
            }
            Kind::Secondary => Rectangle::new(
                Point::new(rect.x, rect.y + rect.height - m.secondary_indicator),
                Size::new(rect.width, m.secondary_indicator),
            ),
        }
    }

    fn roving(&self, state: &TabsState) -> usize {
        (0..self.tabs.len())
            .find(|i| state.items[*i].press.interaction.focus.focused)
            .unwrap_or(self.selected.min(self.tabs.len().saturating_sub(1)))
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Tabs<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabsState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TabsState {
            items: self
                .tabs
                .iter()
                .map(|_| TabState {
                    press: pressable::State::default(),
                    label: Label::default(),
                })
                .collect(),
            indicator: Tween::new(1.0),
            from: Rectangle::with_size(Size::ZERO),
            to: Rectangle::with_size(Size::ZERO),
            selected: usize::MAX,
            now: None,
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            if self.stretch {
                Length::Fill
            } else {
                Length::Shrink
            },
            Length::Fixed(self.row_height()),
        )
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<TabsState>();
        while state.items.len() < self.tabs.len() {
            state.items.push(TabState {
                press: pressable::State::default(),
                label: Label::default(),
            });
        }
        state.items.truncate(self.tabs.len());
        for (tab, item) in self.tabs.iter().zip(&mut state.items) {
            item.label
                .update(tab.label.as_deref().unwrap_or(""), self.label_style);
        }
        let m = &self.metrics;
        let intrinsic: f32 = self
            .tabs
            .iter()
            .zip(&state.items)
            .map(|(tab, item)| {
                let content =
                    item.label
                        .size()
                        .width
                        .max(if tab.icon.is_some() { m.icon_size } else { 0.0 });
                (content + m.padding * 2.0).max(m.min_width)
            })
            .sum();
        let width = if self.stretch {
            Length::Fill
        } else {
            Length::Shrink
        };
        let height = self.row_height();
        layout::Node::new(limits.resolve(
            width,
            Length::Fixed(height),
            Size::new(intrinsic, height),
        ))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<TabsState>();
        if state.items.is_empty() {
            return;
        }
        let rects = self.rects(layout.bounds(), state);
        let i = self.roving(state);
        operation.focusable(None, rects[i], &mut state.items[i].press.interaction.focus);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<TabsState>();
        let n = self.tabs.len();
        if n == 0 {
            return;
        }
        let bounds = layout.bounds();
        let rects = self.rects(bounds, state);
        let selected = self.selected.min(n - 1);
        let target = self.indicator_rect(rects[selected], selected, state);
        if state.selected == usize::MAX {
            state.from = target;
            state.to = target;
            state.selected = selected;
        } else if state.selected != selected || state.to != target {
            state.from = if state.selected == selected {
                target
            } else {
                state.current()
            };
            state.to = target;
            state.selected = selected;
            state.indicator = Tween::new(0.0);
            state.indicator.go(1.0, self.transition, state.now);
            shell.request_redraw();
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if state.indicator.tick(*now) {
                shell.request_redraw();
            }
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            ..
        }) = event
            && let Some(current) = (0..n).find(|i| state.items[*i].press.interaction.focus.focused)
        {
            let next = match key {
                Named::ArrowRight => Some((current + 1) % n),
                Named::ArrowLeft => Some((current + n - 1) % n),
                Named::Home => Some(0),
                Named::End => Some(n - 1),
                _ => None,
            };
            if let Some(next) = next {
                for (i, item) in state.items.iter_mut().enumerate() {
                    let focus = &mut item.press.interaction.focus;
                    focus.focused = i == next;
                    focus.visible = i == next;
                    if i == next {
                        focus.since = None;
                    }
                }
                shell.publish((self.on_select)(next));
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        for (i, (rect, item)) in rects.into_iter().zip(&mut state.items).enumerate() {
            if item
                .press
                .update(event, rect, cursor, true, false, &self.tokens, shell)
                .is_some()
            {
                shell.publish((self.on_select)(i));
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TabsState>();
        let bounds = layout.bounds();
        let m = &self.metrics;
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let rects = self.rects(bounds, state);
        let base = theme.style(&self.class, self.kind, Status::Active, false);
        surface::fill(
            renderer,
            Rectangle::new(
                bounds.position(),
                Size::new(bounds.width, self.row_height() - m.divider),
            ),
            Radius::default(),
            base.container,
        );
        surface::fill(
            renderer,
            Rectangle::new(
                Point::new(bounds.x, bounds.y + bounds.height - m.divider),
                Size::new(bounds.width, m.divider),
            ),
            Radius::default(),
            base.divider,
        );
        for (i, (rect, item)) in rects.iter().zip(&state.items).enumerate() {
            let selected = i == self.selected;
            let tab = &self.tabs[i];
            let look = theme.style(
                &self.class,
                self.kind,
                item.press.status(true, false),
                selected,
            );
            let now = item.press.now();
            surface::state_layer(
                renderer,
                *rect,
                Radius::default(),
                look.hover_layer,
                item.press.interaction.hover.value(now),
            );
            item.press.interaction.ripple.draw(
                renderer,
                *rect,
                Radius::default(),
                look.pressed_layer,
                theme,
                now,
            );
            let label_h = if tab.label.is_some() {
                self.label_style.line_height
            } else {
                0.0
            };
            let icon_h = if tab.icon.is_some() { m.icon_size } else { 0.0 };
            let gap = if tab.label.is_some() && tab.icon.is_some() {
                m.icon_label_space
            } else {
                0.0
            };
            let content_h = icon_h + gap + label_h;
            let mut y = rect.y + (rect.height - content_h) / 2.0;
            if let Some(icon) = &tab.icon {
                let handle = if selected {
                    tab.selected_icon.as_ref().unwrap_or(icon)
                } else {
                    icon
                };
                renderer.draw_svg(
                    tinted(handle.clone(), look.icon, 1.0),
                    Rectangle::new(
                        Point::new(rect.center_x() - m.icon_size / 2.0, y),
                        Size::new(m.icon_size, m.icon_size),
                    ),
                    clip,
                );
                y += m.icon_size + gap;
            }
            if tab.label.is_some() {
                let size = item.label.size();
                item.label.draw(
                    renderer,
                    Point::new(rect.center_x() - size.width / 2.0, y),
                    look.label,
                    clip,
                );
            }
            let ring = item.press.interaction.focus_ring_width(&self.tokens);
            let inward = theme.focus_ring.inward_offset;
            focus_ring::draw(
                renderer,
                rect.shrink(inward + ring),
                Radius::default(),
                0.0,
                ring,
                theme.colors.secondary,
            );
        }
        if !rects.is_empty() {
            let indicator = state.current();
            let radius = match self.kind {
                Kind::Primary => Radius {
                    top_left: m.primary_indicator,
                    top_right: m.primary_indicator,
                    bottom_right: 0.0,
                    bottom_left: 0.0,
                },
                Kind::Secondary => Radius::default(),
            };
            surface::fill(renderer, indicator, radius, base.indicator);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<TabsState>();
        if self
            .rects(layout.bounds(), state)
            .into_iter()
            .any(|r| cursor.is_over(r))
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<Tabs<'a, Message>> for Element<'a, Message> {
    fn from(tabs: Tabs<'a, Message>) -> Self {
        IcedElement::new(tabs)
    }
}
