// SPDX-License-Identifier: LGPL-3.0-only

//! Exposed dropdown menus: a text field that shows the chosen option and opens a menu.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::keyboard::{self, key::Named};
use iced::{Color, Element as IcedElement, Event, Length, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::text::Label;
use crate::icon::symbol;
use crate::interaction::Focus;
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::menu::{self, item};
use crate::widget::text_field::{self, Kind};

struct ValueBox<Message> {
    text: String,
    on_toggle: Option<Message>,
    style: TypeStyle,
    color: Color,
}

#[derive(Default)]
struct ValueState {
    focus: Focus,
    label: Label,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for ValueBox<Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ValueState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ValueState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.style.line_height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        tree.state
            .downcast_mut::<ValueState>()
            .label
            .update(&self.text, self.style);
        layout::Node::new(limits.resolve(
            Length::Fill,
            Length::Fixed(self.style.line_height),
            Size::ZERO,
        ))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_toggle.is_some() {
            operation.focusable(
                None,
                layout.bounds(),
                &mut tree.state.downcast_mut::<ValueState>().focus,
            );
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
        let Some(message) = &self.on_toggle else {
            return;
        };
        let focus = &mut tree.state.downcast_mut::<ValueState>().focus;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(layout.bounds()) {
                    focus.focused = true;
                    focus.visible = false;
                    shell.publish(message.clone());
                    shell.capture_event();
                } else {
                    iced::advanced::widget::operation::Focusable::unfocus(focus);
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key:
                    keyboard::Key::Named(
                        Named::Enter | Named::Space | Named::ArrowDown | Named::ArrowUp,
                    ),
                ..
            }) if focus.focused => {
                shell.publish(message.clone());
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        tree.state.downcast_ref::<ValueState>().label.draw(
            renderer,
            bounds.position(),
            self.color,
            clip,
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
        if self.on_toggle.is_some() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

/// A Material select field.
pub struct Select<'a, Message> {
    kind: Kind,
    options: Vec<String>,
    selected: Option<usize>,
    open: bool,
    label: Option<String>,
    supporting: Option<String>,
    error: bool,
    width: Length,
    on_toggle: Option<Message>,
    on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_dismiss: Option<Message>,
}

fn new<'a, Message>(
    kind: Kind,
    options: impl IntoIterator<Item = impl Into<String>>,
    selected: Option<usize>,
    open: bool,
) -> Select<'a, Message> {
    Select {
        kind,
        options: options.into_iter().map(Into::into).collect(),
        selected,
        open,
        label: None,
        supporting: None,
        error: false,
        width: Length::Fill,
        on_toggle: None,
        on_select: None,
        on_dismiss: None,
    }
}

/// A filled select. Without `on_toggle` it is disabled.
pub fn filled<'a, Message>(
    options: impl IntoIterator<Item = impl Into<String>>,
    selected: Option<usize>,
    open: bool,
) -> Select<'a, Message> {
    new(Kind::Filled, options, selected, open)
}

/// An outlined select. Without `on_toggle` it is disabled.
pub fn outlined<'a, Message>(
    options: impl IntoIterator<Item = impl Into<String>>,
    selected: Option<usize>,
    open: bool,
) -> Select<'a, Message> {
    new(Kind::Outlined, options, selected, open)
}

impl<'a, Message> Select<'a, Message> {
    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the text below the field.
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Shows the error colors.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the message produced when the field asks to open the menu.
    pub fn on_toggle(mut self, message: Message) -> Self {
        self.on_toggle = Some(message);
        self
    }

    /// Sets the message produced with the index of the chosen option.
    pub fn on_select(mut self, on_select: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(on_select));
        self
    }

    /// Sets the message produced when the menu should close.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }
}

impl<'a, Message: Clone + 'a> Select<'a, Message> {
    /// Builds the field with its menu.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        let enabled = self.on_toggle.is_some();
        let mut style = theme.typography.body_large;
        style.tracking = 0.0;
        let color = if enabled {
            theme.colors.on_surface
        } else {
            alpha(theme.colors.on_surface, theme.disabled.content)
        };
        let text = self
            .selected
            .and_then(|i| self.options.get(i))
            .cloned()
            .unwrap_or_default();
        let populated = !text.is_empty();
        let value = IcedElement::new(ValueBox {
            text,
            on_toggle: self.on_toggle.clone(),
            style,
            color,
        });
        let icon = if self.open {
            symbol::arrow_drop_up(false)
        } else {
            symbol::arrow_drop_down(false)
        };
        let mut field = text_field::custom(theme, self.kind, value, populated, enabled)
            .active(self.open)
            .trailing_icon(icon)
            .error(self.error)
            .width(self.width);
        if let Some(label) = self.label {
            field = field.label(label);
        }
        if let Some(text) = self.supporting {
            field = field.supporting_text(text);
        }
        let items: Vec<_> = self
            .options
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                let entry = item(label).selected(self.selected == Some(i));
                match &self.on_select {
                    Some(on_select) => entry.on_select(on_select(i)),
                    None => entry,
                }
            })
            .collect();
        menu::popup(theme, field.into(), items, self.open, self.on_dismiss, true)
    }
}
