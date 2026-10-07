// SPDX-License-Identifier: LGPL-3.0-only

//! Navigation bars and navigation rails.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay, renderer};
use iced::border::Radius;
use iced::{
    Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, Vector, mouse,
};

use crate::Element;
use crate::draw::text::Label;
use crate::draw::{focus_ring, shadow, surface};
use crate::icon::tinted;
use crate::interaction::Tokens;
use crate::motion::Transition;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::Status;
use crate::widget::selection::{self, State, centered};

/// Navigation bar and rail dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of a navigation bar.
    pub bar_height: f32,
    /// Size of the active indicator of a navigation bar.
    pub bar_indicator: Size,
    /// Space above the indicator of a navigation bar item with a label.
    pub bar_indicator_top: f32,
    /// Width of a navigation rail.
    pub rail_width: f32,
    /// Size of the active indicator of a navigation rail item with a label.
    pub rail_indicator: Size,
    /// Height of a navigation rail item.
    pub rail_item_height: f32,
    /// Space between navigation rail items.
    pub rail_spacing: f32,
    /// Space above the first navigation rail item when there is no header.
    pub rail_top: f32,
    /// Space above the header of a navigation rail and below its items.
    pub rail_padding: f32,
    /// Space between the header of a navigation rail and its first item.
    pub rail_header_gap: f32,
    /// Space between the indicator and the label.
    pub label_gap: f32,
    /// Space above the indicator of a navigation rail item with a label.
    pub rail_indicator_top: f32,
    /// Icon size.
    pub icon_size: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            bar_height: 80.0,
            bar_indicator: Size::new(64.0, 32.0),
            bar_indicator_top: 12.0,
            rail_width: 80.0,
            rail_indicator: Size::new(56.0, 32.0),
            rail_item_height: 56.0,
            rail_spacing: 4.0,
            rail_top: 44.0,
            rail_padding: 4.0,
            rail_header_gap: 8.0,
            label_gap: 4.0,
            rail_indicator_top: 4.0,
            icon_size: 24.0,
        }
    }
}

/// A badge on the icon of a destination.
#[derive(Debug, Clone, PartialEq)]
pub enum Badge {
    /// A small dot without a label.
    Dot,
    /// A large badge with a short label such as a count.
    Count(String),
}

/// One place a navigation component leads to.
pub struct Destination {
    label: String,
    icon: iced::widget::svg::Handle,
    selected_icon: Option<iced::widget::svg::Handle>,
    badge: Option<Badge>,
}

impl Destination {
    pub(crate) fn drawer_item(&self) -> crate::widget::navigation_drawer::DrawerItem {
        let mut item = crate::widget::navigation_drawer::DrawerItem::new(self.label.clone())
            .icon(self.icon.clone());
        if let Some(selected) = &self.selected_icon {
            item = item.selected_icon(selected.clone());
        }
        if let Some(Badge::Count(text)) = &self.badge {
            item = item.badge(text.clone());
        }
        item
    }

    /// A destination with an icon and a label.
    pub fn new(label: impl Into<String>, icon: iced::widget::svg::Handle) -> Self {
        Destination {
            label: label.into(),
            icon,
            selected_icon: None,
            badge: None,
        }
    }

    /// A destination using the outlined and the filled variant of a bundled symbol.
    pub fn symbol(label: impl Into<String>, symbol: fn(bool) -> iced::widget::svg::Handle) -> Self {
        Destination::new(label, symbol(false)).selected_icon(symbol(true))
    }

    /// Sets the icon shown while the destination is selected.
    pub fn selected_icon(mut self, icon: iced::widget::svg::Handle) -> Self {
        self.selected_icon = Some(icon);
        self
    }

    /// Adds a badge to the icon.
    pub fn badge(mut self, badge: Badge) -> Self {
        self.badge = Some(badge);
        self
    }
}

/// Which destinations show their label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Labels {
    /// Every destination.
    Always,
    /// Only the selected destination.
    Selected,
    /// No destination.
    Never,
}

/// Navigation bar or rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A bar along the bottom edge of a compact window.
    Bar,
    /// A rail along the side of a larger window.
    Rail,
}

/// The appearance of a destination in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Container color of the bar or rail.
    pub container: Color,
    /// Elevation level of the container.
    pub elevation: f32,
    /// Active indicator color.
    pub indicator: Color,
    /// Icon color.
    pub icon: Color,
    /// Label color.
    pub label: Color,
    /// Hover and press state layer color.
    pub state_layer: Color,
    /// Badge fill.
    pub badge: Color,
    /// Badge label color.
    pub badge_label: Color,
}

/// The appearance catalog of navigation bars and rails.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class for a destination.
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

/// The baseline style of a destination.
pub fn style(theme: &Theme, kind: Kind, _status: Status, selected: bool) -> Style {
    let c = &theme.colors;
    Style {
        container: match kind {
            Kind::Bar => c.surface_container,
            Kind::Rail => c.surface,
        },
        elevation: match kind {
            Kind::Bar => 2.0,
            Kind::Rail => 0.0,
        },
        indicator: c.secondary_container,
        icon: if selected {
            c.on_secondary_container
        } else {
            c.on_surface_variant
        },
        label: if selected {
            c.on_surface
        } else {
            c.on_surface_variant
        },
        state_layer: c.on_surface,
        badge: c.error,
        badge_label: c.on_error,
    }
}

/// A navigation bar or rail.
pub struct Navigation<'a, Message> {
    items: Vec<Destination>,
    selected: usize,
    on_select: Box<dyn Fn(usize) -> Message + 'a>,
    kind: Kind,
    labels: Labels,
    header: Option<Element<'a, Message>>,
    metrics: Metrics,
    tokens: Tokens,
    transition: Transition,
    label_style: TypeStyle,
    badge_style: TypeStyle,
    class: StyleFn<'a>,
}

fn new<'a, Message>(
    theme: &Theme,
    kind: Kind,
    items: Vec<Destination>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Navigation<'a, Message> {
    Navigation {
        items,
        selected,
        on_select: Box::new(on_select),
        kind,
        labels: Labels::Always,
        header: None,
        metrics: theme.components.navigation,
        tokens: Tokens::new(theme),
        transition: Transition {
            duration: theme.motion.duration.short3,
            easing: theme.motion.easing.standard,
        },
        label_style: theme.typography.label_medium,
        badge_style: theme.typography.label_small,
        class: Box::new(style),
    }
}

/// A navigation bar for compact windows.
pub fn navigation_bar<'a, Message>(
    theme: &Theme,
    items: Vec<Destination>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Navigation<'a, Message> {
    new(theme, Kind::Bar, items, selected, on_select)
}

/// A navigation rail for medium and larger windows.
pub fn navigation_rail<'a, Message>(
    theme: &Theme,
    items: Vec<Destination>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Navigation<'a, Message> {
    new(theme, Kind::Rail, items, selected, on_select)
}

impl<'a, Message> Navigation<'a, Message> {
    /// Chooses which destinations show their label.
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Adds a widget above the destinations of a rail, such as a menu button or a FAB.
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Kind, Status, bool) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn labeled(&self, index: usize) -> bool {
        match self.labels {
            Labels::Always => true,
            Labels::Selected => index == self.selected,
            Labels::Never => false,
        }
    }
}

struct ItemState {
    selection: State,
    label: Label,
    badge: Label,
}

#[derive(Default)]
struct NavigationState {
    items: Vec<ItemState>,
    header_height: f32,
}

impl<Message> Navigation<'_, Message> {
    fn item_rects(&self, bounds: Rectangle, header_height: f32) -> Vec<Rectangle> {
        let m = &self.metrics;
        let n = self.items.len();
        match self.kind {
            Kind::Bar => {
                let width = bounds.width / n.max(1) as f32;
                (0..n)
                    .map(|i| {
                        Rectangle::new(
                            Point::new(bounds.x + width * i as f32, bounds.y),
                            Size::new(width, m.bar_height),
                        )
                    })
                    .collect()
            }
            Kind::Rail => {
                let mut y = bounds.y
                    + if self.header.is_some() {
                        m.rail_padding + header_height + m.rail_header_gap
                    } else {
                        m.rail_top
                    };
                (0..n)
                    .map(|_| {
                        let r = Rectangle::new(
                            Point::new(bounds.x, y),
                            Size::new(bounds.width, m.rail_item_height),
                        );
                        y += m.rail_item_height + m.rail_spacing;
                        r
                    })
                    .collect()
            }
        }
    }

    fn indicator(&self, item: Rectangle, labeled: bool) -> Rectangle {
        let m = &self.metrics;
        let (size, top) = match (self.kind, labeled) {
            (Kind::Bar, true) => (m.bar_indicator, m.bar_indicator_top),
            (Kind::Bar, false) => (
                m.bar_indicator,
                (item.height - m.bar_indicator.height) / 2.0,
            ),
            (Kind::Rail, true) => (m.rail_indicator, m.rail_indicator_top),
            (Kind::Rail, false) => (Size::new(m.rail_indicator.width, m.rail_item_height), 0.0),
        };
        Rectangle::new(
            Point::new(item.center_x() - size.width / 2.0, item.y + top),
            size,
        )
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Navigation<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<NavigationState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(NavigationState {
            items: self
                .items
                .iter()
                .enumerate()
                .map(|(i, _)| ItemState {
                    selection: State::new(i == self.selected),
                    label: Label::default(),
                    badge: Label::default(),
                })
                .collect(),
            header_height: 0.0,
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.header.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements: Vec<&Element<'_, Message>> = self.header.iter().collect();
        tree.diff_children_custom(
            &elements,
            |tree, element| tree.diff(element.as_widget()),
            |element| Tree::new(*element),
        );
    }

    fn size(&self) -> Size<Length> {
        match self.kind {
            Kind::Bar => Size::new(Length::Fill, Length::Fixed(self.metrics.bar_height)),
            Kind::Rail => Size::new(Length::Fixed(self.metrics.rail_width), Length::Fill),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<NavigationState>();
        for (item, s) in self.items.iter().zip(&mut state.items) {
            s.label.update(&item.label, self.label_style);
            let text = match &item.badge {
                Some(Badge::Count(text)) => text.as_str(),
                _ => "",
            };
            s.badge.update(text, self.badge_style);
        }
        let m = &self.metrics;
        let mut children = Vec::new();
        state.header_height = 0.0;
        if let Some(header) = &mut self.header {
            let node = header.as_widget_mut().layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(m.rail_width, limits.max().height)),
            );
            let size = node.size();
            state.header_height = size.height;
            children.push(node.move_to(Point::new(
                (m.rail_width - size.width) / 2.0,
                m.rail_padding,
            )));
        }
        let size = match self.kind {
            Kind::Bar => Size::new(limits.max().width, m.bar_height),
            Kind::Rail => Size::new(m.rail_width, limits.max().height),
        };
        layout::Node::with_children(size, children)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let (state_tree, children) = (&mut tree.state, &mut tree.children);
        let state = state_tree.downcast_mut::<NavigationState>();
        if let (Some(header), Some(child)) = (&mut self.header, layout.children().next()) {
            header
                .as_widget_mut()
                .operate(&mut children[0], child, renderer, operation);
        }
        let rects = self.item_rects(layout.bounds(), state.header_height);
        for (rect, item) in rects.into_iter().zip(&mut state.items) {
            operation.focusable(None, rect, &mut item.selection.press.interaction.focus);
        }
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
        let (state_tree, children) = (&mut tree.state, &mut tree.children);
        let state = state_tree.downcast_mut::<NavigationState>();
        if let (Some(header), Some(child)) = (&mut self.header, layout.children().next()) {
            header.as_widget_mut().update(
                &mut children[0],
                event,
                child,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            if shell.is_event_captured() {
                return;
            }
        }
        let rects = self.item_rects(layout.bounds(), state.header_height);
        for (i, (rect, item)) in rects.into_iter().zip(&mut state.items).enumerate() {
            let activation = item.selection.update(
                event,
                i == self.selected,
                self.transition,
                rect,
                cursor,
                true,
                &self.tokens,
                shell,
            );
            if activation.is_some() {
                shell.publish((self.on_select)(i));
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style_in: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<NavigationState>();
        let bounds = layout.bounds();
        let m = &self.metrics;
        let base = theme.style(&self.class, self.kind, Status::Active, false);
        shadow::draw(
            renderer,
            bounds,
            Radius::default(),
            base.elevation,
            theme.colors.shadow,
            &theme.elevation,
        );
        surface::fill(renderer, bounds, Radius::default(), base.container);
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let rects = self.item_rects(bounds, state.header_height);
        for (i, (rect, item)) in rects.into_iter().zip(&state.items).enumerate() {
            let selected = i == self.selected;
            let labeled = self.labeled(i);
            let look = theme.style(
                &self.class,
                self.kind,
                item.selection.press.status(true, false),
                selected,
            );
            let pill = self.indicator(rect, labeled);
            let radius = Radius::from(pill.height / 2.0);
            let t = item.selection.progress().clamp(0.0, 1.0);
            if t > 0.0 {
                let width = pill.width * t;
                surface::fill(
                    renderer,
                    Rectangle::new(
                        Point::new(pill.center_x() - width / 2.0, pill.y),
                        Size::new(width, pill.height),
                    ),
                    radius,
                    look.indicator,
                );
            }
            selection::draw_state_layer_rect(
                renderer,
                theme,
                &item.selection,
                pill,
                radius,
                look.state_layer,
                look.state_layer,
            );
            let destination = &self.items[i];
            let handle = if selected {
                destination
                    .selected_icon
                    .as_ref()
                    .unwrap_or(&destination.icon)
            } else {
                &destination.icon
            };
            let icon = centered(pill, Size::new(m.icon_size, m.icon_size));
            renderer.draw_svg(tinted(handle.clone(), look.icon, 1.0), icon, clip);
            if labeled {
                let size = item.label.size();
                item.label.draw(
                    renderer,
                    Point::new(
                        rect.center_x() - size.width / 2.0,
                        pill.y + pill.height + m.label_gap,
                    ),
                    look.label,
                    clip,
                );
            }
            match &destination.badge {
                Some(Badge::Dot) => {
                    let s = theme.components.badge.small_size;
                    let offset = theme.components.badge.small_offset;
                    surface::fill(
                        renderer,
                        Rectangle::new(
                            Point::new(icon.center_x() + offset.x, icon.y + offset.y),
                            Size::new(s, s),
                        ),
                        Radius::from(s / 2.0),
                        look.badge,
                    );
                }
                Some(Badge::Count(_)) => {
                    let b = &theme.components.badge;
                    let text = item.badge.size();
                    let width = (text.width + b.large_padding * 2.0).max(b.large_size);
                    let badge = Rectangle::new(
                        Point::new(
                            icon.center_x() + b.large_offset.x,
                            icon.y + b.large_offset.y,
                        ),
                        Size::new(width, b.large_size),
                    );
                    surface::fill(
                        renderer,
                        badge,
                        Radius::from(b.large_size / 2.0),
                        look.badge,
                    );
                    item.badge.draw(
                        renderer,
                        Point::new(
                            badge.center_x() - text.width / 2.0,
                            badge.center_y() - text.height / 2.0,
                        ),
                        look.badge_label,
                        clip,
                    );
                }
                None => {}
            }
            focus_ring::draw(
                renderer,
                pill,
                radius,
                theme.focus_ring.outward_offset,
                item.selection
                    .press
                    .interaction
                    .focus_ring_width(&self.tokens),
                theme.colors.secondary,
            );
        }
        if let (Some(header), Some(child)) = (&self.header, layout.children().next()) {
            header.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style_in,
                child,
                cursor,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<NavigationState>();
        if let (Some(header), Some(child)) = (&self.header, layout.children().next()) {
            let inner = header.as_widget().mouse_interaction(
                &tree.children[0],
                child,
                cursor,
                viewport,
                renderer,
            );
            if inner != mouse::Interaction::default() {
                return inner;
            }
        }
        if self
            .item_rects(layout.bounds(), state.header_height)
            .into_iter()
            .any(|r| cursor.is_over(r))
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let header = self.header.as_mut()?;
        header.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<Navigation<'a, Message>> for Element<'a, Message> {
    fn from(navigation: Navigation<'a, Message>) -> Self {
        IcedElement::new(navigation)
    }
}
