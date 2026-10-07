// SPDX-License-Identifier: LGPL-3.0-only

use std::ops::RangeInclusive;

use iced::advanced::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::mouse::ScrollDelta;
use iced::{Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::text::Label;
use crate::draw::{focus_ring, surface};
use crate::interaction::Focus;
use crate::theme::Theme;
use crate::typography::TypeStyle;

const COLUMNS: usize = 3;
const CELL: Size = Size::new(72.0, 36.0);
const ROW: f32 = 52.0;

pub(crate) struct YearGrid<'a, Message> {
    years: RangeInclusive<i32>,
    selected: i32,
    today: i32,
    height: f32,
    on_pick: Box<dyn Fn(i32) -> Message + 'a>,
    style: TypeStyle,
    hover: f32,
    pressed: f32,
}

impl<'a, Message> YearGrid<'a, Message> {
    pub fn new(
        theme: &Theme,
        years: RangeInclusive<i32>,
        selected: i32,
        today: i32,
        height: f32,
        on_pick: impl Fn(i32) -> Message + 'a,
    ) -> Self {
        YearGrid {
            years,
            selected,
            today,
            height,
            on_pick: Box::new(on_pick),
            style: theme.typography.body_large,
            hover: theme.state.hover,
            pressed: theme.state.pressed,
        }
    }

    fn count(&self) -> usize {
        (*self.years.end() - *self.years.start() + 1).max(0) as usize
    }

    fn rows(&self) -> usize {
        self.count().div_ceil(COLUMNS)
    }

    fn max_offset(&self) -> f32 {
        (self.rows() as f32 * ROW - self.height).max(0.0)
    }

    fn index_of(&self, year: i32) -> usize {
        (year - *self.years.start()).clamp(0, self.count() as i32 - 1) as usize
    }

    fn cell(&self, bounds: Rectangle, index: usize, offset: f32) -> Rectangle {
        let pitch = bounds.width / COLUMNS as f32;
        let row = (index / COLUMNS) as f32;
        let col = (index % COLUMNS) as f32;
        Rectangle::new(
            Point::new(
                bounds.x + col * pitch + (pitch - CELL.width) / 2.0,
                bounds.y + row * ROW - offset + (ROW - CELL.height) / 2.0,
            ),
            CELL,
        )
    }

    fn index_at(&self, bounds: Rectangle, point: Point, offset: f32) -> Option<usize> {
        if !bounds.contains(point) {
            return None;
        }
        let col = ((point.x - bounds.x) / (bounds.width / COLUMNS as f32)) as usize;
        let row = ((point.y - bounds.y + offset) / ROW) as usize;
        let index = row * COLUMNS + col.min(COLUMNS - 1);
        (index < self.count()).then_some(index)
    }
}

struct GridState {
    focus: Focus,
    offset: f32,
    cursor: usize,
    hover: Option<usize>,
    pressed: Option<usize>,
    labels: Vec<Label>,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for YearGrid<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GridState>()
    }

    fn state(&self) -> tree::State {
        let index = self.index_of(self.selected);
        let first_row = (index / COLUMNS).saturating_sub(1);
        tree::State::new(GridState {
            focus: Focus::default(),
            offset: (first_row as f32 * ROW).min(self.max_offset()),
            cursor: index,
            hover: None,
            pressed: None,
            labels: Vec::new(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<GridState>();
        if state.labels.len() != self.count() {
            state.labels = (0..self.count()).map(|_| Label::default()).collect();
        }
        for (i, label) in state.labels.iter_mut().enumerate() {
            label.update(&(*self.years.start() + i as i32).to_string(), self.style);
        }
        layout::Node::new(limits.resolve(
            Length::Fill,
            Length::Fixed(self.height),
            Size::new(CELL.width * COLUMNS as f32, self.height),
        ))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<GridState>();
        let rect = self.cell(layout.bounds(), state.cursor, state.offset);
        operation.focusable(None, rect, &mut state.focus);
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
        let state = tree.state.downcast_mut::<GridState>();
        let bounds = layout.bounds();
        let hovered = cursor
            .position()
            .and_then(|p| self.index_at(bounds, p, state.offset));
        if hovered != state.hover {
            state.hover = hovered;
            shell.request_redraw();
        }
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let y = match delta {
                    ScrollDelta::Lines { y, .. } => y * ROW,
                    ScrollDelta::Pixels { y, .. } => *y,
                };
                state.offset = (state.offset - y).clamp(0.0, self.max_offset());
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(index) = hovered {
                    state.pressed = Some(index);
                    state.cursor = index;
                    state.focus.focused = true;
                    state.focus.visible = false;
                    shell.capture_event();
                    shell.request_redraw();
                } else if state.focus.focused {
                    state.focus.unfocus();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(index) = state.pressed.take() {
                    if hovered == Some(index) {
                        shell.publish((self.on_pick)(*self.years.start() + index as i32));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) if state.focus.focused => {
                let step: Option<i32> = match key {
                    Named::ArrowLeft => Some(-1),
                    Named::ArrowRight => Some(1),
                    Named::ArrowUp => Some(-(COLUMNS as i32)),
                    Named::ArrowDown => Some(COLUMNS as i32),
                    _ => None,
                };
                if let Some(step) = step {
                    let index = (state.cursor as i32 + step).clamp(0, self.count() as i32 - 1);
                    state.cursor = index as usize;
                    state.focus.visible = true;
                    let top = (state.cursor / COLUMNS) as f32 * ROW;
                    if top < state.offset {
                        state.offset = top;
                    } else if top + ROW > state.offset + self.height {
                        state.offset = top + ROW - self.height;
                    }
                    shell.capture_event();
                    shell.request_redraw();
                } else if matches!(key, Named::Enter | Named::Space) {
                    shell.publish((self.on_pick)(*self.years.start() + state.cursor as i32));
                    shell.capture_event();
                }
            }
            _ => {}
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
        let state = tree.state.downcast_ref::<GridState>();
        let bounds = layout.bounds();
        let c = &theme.colors;
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            for (index, label) in state.labels.iter().enumerate() {
                let rect = self.cell(bounds, index, state.offset);
                if rect.y + rect.height < bounds.y || rect.y > bounds.y + bounds.height {
                    continue;
                }
                let year = *self.years.start() + index as i32;
                let radius = Radius::from(rect.height / 2.0);
                let selected = year == self.selected;
                let color = if selected {
                    surface::fill(renderer, rect, radius, c.primary);
                    c.on_primary
                } else if year == self.today {
                    c.primary
                } else {
                    c.on_surface_variant
                };
                let layer_color = if selected {
                    c.on_primary
                } else {
                    c.on_surface_variant
                };
                let amount = if state.pressed == Some(index) {
                    self.pressed
                } else if state.hover == Some(index) {
                    self.hover
                } else {
                    0.0
                };
                surface::state_layer(renderer, rect, radius, layer_color, amount);
                let size = label.size();
                label.draw(
                    renderer,
                    Point::new(
                        rect.center_x() - size.width / 2.0,
                        rect.center_y() - size.height / 2.0,
                    ),
                    color,
                    clip,
                );
                if state.focus.visible && state.cursor == index {
                    focus_ring::draw(
                        renderer,
                        rect,
                        radius,
                        theme.focus_ring.outward_offset,
                        theme.focus_ring.width,
                        c.secondary,
                    );
                }
            }
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<GridState>();
        match cursor
            .position()
            .and_then(|p| self.index_at(layout.bounds(), p, state.offset))
        {
            Some(_) => mouse::Interaction::Pointer,
            None => mouse::Interaction::default(),
        }
    }
}

impl<'a, Message: Clone + 'a> From<YearGrid<'a, Message>> for Element<'a, Message> {
    fn from(grid: YearGrid<'a, Message>) -> Self {
        IcedElement::new(grid)
    }
}
