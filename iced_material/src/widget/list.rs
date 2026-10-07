// SPDX-License-Identifier: LGPL-3.0-only

//! List items with one, two or three lines and leading and trailing elements.

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
use crate::draw::{focus_ring, surface};
use crate::icon::tinted;
use crate::interaction::{Activation, Tokens};
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::{self, Status};

/// List item dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height with one line of text.
    pub one_line: f32,
    /// Height with two lines of text.
    pub two_line: f32,
    /// Height with three lines of text.
    pub three_line: f32,
    /// Start padding.
    pub leading_space: f32,
    /// End padding.
    pub trailing_space: f32,
    /// Top and bottom padding.
    pub vertical_space: f32,
    /// Space between the leading element, the text and the trailing element.
    pub gap: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Diameter of an avatar.
    pub avatar_size: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            one_line: 56.0,
            two_line: 72.0,
            three_line: 88.0,
            leading_space: 16.0,
            trailing_space: 16.0,
            vertical_space: 12.0,
            gap: 16.0,
            icon_size: 24.0,
            avatar_size: 40.0,
        }
    }
}

/// The appearance of a list item in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Container fill and state layers.
    pub base: pressable::Style,
    /// Color of the headline.
    pub label: Color,
    /// Color of the supporting text.
    pub supporting: Color,
    /// Color of the overline text.
    pub overline: Color,
    /// Color of a leading icon.
    pub leading_icon: Color,
    /// Color of a trailing icon.
    pub trailing_icon: Color,
    /// Color of trailing text.
    pub trailing_text: Color,
    /// Fill of an avatar.
    pub avatar: Color,
    /// Text color of an avatar.
    pub avatar_label: Color,
}

/// The appearance catalog of list items.
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

/// The baseline list item style.
pub fn style(theme: &Theme, status: Status, selected: bool) -> Style {
    let c = &theme.colors;
    let container = if selected {
        c.secondary_container
    } else {
        c.surface
    };
    if status == Status::Disabled {
        let icon = alpha(c.on_surface, theme.disabled.content);
        let text = alpha(c.on_surface, 0.3);
        return Style {
            base: pressable::Style {
                container,
                ..pressable::Style::content(theme, text, icon, c.on_surface)
            },
            label: text,
            supporting: text,
            overline: text,
            leading_icon: icon,
            trailing_icon: icon,
            trailing_text: text,
            avatar: alpha(c.on_surface, theme.disabled.container),
            avatar_label: icon,
        };
    }
    Style {
        base: pressable::Style {
            container,
            ..pressable::Style::content(theme, c.on_surface, c.on_surface_variant, c.on_surface)
        },
        label: if selected {
            c.on_secondary_container
        } else {
            c.on_surface
        },
        supporting: c.on_surface_variant,
        overline: c.on_surface_variant,
        leading_icon: c.on_surface_variant,
        trailing_icon: if selected {
            c.primary
        } else {
            c.on_surface_variant
        },
        trailing_text: c.on_surface_variant,
        avatar: c.primary_container,
        avatar_label: c.on_primary_container,
    }
}

/// The element before the text.
pub enum Leading<'a, Message> {
    /// A 24 dp icon.
    Icon(iced::widget::svg::Handle),
    /// A circle with up to two letters.
    Avatar(String),
    /// Any widget, such as a checkbox.
    Element(Element<'a, Message>),
}

/// The element after the text.
pub enum Trailing<'a, Message> {
    /// A 24 dp icon.
    Icon(iced::widget::svg::Handle),
    /// Short text such as a time.
    Text(String),
    /// Any widget, such as a switch.
    Element(Element<'a, Message>),
}

/// A Material list item.
pub struct ListItem<'a, Message> {
    headline: String,
    supporting: Option<String>,
    overline: Option<String>,
    leading: Option<Leading<'a, Message>>,
    trailing: Option<Trailing<'a, Message>>,
    selected: bool,
    disabled: bool,
    on_press: Option<Message>,
    width: Length,
    metrics: Metrics,
    tokens: Tokens,
    class: StyleFn<'a>,
    styles: [TypeStyle; 5],
}

/// A list item with a headline.
pub fn list_item<'a, Message>(theme: &Theme, headline: impl Into<String>) -> ListItem<'a, Message> {
    let t = &theme.typography;
    ListItem {
        headline: headline.into(),
        supporting: None,
        overline: None,
        leading: None,
        trailing: None,
        selected: false,
        disabled: false,
        on_press: None,
        width: Length::Fill,
        metrics: theme.components.list,
        tokens: Tokens::new(theme),
        class: Box::new(style),
        styles: [
            t.body_large,
            t.body_medium,
            t.label_small,
            t.label_small,
            t.title_medium,
        ],
    }
}

impl<'a, Message> ListItem<'a, Message> {
    /// Adds a line below the headline.
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Adds a line above the headline.
    pub fn overline(mut self, text: impl Into<String>) -> Self {
        self.overline = Some(text.into());
        self
    }

    /// Sets the leading element.
    pub fn leading(mut self, leading: Leading<'a, Message>) -> Self {
        self.leading = Some(leading);
        self
    }

    /// Sets the trailing element.
    pub fn trailing(mut self, trailing: Trailing<'a, Message>) -> Self {
        self.trailing = Some(trailing);
        self
    }

    /// Marks the item as selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Draws the item with the disabled style and ignores presses.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Makes the whole item pressable.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, bool) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn lines(&self) -> usize {
        1 + usize::from(self.supporting.is_some()) + usize::from(self.overline.is_some())
    }

    fn height(&self, text_height: f32) -> f32 {
        let m = &self.metrics;
        let base = match self.lines() {
            1 => m.one_line,
            2 => m.two_line,
            _ => m.three_line,
        };
        base.max(text_height + m.vertical_space * 2.0)
    }

    fn pressable(&self) -> bool {
        self.on_press.is_some() && !self.disabled
    }

    fn elements(&self) -> Vec<&Element<'a, Message>> {
        let leading = match &self.leading {
            Some(Leading::Element(e)) => Some(e),
            _ => None,
        };
        let trailing = match &self.trailing {
            Some(Trailing::Element(e)) => Some(e),
            _ => None,
        };
        leading.into_iter().chain(trailing).collect()
    }

    fn elements_mut(&mut self) -> Vec<&mut Element<'a, Message>> {
        let leading = match &mut self.leading {
            Some(Leading::Element(e)) => Some(e),
            _ => None,
        };
        let trailing = match &mut self.trailing {
            Some(Trailing::Element(e)) => Some(e),
            _ => None,
        };
        leading.into_iter().chain(trailing).collect()
    }
}

#[derive(Default)]
struct State {
    press: pressable::State,
    headline: Label,
    supporting: Label,
    overline: Label,
    trailing: Label,
    avatar: Label,
    sizes: [Size; 2],
}

struct Geometry {
    text: Rectangle,
    leading: Option<Rectangle>,
    trailing: Option<Rectangle>,
    height: f32,
}

impl<Message> ListItem<'_, Message> {
    fn geometry(&self, width: f32, state: &State) -> Geometry {
        let m = &self.metrics;
        let heights = [
            self.overline.as_ref().map(|_| self.styles[2].line_height),
            Some(self.styles[0].line_height),
            self.supporting.as_ref().map(|_| self.styles[1].line_height),
        ];
        let text_height: f32 = heights.iter().flatten().sum();
        let height = self.height(text_height);
        let top_aligned = self.lines() >= 3;
        let place = |size: Size| {
            if top_aligned {
                m.vertical_space
            } else {
                (height - size.height) / 2.0
            }
        };
        let mut x = m.leading_space;
        let leading = self.leading.as_ref().map(|leading| {
            let size = match leading {
                Leading::Icon(_) => Size::new(m.icon_size, m.icon_size),
                Leading::Avatar(_) => Size::new(m.avatar_size, m.avatar_size),
                Leading::Element(_) => state.sizes[0],
            };
            let r = Rectangle::new(Point::new(x, place(size)), size);
            x += size.width + m.gap;
            r
        });
        let mut right = width - m.trailing_space;
        let trailing = self.trailing.as_ref().map(|trailing| {
            let size = match trailing {
                Trailing::Icon(_) => Size::new(m.icon_size, m.icon_size),
                Trailing::Text(_) => state.trailing.size(),
                Trailing::Element(_) => state.sizes[1],
            };
            let r = Rectangle::new(Point::new(right - size.width, place(size)), size);
            right = r.x - m.gap;
            r
        });
        Geometry {
            text: Rectangle::new(
                Point::new(x, (height - text_height) / 2.0),
                Size::new((right - x).max(0.0), text_height),
            ),
            leading,
            trailing,
            height,
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for ListItem<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.elements().into_iter().map(|e| Tree::new(e)).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements = self.elements();
        tree.diff_children_custom(
            &elements,
            |tree, element| tree.diff(element.as_widget()),
            |element| Tree::new(*element),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        state.headline.update(&self.headline, self.styles[0]);
        state
            .supporting
            .update(self.supporting.as_deref().unwrap_or(""), self.styles[1]);
        state
            .overline
            .update(self.overline.as_deref().unwrap_or(""), self.styles[2]);
        let trailing_text = match &self.trailing {
            Some(Trailing::Text(t)) => t.as_str(),
            _ => "",
        };
        state.trailing.update(trailing_text, self.styles[3]);
        if let Some(Leading::Avatar(letters)) = &self.leading {
            state.avatar.update(letters, self.styles[4]);
        }
        let probe = self.height(0.0);
        let width = limits
            .resolve(self.width, Length::Shrink, Size::new(0.0, probe))
            .width;
        let element_limits =
            layout::Limits::new(Size::ZERO, Size::new(width, self.metrics.three_line));
        let mut nodes = Vec::new();
        let mut slots = [false, false];
        slots[0] = matches!(self.leading, Some(Leading::Element(_)));
        slots[1] = matches!(self.trailing, Some(Trailing::Element(_)));
        state.sizes = [Size::ZERO; 2];
        let mut elements = self.elements_mut().into_iter();
        let mut trees = tree.children.iter_mut();
        for (slot, present) in slots.into_iter().enumerate() {
            if !present {
                continue;
            }
            if let (Some(element), Some(child_tree)) = (elements.next(), trees.next()) {
                let node = element
                    .as_widget_mut()
                    .layout(child_tree, renderer, &element_limits);
                state.sizes[slot] = node.size();
                nodes.push((slot, node));
            }
        }
        let geometry = self.geometry(width, tree.state.downcast_ref::<State>());
        let children = nodes
            .into_iter()
            .filter_map(|(slot, node)| {
                let at = if slot == 0 {
                    geometry.leading
                } else {
                    geometry.trailing
                };
                at.map(|r| node.move_to(r.position()))
            })
            .collect();
        layout::Node::with_children(Size::new(width, geometry.height), children)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.pressable() {
            let state = tree.state.downcast_mut::<State>();
            operation.focusable(None, layout.bounds(), &mut state.press.interaction.focus);
        }
        for ((element, tree), layout) in self
            .elements_mut()
            .into_iter()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            element
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
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
        for ((element, tree), layout) in self
            .elements_mut()
            .into_iter()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            element.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
        if self.on_press.is_none() || shell.is_event_captured() {
            return;
        }
        let enabled = self.pressable();
        let state = tree.state.downcast_mut::<State>();
        if state
            .press
            .update(
                event,
                layout.bounds(),
                cursor,
                enabled,
                false,
                &self.tokens,
                shell,
            )
            .is_some_and(|a| matches!(a, Activation::Pointer | Activation::Keyboard))
            && let Some(message) = &self.on_press
        {
            shell.publish(message.clone());
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let origin = bounds.position();
        let geometry = self.geometry(bounds.width, state);
        let enabled = !self.disabled;
        let selected = self.selected;
        let look = theme.style(&self.class, state.press.status(enabled, false), selected);
        pressable::draw_container(
            renderer,
            theme,
            bounds,
            Radius::default(),
            &look.base,
            0.0,
            &state.press,
        );
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let m = &self.metrics;
        let icon = |renderer: &mut Renderer,
                    handle: &iced::widget::svg::Handle,
                    at: Rectangle,
                    color: Color| {
            renderer.draw_svg(
                tinted(handle.clone(), color, 1.0),
                Rectangle::new(
                    Point::new(origin.x + at.x, origin.y + at.y),
                    Size::new(m.icon_size, m.icon_size),
                ),
                clip,
            );
        };
        if let (Some(leading), Some(at)) = (&self.leading, geometry.leading) {
            match leading {
                Leading::Icon(handle) => icon(renderer, handle, at, look.leading_icon),
                Leading::Avatar(_) => {
                    let circle =
                        Rectangle::new(Point::new(origin.x + at.x, origin.y + at.y), at.size());
                    surface::fill(renderer, circle, Radius::from(at.width / 2.0), look.avatar);
                    let size = state.avatar.size();
                    state.avatar.draw(
                        renderer,
                        Point::new(
                            circle.center_x() - size.width / 2.0,
                            circle.center_y() - size.height / 2.0,
                        ),
                        look.avatar_label,
                        clip,
                    );
                }
                Leading::Element(_) => {}
            }
        }
        if let (Some(trailing), Some(at)) = (&self.trailing, geometry.trailing) {
            match trailing {
                Trailing::Icon(handle) => icon(renderer, handle, at, look.trailing_icon),
                Trailing::Text(_) => state.trailing.draw(
                    renderer,
                    Point::new(origin.x + at.x, origin.y + at.y),
                    look.trailing_text,
                    clip,
                ),
                Trailing::Element(_) => {}
            }
        }
        let text_clip = Rectangle::new(
            Point::new(origin.x + geometry.text.x, bounds.y),
            Size::new(geometry.text.width, bounds.height),
        )
        .intersection(&clip)
        .unwrap_or(clip);
        let mut y = origin.y + geometry.text.y;
        let x = origin.x + geometry.text.x;
        for (present, label, color, line) in [
            (
                self.overline.is_some(),
                &state.overline,
                look.overline,
                self.styles[2].line_height,
            ),
            (
                true,
                &state.headline,
                look.label,
                self.styles[0].line_height,
            ),
            (
                self.supporting.is_some(),
                &state.supporting,
                look.supporting,
                self.styles[1].line_height,
            ),
        ] {
            if present {
                label.draw(renderer, Point::new(x, y), color, text_clip);
                y += line;
            }
        }
        let mut children = layout.children();
        for (element, tree) in self.elements().into_iter().zip(&tree.children) {
            if let Some(child) = children.next() {
                element.as_widget().draw(
                    tree,
                    renderer,
                    theme,
                    &renderer::Style {
                        text_color: look.label,
                    },
                    child,
                    cursor,
                    viewport,
                );
            }
        }
        if self.pressable() {
            let ring = state.press.interaction.focus_ring_width(&self.tokens);
            let inward = theme.focus_ring.inward_offset;
            focus_ring::draw(
                renderer,
                bounds.shrink(inward + ring),
                Radius::default(),
                0.0,
                ring,
                theme.colors.secondary,
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
        for ((element, tree), layout) in self
            .elements()
            .into_iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            let inner = element
                .as_widget()
                .mouse_interaction(tree, layout, cursor, viewport, renderer);
            if inner != mouse::Interaction::default() {
                return inner;
            }
        }
        if self.pressable() && cursor.is_over(layout.bounds()) {
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
        let children: Vec<_> = self
            .elements_mut()
            .into_iter()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((element, tree), layout)| {
                element
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect();
        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message: Clone + 'a> From<ListItem<'a, Message>> for Element<'a, Message> {
    fn from(item: ListItem<'a, Message>) -> Self {
        IcedElement::new(item)
    }
}
