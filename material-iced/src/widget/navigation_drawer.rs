// SPDX-License-Identifier: LGPL-3.0-only

//! Standard and modal navigation drawers.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::widget::{column, container};
use iced::{
    Color, Element as IcedElement, Event, Length, Padding, Point, Rectangle, Renderer, Size, mouse,
};

use crate::Element;
use crate::draw::focus_ring;
use crate::draw::surface;
use crate::draw::text::Label;
use crate::icon::tinted;
use crate::interaction::Tokens;
use crate::motion::Transition;
use crate::shape::Shape;
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::divider;
use crate::widget::layer::{Edge, edge_modal};
use crate::widget::panel::{Panel, Surface, text};
use crate::widget::pressable::Status;
use crate::widget::selection::{self, State, centered};

/// Navigation drawer dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Width of the drawer.
    pub width: f32,
    /// Height of an item and of its active indicator.
    pub item_height: f32,
    /// Space on both sides of an item.
    pub item_margin: f32,
    /// Space before the icon and, without one, before the label.
    pub item_start: f32,
    /// Space after the label or badge text.
    pub item_end: f32,
    /// Space between the icon and the label.
    pub icon_label_space: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Space above the first and below the last item.
    pub vertical_padding: f32,
    /// Height of a section headline.
    pub headline_height: f32,
    /// Space before a section headline.
    pub headline_start: f32,
    /// Space above and below a divider between sections.
    pub divider_space: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            width: 360.0,
            item_height: 56.0,
            item_margin: 12.0,
            item_start: 16.0,
            item_end: 24.0,
            icon_label_space: 12.0,
            icon_size: 24.0,
            vertical_padding: 12.0,
            headline_height: 56.0,
            headline_start: 16.0,
            divider_space: 8.0,
        }
    }
}

/// The appearance of a drawer item in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Active indicator color.
    pub indicator: Color,
    /// Icon color.
    pub icon: Color,
    /// Label color.
    pub label: Color,
    /// Badge text color.
    pub badge: Color,
    /// Hover state layer color.
    pub hover_layer: Color,
    /// Press ripple color.
    pub pressed_layer: Color,
}

/// The appearance catalog of drawer items.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, bool) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style {
        class(self, status, selected)
    }
}

/// The baseline drawer item style.
pub fn style(theme: &Theme, _status: Status, selected: bool) -> Style {
    let c = &theme.colors;
    Style {
        indicator: c.secondary_container,
        icon: if selected {
            c.on_secondary_container
        } else {
            c.on_surface_variant
        },
        label: if selected {
            c.on_secondary_container
        } else {
            c.on_surface_variant
        },
        badge: c.on_surface_variant,
        hover_layer: if selected {
            c.on_secondary_container
        } else {
            c.on_surface
        },
        pressed_layer: c.on_secondary_container,
    }
}

/// One entry of a drawer.
pub struct DrawerItem {
    label: String,
    icon: Option<iced::widget::svg::Handle>,
    selected_icon: Option<iced::widget::svg::Handle>,
    badge: Option<String>,
}

impl DrawerItem {
    /// An item with a label.
    pub fn new(label: impl Into<String>) -> Self {
        DrawerItem {
            label: label.into(),
            icon: None,
            selected_icon: None,
            badge: None,
        }
    }

    /// Adds an icon before the label.
    pub fn icon(mut self, icon: iced::widget::svg::Handle) -> Self {
        self.icon = Some(icon);
        self
    }

    pub(crate) fn selected_icon(mut self, icon: iced::widget::svg::Handle) -> Self {
        self.selected_icon = Some(icon);
        self
    }

    /// Adds the outlined and the filled variant of a bundled symbol.
    pub fn symbol(mut self, symbol: fn(bool) -> iced::widget::svg::Handle) -> Self {
        self.icon = Some(symbol(false));
        self.selected_icon = Some(symbol(true));
        self
    }

    /// Adds text at the end of the item, such as a count.
    pub fn badge(mut self, text: impl Into<String>) -> Self {
        self.badge = Some(text.into());
        self
    }
}

/// A group of items with an optional headline.
pub struct Section {
    headline: Option<String>,
    items: Vec<DrawerItem>,
}

/// A section of items.
pub fn section(items: Vec<DrawerItem>) -> Section {
    Section {
        headline: None,
        items,
    }
}

impl Section {
    /// Adds a headline above the items.
    pub fn headline(mut self, headline: impl Into<String>) -> Self {
        self.headline = Some(headline.into());
        self
    }
}

struct Row<'a, Message> {
    item: DrawerItem,
    selected: bool,
    on_press: Message,
    metrics: Metrics,
    tokens: Tokens,
    transition: Transition,
    label_style: TypeStyle,
    class: StyleFn<'a>,
}

#[derive(Default)]
struct RowLabels {
    label: Label,
    badge: Label,
}

struct RowState {
    selection: State,
    labels: RowLabels,
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Row<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<RowState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(RowState {
            selection: State::new(self.selected),
            labels: RowLabels::default(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.metrics.item_height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<RowState>();
        state
            .labels
            .label
            .update(&self.item.label, self.label_style);
        state
            .labels
            .badge
            .update(self.item.badge.as_deref().unwrap_or(""), self.label_style);
        let h = self.metrics.item_height;
        layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(h), Size::new(0.0, h)))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<RowState>();
        operation.focusable(
            None,
            layout.bounds(),
            &mut state.selection.press.interaction.focus,
        );
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
        let state = tree.state.downcast_mut::<RowState>();
        if state
            .selection
            .update(
                event,
                self.selected,
                self.transition,
                layout.bounds(),
                cursor,
                true,
                &self.tokens,
                shell,
            )
            .is_some()
        {
            shell.publish(self.on_press.clone());
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
        let state = tree.state.downcast_ref::<RowState>();
        let bounds = layout.bounds();
        let m = &self.metrics;
        let look = theme.style(
            &self.class,
            state.selection.press.status(true, false),
            self.selected,
        );
        let radius = Radius::from(bounds.height / 2.0);
        let t = state.selection.progress().clamp(0.0, 1.0);
        surface::fill(renderer, bounds, radius, alpha(look.indicator, t));
        selection::draw_state_layer_rect(
            renderer,
            theme,
            &state.selection,
            bounds,
            radius,
            look.hover_layer,
            look.pressed_layer,
        );
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let mut x = bounds.x + m.item_start;
        if let Some(icon) = &self.item.icon {
            let handle = if self.selected {
                self.item.selected_icon.as_ref().unwrap_or(icon)
            } else {
                icon
            };
            let r = centered(
                Rectangle::new(
                    Point::new(x, bounds.y),
                    Size::new(m.icon_size, bounds.height),
                ),
                Size::new(m.icon_size, m.icon_size),
            );
            renderer.draw_svg(tinted(handle.clone(), look.icon, 1.0), r, clip);
            x += m.icon_size + m.icon_label_space;
        }
        let label = state.labels.label.size();
        state.labels.label.draw(
            renderer,
            Point::new(x, bounds.center_y() - label.height / 2.0),
            look.label,
            clip,
        );
        if self.item.badge.is_some() {
            let badge = state.labels.badge.size();
            state.labels.badge.draw(
                renderer,
                Point::new(
                    bounds.x + bounds.width - m.item_end - badge.width,
                    bounds.center_y() - badge.height / 2.0,
                ),
                look.badge,
                clip,
            );
        }
        focus_ring::draw(
            renderer,
            bounds,
            radius,
            theme.focus_ring.outward_offset,
            state
                .selection
                .press
                .interaction
                .focus_ring_width(&self.tokens),
            theme.colors.secondary,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

/// A navigation drawer.
pub struct Drawer<Message> {
    sections: Vec<Section>,
    selected: usize,
    on_select: Box<dyn Fn(usize) -> Message>,
    modal: bool,
    fill_height: bool,
}

/// A drawer with sections. `selected` counts items across all sections.
pub fn navigation_drawer<Message>(
    sections: Vec<Section>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + 'static,
) -> Drawer<Message> {
    Drawer {
        sections,
        selected,
        on_select: Box::new(on_select),
        modal: false,
        fill_height: true,
    }
}

impl<Message> Drawer<Message> {
    fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    /// Sizes the drawer to its content instead of the available height.
    pub fn shrink_height(mut self) -> Self {
        self.fill_height = false;
        self
    }
}

impl<Message: Clone + 'static> Drawer<Message> {
    /// Builds the drawer widget.
    pub fn build(self, theme: &Theme) -> Element<'static, Message> {
        let m = theme.components.navigation_drawer;
        let c = &theme.colors;
        let mut body = column![].width(Length::Fill);
        let mut index = 0;
        let count = self.sections.len();
        for (n, section) in self.sections.into_iter().enumerate() {
            if let Some(headline) = section.headline {
                body = body.push(
                    container(text(
                        headline,
                        theme.typography.title_small,
                        c.on_surface_variant,
                    ))
                    .center_y(Length::Fixed(m.headline_height))
                    .padding(Padding::ZERO.left(m.headline_start)),
                );
            }
            for item in section.items {
                let selected = index == self.selected;
                body = body.push(IcedElement::new(Row {
                    item,
                    selected,
                    on_press: (self.on_select)(index),
                    metrics: m,
                    tokens: Tokens::new(theme),
                    transition: Transition {
                        duration: theme.motion.duration.short3,
                        easing: theme.motion.easing.standard,
                    },
                    label_style: theme.typography.label_large,
                    class: Box::new(style),
                }));
                index += 1;
            }
            if n + 1 < count {
                body = body.push(
                    container(divider::horizontal(theme))
                        .padding(Padding::from([m.divider_space, 0.0])),
                );
            }
        }
        let shape = if self.modal {
            theme.shape.large_end
        } else {
            Shape::all(crate::shape::Corner::Fixed(0.0))
        };
        let mut panel = Panel::new(
            body.into(),
            Surface {
                color: if self.modal {
                    c.surface_container_low
                } else {
                    c.surface
                },
                text: c.on_surface,
                shape,
                elevation: if self.modal { 1.0 } else { 0.0 },
                padding: Padding::from([m.vertical_padding, m.item_margin]),
                min_width: m.width,
                max_width: m.width,
            },
        );
        if self.fill_height {
            panel = panel.fill_height();
        }
        panel.into()
    }
}

/// Shows `drawer` sliding in from the start edge over `base` while `open` is true.
pub struct ModalDrawer<Message> {
    drawer: Drawer<Message>,
    open: bool,
    on_dismiss: Option<Message>,
    dismiss_on_scrim: bool,
}

/// A modal navigation drawer layer.
pub fn modal_drawer<Message>(drawer: Drawer<Message>, open: bool) -> ModalDrawer<Message> {
    ModalDrawer {
        drawer: drawer.modal(),
        open,
        on_dismiss: None,
        dismiss_on_scrim: true,
    }
}

impl<Message: Clone + 'static> ModalDrawer<Message> {
    /// Sets the message produced when the user asks to close the drawer.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Chooses whether a press on the scrim dismisses. The default is true.
    pub fn dismiss_on_scrim(mut self, dismiss: bool) -> Self {
        self.dismiss_on_scrim = dismiss;
        self
    }

    /// Builds the layer over `base`.
    pub fn build<'a>(
        self,
        theme: &Theme,
        base: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        edge_modal(
            theme,
            base.into(),
            self.drawer.build(theme),
            self.open,
            Edge::Start,
            self.on_dismiss,
            self.dismiss_on_scrim,
        )
    }
}
