// SPDX-License-Identifier: LGPL-3.0-only

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::{Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::calendar::{Date, Names, days_in_month};
use crate::draw::text::Label;
use crate::draw::{focus_ring, surface};
use crate::interaction::Focus;
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;

pub(crate) const ROW: f32 = 48.0;
pub(crate) const COLUMN: f32 = 48.0;

pub(crate) struct DayGrid<'a, Message> {
    year: i32,
    month: u8,
    selected: Option<Date>,
    today: Date,
    min: Date,
    max: Date,
    names: Names,
    container: f32,
    on_pick: Box<dyn Fn(Date) -> Message + 'a>,
    on_month: Box<dyn Fn(i32) -> Message + 'a>,
    style: TypeStyle,
    state_layer: f32,
    hover: f32,
    pressed: f32,
}

impl<'a, Message> DayGrid<'a, Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        theme: &Theme,
        shown: (i32, u8),
        selected: Option<Date>,
        today: Date,
        range: (Date, Date),
        names: Names,
        container: f32,
        on_pick: impl Fn(Date) -> Message + 'a,
        on_month: impl Fn(i32) -> Message + 'a,
    ) -> Self {
        DayGrid {
            year: shown.0,
            month: shown.1,
            selected,
            today,
            min: range.0,
            max: range.1,
            names,
            container,
            on_pick: Box::new(on_pick),
            on_month: Box::new(on_month),
            style: theme.typography.body_large,
            state_layer: 40.0,
            hover: theme.state.hover,
            pressed: theme.state.pressed,
        }
    }

    fn offset(&self) -> usize {
        let first = Date {
            year: self.year,
            month: self.month,
            day: 1,
        };
        (usize::from(first.weekday()) + 7 - usize::from(self.names.first_weekday)) % 7
    }

    fn cell(&self, bounds: Rectangle, day: u8) -> Rectangle {
        let index = self.offset() + usize::from(day) - 1;
        Rectangle::new(
            Point::new(
                bounds.x + (index % 7) as f32 * COLUMN,
                bounds.y + ROW + (index / 7) as f32 * ROW,
            ),
            Size::new(COLUMN, ROW),
        )
    }

    fn day_at(&self, bounds: Rectangle, point: Point) -> Option<u8> {
        if point.y < bounds.y + ROW || !bounds.contains(point) {
            return None;
        }
        let col = ((point.x - bounds.x) / COLUMN) as usize;
        let row = ((point.y - bounds.y - ROW) / ROW) as usize;
        let index = row * 7 + col;
        let day = index.checked_sub(self.offset())? + 1;
        (day <= usize::from(days_in_month(self.year, self.month))).then_some(day as u8)
    }

    fn enabled(&self, day: u8) -> bool {
        let date = Date {
            year: self.year,
            month: self.month,
            day,
        };
        date >= self.min && date <= self.max
    }
}

struct GridState {
    focus: Focus,
    day: u8,
    hover: Option<u8>,
    pressed: Option<u8>,
    weekdays: Vec<Label>,
    days: Vec<Label>,
    key: (i32, u8),
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for DayGrid<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GridState>()
    }

    fn state(&self) -> tree::State {
        let initial = self
            .selected
            .filter(|d| d.year == self.year && d.month == self.month)
            .map_or(1, |d| d.day);
        tree::State::new(GridState {
            focus: Focus::default(),
            day: initial,
            hover: None,
            pressed: None,
            weekdays: Vec::new(),
            days: Vec::new(),
            key: (self.year, self.month),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(COLUMN * 7.0), Length::Fixed(ROW * 7.0))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<GridState>();
        if state.weekdays.is_empty() {
            state.weekdays = (0..7).map(|_| Label::default()).collect();
            state.days = (0..31).map(|_| Label::default()).collect();
        }
        for i in 0..7 {
            let name = self.names.weekday_initials[(usize::from(self.names.first_weekday) + i) % 7];
            state.weekdays[i].update(name, self.style);
        }
        for (i, label) in state.days.iter_mut().enumerate() {
            label.update(&(i + 1).to_string(), self.style);
        }
        if state.key != (self.year, self.month) {
            state.key = (self.year, self.month);
            state.day = state.day.min(days_in_month(self.year, self.month));
        }
        layout::Node::new(Size::new(COLUMN * 7.0, ROW * 7.0))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<GridState>();
        let rect = self.cell(layout.bounds(), state.day);
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
            .and_then(|p| self.day_at(bounds, p))
            .filter(|d| self.enabled(*d));
        if hovered != state.hover {
            state.hover = hovered;
            shell.request_redraw();
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(day) = hovered {
                    state.pressed = Some(day);
                    state.day = day;
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
                if let Some(day) = state.pressed.take() {
                    if hovered == Some(day) {
                        shell.publish((self.on_pick)(Date {
                            year: self.year,
                            month: self.month,
                            day,
                        }));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) if state.focus.focused => {
                let current = Date {
                    year: self.year,
                    month: self.month,
                    day: state.day,
                };
                let step = match key {
                    Named::ArrowLeft => Some(-1),
                    Named::ArrowRight => Some(1),
                    Named::ArrowUp => Some(-7),
                    Named::ArrowDown => Some(7),
                    _ => None,
                };
                if let Some(step) = step {
                    let target = current.add_days(step).max(self.min).min(self.max);
                    state.day = target.day;
                    state.focus.visible = true;
                    if (target.year, target.month) != (self.year, self.month) {
                        let delta = (target.year - self.year) * 12 + i32::from(target.month)
                            - i32::from(self.month);
                        state.key = (target.year, target.month);
                        shell.publish((self.on_month)(delta));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                } else if matches!(key, Named::Enter | Named::Space) && self.enabled(state.day) {
                    shell.publish((self.on_pick)(current));
                    shell.capture_event();
                } else if matches!(key, Named::Home | Named::End) {
                    state.day = if matches!(key, Named::Home) {
                        1
                    } else {
                        days_in_month(self.year, self.month)
                    };
                    shell.capture_event();
                    shell.request_redraw();
                } else if matches!(key, Named::PageUp | Named::PageDown) {
                    let delta = if matches!(key, Named::PageUp) { -1 } else { 1 };
                    let target = current.add_months(delta);
                    state.day = target.day;
                    state.key = (target.year, target.month);
                    shell.publish((self.on_month)(delta));
                    shell.capture_event();
                    shell.request_redraw();
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
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let center = |label: &Label, rect: Rectangle| {
            let size = label.size();
            Point::new(
                rect.center_x() - size.width / 2.0,
                rect.center_y() - size.height / 2.0,
            )
        };
        for (i, label) in state.weekdays.iter().enumerate() {
            let rect = Rectangle::new(
                Point::new(bounds.x + i as f32 * COLUMN, bounds.y),
                Size::new(COLUMN, ROW),
            );
            label.draw(renderer, center(label, rect), c.on_surface, clip);
        }
        let layer = |rect: Rectangle, size: f32| {
            Rectangle::new(
                Point::new(rect.center_x() - size / 2.0, rect.center_y() - size / 2.0),
                Size::new(size, size),
            )
        };
        for day in 1..=days_in_month(self.year, self.month) {
            let rect = self.cell(bounds, day);
            let date = Date {
                year: self.year,
                month: self.month,
                day,
            };
            let selected = self.selected == Some(date);
            let enabled = self.enabled(day);
            let (label_color, on_layer) = if !enabled {
                (alpha(c.on_surface, theme.disabled.content), c.on_surface)
            } else if selected {
                (c.on_primary, c.on_primary)
            } else if date == self.today {
                (c.primary, c.primary)
            } else {
                (c.on_surface, c.on_surface)
            };
            if selected {
                let r = layer(rect, self.container);
                surface::fill(renderer, r, Radius::from(r.width / 2.0), c.primary);
            } else if date == self.today && enabled {
                let r = layer(rect, self.container);
                surface::outline(renderer, r, Radius::from(r.width / 2.0), 1.0, c.primary);
            }
            if enabled {
                let amount = if state.pressed == Some(day) {
                    self.pressed
                } else if state.hover == Some(day) {
                    self.hover
                } else {
                    0.0
                };
                let r = layer(rect, self.state_layer);
                surface::state_layer(renderer, r, Radius::from(r.width / 2.0), on_layer, amount);
            }
            let label = &state.days[usize::from(day) - 1];
            label.draw(renderer, center(label, rect), label_color, clip);
            if state.focus.visible && state.day == day {
                let r = layer(rect, self.state_layer);
                focus_ring::draw(
                    renderer,
                    r,
                    Radius::from(r.width / 2.0),
                    theme.focus_ring.outward_offset,
                    theme.focus_ring.width,
                    c.secondary,
                );
            }
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
        match cursor
            .position()
            .and_then(|p| self.day_at(layout.bounds(), p))
        {
            Some(day) if self.enabled(day) => mouse::Interaction::Pointer,
            _ => mouse::Interaction::default(),
        }
    }
}

impl<'a, Message: Clone + 'a> From<DayGrid<'a, Message>> for Element<'a, Message> {
    fn from(grid: DayGrid<'a, Message>) -> Self {
        IcedElement::new(grid)
    }
}
