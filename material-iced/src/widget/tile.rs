// SPDX-License-Identifier: LGPL-3.0-only

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::{Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::text::Label;
use crate::interaction::Tokens;
use crate::shape::Shape;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::{self, Style};

pub(crate) struct Colors {
    pub container: Color,
    pub label: Color,
    pub layer: Color,
    pub outline: Color,
    pub outline_width: f32,
}

pub(crate) struct Tile<Message> {
    label: String,
    style: TypeStyle,
    colors: Colors,
    shape: Shape,
    size: Size,
    on_press: Message,
    tokens: Tokens,
    shadow: Color,
}

impl<Message> Tile<Message> {
    pub fn new(
        theme: &Theme,
        label: impl Into<String>,
        style: TypeStyle,
        colors: Colors,
        shape: Shape,
        size: Size,
        on_press: Message,
    ) -> Self {
        Tile {
            label: label.into(),
            style,
            colors,
            shape,
            size,
            on_press,
            tokens: Tokens::new(theme),
            shadow: theme.colors.shadow,
        }
    }
}

#[derive(Default)]
struct State {
    press: pressable::State,
    label: Label,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Tile<Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.size.width),
            Length::Fixed(self.size.height),
        )
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        tree.state
            .downcast_mut::<State>()
            .label
            .update(&self.label, self.style);
        layout::Node::new(self.size)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        operation.focusable(None, layout.bounds(), &mut state.press.interaction.focus);
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
        let state = tree.state.downcast_mut::<State>();
        if state
            .press
            .update(
                event,
                layout.bounds(),
                cursor,
                true,
                false,
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let radius = self.shape.radius(bounds.size());
        let c = &self.colors;
        let style = Style {
            container: c.container,
            outline: c.outline,
            outline_width: c.outline_width,
            label: c.label,
            icon: c.label,
            hover_layer: c.layer,
            pressed_layer: c.layer,
            elevation: 0.0,
            shadow: self.shadow,
        };
        pressable::draw_container(renderer, theme, bounds, radius, &style, 0.0, &state.press);
        let size = state.label.size();
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        state.label.draw(
            renderer,
            Point::new(
                bounds.center_x() - size.width / 2.0,
                bounds.center_y() - size.height / 2.0,
            ),
            c.label,
            clip,
        );
        pressable::draw_focus(renderer, theme, bounds, radius, &state.press, &self.tokens);
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

impl<'a, Message: Clone + 'a> From<Tile<Message>> for Element<'a, Message> {
    fn from(tile: Tile<Message>) -> Self {
        IcedElement::new(tile)
    }
}
