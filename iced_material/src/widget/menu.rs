// SPDX-License-Identifier: LGPL-3.0-only

//! Dropdown menus with icons, shortcuts, dividers and nested submenus.

use std::time::Duration;

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::time::Instant;
use iced::{
    Element as IcedElement, Event, Length, Padding, Point, Rectangle, Renderer, Size, Vector,
    mouse, window,
};

use crate::Element;
use crate::draw::surface;
use crate::draw::text::Label;
use crate::icon::{symbol, tinted};
use crate::interaction::Tokens;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::layer::{self, LayerState, Layered, Mode, Placement};
use crate::widget::list;
use crate::widget::panel::{Panel, Surface};
use crate::widget::pressable::{self, Status};

/// Menu dimensions and timing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of an item.
    pub item_height: f32,
    /// Start and end padding of an item.
    pub item_padding: f32,
    /// Space between the icon, the label, the shortcut and the arrow.
    pub gap: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Top and bottom padding of the menu.
    pub vertical_padding: f32,
    /// Space above and below a divider.
    pub divider_space: f32,
    /// Minimum width.
    pub min_width: f32,
    /// Maximum width.
    pub max_width: f32,
    /// Time the pointer rests on an item before its submenu opens.
    pub submenu_delay: Duration,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            item_height: 48.0,
            item_padding: 12.0,
            gap: 12.0,
            icon_size: 24.0,
            vertical_padding: 8.0,
            divider_space: 8.0,
            min_width: 112.0,
            max_width: 280.0,
            submenu_delay: Duration::from_millis(400),
        }
    }
}

/// The appearance catalog of menu items.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> list::Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, bool) -> list::Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> list::Style {
        class(self, status, selected)
    }
}

/// The baseline menu item style: a list item on the container of the menu.
pub fn style(theme: &Theme, status: Status, selected: bool) -> list::Style {
    let c = &theme.colors;
    let mut style = list::style(theme, status, selected);
    if !selected {
        style.base.container = iced::Color::TRANSPARENT;
    }
    if selected && status != Status::Disabled {
        style.leading_icon = c.on_secondary_container;
        style.trailing_icon = c.on_secondary_container;
    }
    style
}

struct Entry<Message> {
    label: String,
    leading: Option<iced::widget::svg::Handle>,
    shortcut: Option<String>,
    selected: bool,
    enabled: bool,
    on_select: Option<Message>,
    submenu: Vec<Item<Message>>,
}

/// One row of a menu.
pub struct Item<Message>(Kind<Message>);

enum Kind<Message> {
    Entry(Entry<Message>),
    Divider,
}

/// A menu item with a label.
pub fn item<Message>(label: impl Into<String>) -> Item<Message> {
    Item(Kind::Entry(Entry {
        label: label.into(),
        leading: None,
        shortcut: None,
        selected: false,
        enabled: true,
        on_select: None,
        submenu: Vec::new(),
    }))
}

/// A divider between groups of items.
pub fn divider<Message>() -> Item<Message> {
    Item(Kind::Divider)
}

impl<Message> Item<Message> {
    fn entry(mut self, f: impl FnOnce(&mut Entry<Message>)) -> Self {
        if let Kind::Entry(entry) = &mut self.0 {
            f(entry);
        }
        self
    }

    /// Sets the message produced when the item is chosen.
    pub fn on_select(self, message: Message) -> Self {
        self.entry(|e| e.on_select = Some(message))
    }

    /// Adds an icon before the label.
    pub fn leading_icon(self, icon: iced::widget::svg::Handle) -> Self {
        self.entry(|e| e.leading = Some(icon))
    }

    /// Adds text at the end of the item, such as a keyboard shortcut.
    pub fn shortcut(self, text: impl Into<String>) -> Self {
        self.entry(|e| e.shortcut = Some(text.into()))
    }

    /// Marks the item as the selected one.
    pub fn selected(self, selected: bool) -> Self {
        self.entry(|e| e.selected = selected)
    }

    /// Disables the item.
    pub fn disabled(self, disabled: bool) -> Self {
        self.entry(|e| e.enabled = !disabled)
    }

    /// Opens a nested menu instead of producing a message.
    pub fn submenu(self, items: Vec<Item<Message>>) -> Self {
        self.entry(|e| e.submenu = items)
    }
}

struct Row<Message> {
    entry: Option<Entry<Message>>,
    has_submenu: bool,
}

struct MenuList<'a, Message> {
    rows: Vec<Row<Message>>,
    subpanels: Vec<Option<Element<'a, Message>>>,
    on_dismiss: Option<Message>,
    metrics: Metrics,
    tokens: Tokens,
    layer_style: layer::Style,
    class: StyleFn<'a>,
    label_style: TypeStyle,
    shortcut_style: TypeStyle,
}

#[derive(Default)]
struct ItemState {
    press: pressable::State,
    label: Label,
    shortcut: Label,
}

#[derive(Default)]
struct MenuState {
    items: Vec<ItemState>,
    open_sub: Option<usize>,
    sub: LayerState,
    candidate: Option<(usize, Instant)>,
    now: Option<Instant>,
}

fn build_list<'a, Message: Clone + 'a>(
    theme: &Theme,
    items: Vec<Item<Message>>,
    on_dismiss: Option<Message>,
) -> MenuList<'a, Message> {
    let mut rows = Vec::new();
    let mut subpanels = Vec::new();
    for item in items {
        match item.0 {
            Kind::Divider => {
                rows.push(Row {
                    entry: None,
                    has_submenu: false,
                });
                subpanels.push(None);
            }
            Kind::Entry(mut entry) => {
                let sub = std::mem::take(&mut entry.submenu);
                let has_submenu = !sub.is_empty();
                subpanels.push(has_submenu.then(|| panel(theme, sub, on_dismiss.clone())));
                rows.push(Row {
                    entry: Some(entry),
                    has_submenu,
                });
            }
        }
    }
    MenuList {
        rows,
        subpanels,
        on_dismiss,
        metrics: theme.components.menu,
        tokens: Tokens::new(theme),
        layer_style: layer::Style::new(theme),
        class: Box::new(style),
        label_style: theme.typography.body_large,
        shortcut_style: theme.typography.label_small,
    }
}

fn panel<'a, Message: Clone + 'a>(
    theme: &Theme,
    items: Vec<Item<Message>>,
    on_dismiss: Option<Message>,
) -> Element<'a, Message> {
    let m = theme.components.menu;
    let c = &theme.colors;
    Panel::new(
        IcedElement::new(build_list(theme, items, on_dismiss)),
        Surface {
            color: c.surface_container,
            text: c.on_surface,
            shape: theme.shape.extra_small,
            elevation: 2.0,
            padding: Padding::from([m.vertical_padding, 0.0]),
            min_width: m.min_width,
            max_width: m.max_width,
        },
    )
    .into()
}

impl<Message> MenuList<'_, Message> {
    fn height_of(&self, row: &Row<Message>) -> f32 {
        match row.entry {
            Some(_) => self.metrics.item_height,
            None => self.metrics.divider_space * 2.0 + 1.0,
        }
    }

    fn rects(&self, bounds: Rectangle) -> Vec<Rectangle> {
        let mut y = bounds.y;
        self.rows
            .iter()
            .map(|row| {
                let height = self.height_of(row);
                let r = Rectangle::new(Point::new(bounds.x, y), Size::new(bounds.width, height));
                y += height;
                r
            })
            .collect()
    }

    fn enabled(&self, index: usize) -> bool {
        self.rows[index].entry.as_ref().is_some_and(|e| e.enabled)
    }

    fn focused(&self, state: &MenuState) -> Option<usize> {
        (0..self.rows.len()).find(|i| {
            state
                .items
                .get(*i)
                .is_some_and(|s| s.press.interaction.focus.focused)
        })
    }

    fn focus(&self, state: &mut MenuState, index: Option<usize>) {
        for (i, item) in state.items.iter_mut().enumerate() {
            let focus = &mut item.press.interaction.focus;
            if Some(i) == index {
                focus.focused = true;
                focus.visible = true;
                focus.since = None;
            } else {
                focus.focused = false;
                focus.visible = false;
            }
        }
    }

    fn step(&self, from: Option<usize>, forward: bool) -> Option<usize> {
        let n = self.rows.len();
        let mut i = match (from, forward) {
            (Some(i), true) => (i + 1) % n,
            (Some(i), false) => (i + n - 1) % n,
            (None, true) => 0,
            (None, false) => n - 1,
        };
        for _ in 0..n {
            if self.enabled(i) {
                return Some(i);
            }
            i = if forward {
                (i + 1) % n
            } else {
                (i + n - 1) % n
            };
        }
        None
    }

    fn child_index(&self, row: usize) -> usize {
        self.subpanels[..row].iter().filter(|p| p.is_some()).count()
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for MenuList<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.subpanels
            .iter()
            .flatten()
            .map(|e| Tree::new(e))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements: Vec<&Element<'_, Message>> = self.subpanels.iter().flatten().collect();
        tree.diff_children_custom(
            &elements,
            |tree, element| tree.diff(element.as_widget()),
            |element| Tree::new(*element),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<MenuState>();
        while state.items.len() < self.rows.len() {
            state.items.push(ItemState::default());
        }
        state.items.truncate(self.rows.len());
        let m = &self.metrics;
        let mut width: f32 = 0.0;
        for (row, item) in self.rows.iter().zip(&mut state.items) {
            let Some(entry) = &row.entry else { continue };
            let label = item.label.update(&entry.label, self.label_style).width;
            let shortcut = item
                .shortcut
                .update(entry.shortcut.as_deref().unwrap_or(""), self.shortcut_style)
                .width;
            let mut w = m.item_padding * 2.0 + label;
            if entry.leading.is_some() {
                w += m.icon_size + m.gap;
            }
            if entry.shortcut.is_some() {
                w += m.gap + shortcut;
            }
            if row.has_submenu {
                w += m.gap + m.icon_size;
            }
            width = width.max(w);
        }
        let height: f32 = self.rows.iter().map(|r| self.height_of(r)).sum();
        layout::Node::new(limits.resolve(Length::Shrink, Length::Shrink, Size::new(width, height)))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<MenuState>();
        for (i, rect) in self.rects(layout.bounds()).into_iter().enumerate() {
            if self.enabled(i)
                && let Some(item) = state.items.get_mut(i)
            {
                operation.focusable(None, rect, &mut item.press.interaction.focus);
            }
        }
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
        let state = tree.state.downcast_mut::<MenuState>();
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
        }
        let rects = self.rects(layout.bounds());

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            ..
        }) = event
        {
            let focused = self.focused(state);
            let target = match key {
                Named::ArrowDown => Some(self.step(focused, true)),
                Named::ArrowUp => Some(self.step(focused, false)),
                Named::Home => Some(self.step(None, true)),
                Named::End => Some(self.step(None, false)),
                _ => None,
            };
            if let Some(target) = target
                && focused.is_some()
                && !self.rows.is_empty()
            {
                self.focus(state, target);
                state.open_sub = None;
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            if matches!(key, Named::ArrowRight)
                && let Some(i) = focused
                && self.rows[i].has_submenu
                && self.enabled(i)
            {
                self.focus(state, None);
                state.open_sub = Some(i);
                state.sub = LayerState::default();
                state.sub.request_focus();
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            if matches!(key, Named::ArrowLeft)
                && let Some(i) = state.open_sub.take()
            {
                self.focus(state, Some(i));
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        let hovered = cursor
            .position()
            .and_then(|p| rects.iter().position(|r| r.contains(p)));
        let now = state.now.unwrap_or_else(Instant::now);
        match (hovered, state.candidate) {
            (Some(i), Some((c, _))) if i == c => {}
            (Some(i), _) => state.candidate = Some((i, now)),
            (None, _) => {}
        }
        if let Some((i, since)) = state.candidate
            && hovered == Some(i)
            && self.enabled(i)
        {
            if now.saturating_duration_since(since) >= self.metrics.submenu_delay {
                let want = self.rows[i].has_submenu.then_some(i);
                if state.open_sub != want {
                    state.open_sub = want;
                    state.sub = LayerState::default();
                    shell.request_redraw();
                }
            } else {
                shell.request_redraw();
            }
        }

        let mut chosen = None;
        for (i, rect) in rects.iter().enumerate() {
            let enabled = self.enabled(i);
            let Some(item) = state.items.get_mut(i) else {
                continue;
            };
            if self.rows[i].entry.is_none() {
                continue;
            }
            if item
                .press
                .update(event, *rect, cursor, enabled, false, &self.tokens, shell)
                .is_some()
            {
                chosen = Some(i);
            }
        }
        if let Some(i) = chosen {
            if self.rows[i].has_submenu {
                state.open_sub = Some(i);
                state.sub = LayerState::default();
                state.sub.request_focus();
            } else if let Some(entry) = &self.rows[i].entry {
                if let Some(message) = &entry.on_select {
                    shell.publish(message.clone());
                }
                if let Some(message) = &self.on_dismiss {
                    shell.publish(message.clone());
                }
            }
            shell.request_redraw();
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
        let state = tree.state.downcast_ref::<MenuState>();
        let m = &self.metrics;
        let clip = layout.bounds().intersection(viewport).unwrap_or(*viewport);
        for ((row, rect), item) in self
            .rows
            .iter()
            .zip(self.rects(layout.bounds()))
            .zip(&state.items)
        {
            let Some(entry) = &row.entry else {
                surface::fill(
                    renderer,
                    Rectangle::new(
                        Point::new(rect.x, rect.y + m.divider_space),
                        Size::new(rect.width, 1.0),
                    ),
                    Radius::default(),
                    theme.colors.outline_variant,
                );
                continue;
            };
            let selected = entry.selected;
            let status = item.press.status(entry.enabled, false);
            let look = theme.style(&self.class, status, selected);
            pressable::draw_container(
                renderer,
                theme,
                rect,
                Radius::default(),
                &look.base,
                0.0,
                &item.press,
            );
            let mut x = rect.x + m.item_padding;
            let icon = |renderer: &mut Renderer,
                        handle: &iced::widget::svg::Handle,
                        x: f32,
                        color: iced::Color| {
                renderer.draw_svg(
                    tinted(handle.clone(), color, 1.0),
                    Rectangle::new(
                        Point::new(x, rect.center_y() - m.icon_size / 2.0),
                        Size::new(m.icon_size, m.icon_size),
                    ),
                    clip,
                );
            };
            if let Some(handle) = &entry.leading {
                icon(renderer, handle, x, look.leading_icon);
                x += m.icon_size + m.gap;
            }
            let label = item.label.size();
            item.label.draw(
                renderer,
                Point::new(x, rect.center_y() - label.height / 2.0),
                look.label,
                clip,
            );
            let mut right = rect.x + rect.width - m.item_padding;
            if row.has_submenu {
                right -= m.icon_size;
                icon(
                    renderer,
                    &symbol::chevron_right(false),
                    right,
                    look.trailing_icon,
                );
                right -= m.gap;
            }
            if entry.shortcut.is_some() {
                let size = item.shortcut.size();
                item.shortcut.draw(
                    renderer,
                    Point::new(right - size.width, rect.center_y() - size.height / 2.0),
                    look.trailing_text,
                    clip,
                );
            }
            let ring = item.press.interaction.focus_ring_width(&self.tokens);
            let inward = theme.focus_ring.inward_offset;
            crate::draw::focus_ring::draw(
                renderer,
                rect.shrink(inward + ring),
                Radius::default(),
                0.0,
                ring,
                theme.colors.secondary,
            );
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let over = self
            .rects(layout.bounds())
            .into_iter()
            .enumerate()
            .any(|(i, r)| self.enabled(i) && cursor.is_over(r));
        if over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<MenuState>();
        let open = state.open_sub?;
        let rect = *self.rects(layout.bounds()).get(open)?;
        let index = self.child_index(open);
        let element = self.subpanels[open].as_mut()?;
        let anchor = Rectangle::new(
            Point::new(
                rect.x + translation.x,
                rect.y + translation.y - self.metrics.vertical_padding,
            ),
            rect.size(),
        );
        Some(layer::passive_popup(
            element,
            children.get_mut(index)?,
            &mut state.sub,
            &self.layer_style,
            anchor,
            Placement::Side(0.0),
        ))
    }
}

/// A menu that opens below its anchor while `open` is true.
pub struct Menu<'a, Message> {
    anchor: Element<'a, Message>,
    items: Vec<Item<Message>>,
    open: bool,
    on_dismiss: Option<Message>,
    match_width: bool,
}

/// A menu over `anchor`.
pub fn menu<'a, Message>(
    anchor: impl Into<Element<'a, Message>>,
    items: Vec<Item<Message>>,
    open: bool,
) -> Menu<'a, Message> {
    Menu {
        anchor: anchor.into(),
        items,
        open,
        on_dismiss: None,
        match_width: false,
    }
}

impl<'a, Message: Clone + 'a> Menu<'a, Message> {
    /// Sets the message produced when the menu should close: an outside click,
    /// Escape, or a chosen item.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Makes the menu at least as wide as its anchor.
    pub fn match_anchor_width(mut self, value: bool) -> Self {
        self.match_width = value;
        self
    }

    /// Builds the anchor with its menu.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        popup(
            theme,
            self.anchor,
            self.items,
            self.open,
            self.on_dismiss,
            self.match_width,
        )
    }
}

pub(crate) fn popup<'a, Message: Clone + 'a>(
    theme: &Theme,
    anchor: Element<'a, Message>,
    items: Vec<Item<Message>>,
    open: bool,
    on_dismiss: Option<Message>,
    match_width: bool,
) -> Element<'a, Message> {
    Layered::new(
        theme,
        anchor,
        panel(theme, items, on_dismiss.clone()),
        Mode::Popup { open, on_dismiss },
        Placement::BelowStart(0.0),
    )
    .match_anchor_width(match_width)
    .into()
}
