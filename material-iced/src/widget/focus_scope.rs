// SPDX-License-Identifier: LGPL-3.0-only

//! Keyboard focus traversal with Tab and Shift+Tab.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, operation};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key::Named};
use iced::{Event, Length, Rectangle, Renderer, Size, Vector, mouse};

use crate::Element;
use crate::theme::Theme;

/// Moves keyboard focus between focusable widgets of its content on Tab and Shift+Tab.
///
/// Iced has no built-in focus traversal. Wrap the root of a view in a
/// focus scope to get it.
pub struct FocusScope<'a, Message> {
    content: Element<'a, Message>,
}

/// Creates a [`FocusScope`].
pub fn focus_scope<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> FocusScope<'a, Message> {
    FocusScope {
        content: content.into(),
    }
}

fn run(
    content: &mut Element<'_, impl Sized>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    operation: &mut dyn Operation,
) {
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, operation);
    if let operation::Outcome::Chain(mut next) = operation.finish() {
        run(content, tree, layout, renderer, next.as_mut());
    }
}

impl<Message> Widget<Message, Theme, Renderer> for FocusScope<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(Named::Tab),
            modifiers,
            ..
        }) = event
        {
            let child = &mut tree.children[0];
            if modifiers.shift() {
                let mut operation = operation::focusable::focus_previous::<()>();
                run(&mut self.content, child, layout, renderer, &mut operation);
            } else {
                let mut operation = operation::focusable::focus_next::<()>();
                run(&mut self.content, child, layout, renderer, &mut operation);
            }
            shell.capture_event();
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<FocusScope<'a, Message>> for Element<'a, Message> {
    fn from(scope: FocusScope<'a, Message>) -> Self {
        Element::new(scope)
    }
}
