// SPDX-License-Identifier: LGPL-3.0-only

//! A wrapping row of chips with arrow key navigation.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use std::any::Any;

use iced::advanced::widget::{Id, Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, key::Named};
use iced::widget::row;
use iced::{Event, Length, Rectangle, Renderer, Size, Vector, mouse};

use crate::Element;
use crate::theme::Theme;
use crate::widget::chip;

/// Lays out chips in a wrapping row with 8 dp gaps.
///
/// Left and Right move focus to the previous and next enabled chip and wrap
/// around. Home and End focus the first and last chip.
pub struct ChipSet<'a, Message> {
    content: Element<'a, Message>,
}

/// Creates a [`ChipSet`].
pub fn chip_set<'a, Message: 'a>(
    theme: &Theme,
    chips: impl IntoIterator<Item = Element<'a, Message>>,
) -> ChipSet<'a, Message> {
    ChipSet {
        content: row(chips)
            .spacing(theme.components.chip_set.gap)
            .wrap()
            .vertical_spacing(theme.components.chip_set.gap)
            .into(),
    }
}

/// Chip set dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Horizontal and vertical space between chips.
    pub gap: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics { gap: 8.0 }
    }
}

#[derive(Default)]
struct Scan {
    enabled: Vec<bool>,
    focused: Option<usize>,
}

impl Operation for Scan {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if let Some(chip) = state.downcast_mut::<chip::State>() {
            if chip.is_focused() {
                self.focused = Some(self.enabled.len());
            }
            self.enabled.push(chip.enabled);
        }
    }
}

struct Focus {
    target: usize,
    trailing: bool,
    index: usize,
}

impl Operation for Focus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if let Some(chip) = state.downcast_mut::<chip::State>() {
            chip.unfocus();
            if self.index == self.target {
                chip.focus(self.trailing);
            }
            self.index += 1;
        }
    }
}

fn target(key: Named, enabled: &[bool], focused: Option<usize>) -> Option<(usize, bool)> {
    let n = enabled.len();
    let first = enabled.iter().position(|e| *e)?;
    let last = enabled.iter().rposition(|e| *e)?;
    match (key, focused) {
        (Named::Home, _) => Some((first, false)),
        (Named::End, _) => Some((last, true)),
        (Named::ArrowRight, None) => Some((first, false)),
        (Named::ArrowLeft, None) => Some((last, true)),
        (_, Some(current)) => {
            let forward = key == Named::ArrowRight;
            let step = if forward { 1 } else { n - 1 };
            let mut next = (current + step) % n;
            while next != current {
                if enabled[next] {
                    return Some((next, !forward));
                }
                next = (next + step) % n;
            }
            None
        }
        _ => None,
    }
}

impl<Message> Widget<Message, Theme, Renderer> for ChipSet<'_, Message> {
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
            key:
                keyboard::Key::Named(
                    key @ (Named::ArrowLeft | Named::ArrowRight | Named::Home | Named::End),
                ),
            ..
        }) = event
        {
            let child = &mut tree.children[0];
            let mut scan = Scan::default();
            self.content
                .as_widget_mut()
                .operate(child, layout, renderer, &mut scan);
            if scan.focused.is_none() && matches!(key, Named::Home | Named::End) {
                return;
            }
            if let Some((target, trailing)) = target(*key, &scan.enabled, scan.focused)
                && scan.enabled.len() > 1
            {
                let mut focus = Focus {
                    target,
                    trailing,
                    index: 0,
                };
                self.content
                    .as_widget_mut()
                    .operate(child, layout, renderer, &mut focus);
                shell.capture_event();
                shell.request_redraw();
            }
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

impl<'a, Message: 'a> From<ChipSet<'a, Message>> for Element<'a, Message> {
    fn from(set: ChipSet<'a, Message>) -> Self {
        Element::new(set)
    }
}
