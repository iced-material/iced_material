// SPDX-License-Identifier: LGPL-3.0-only

use iced::advanced::Renderer as _;
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::keyboard::{self, key::Named};
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::text::Label;
use crate::draw::{focus_ring, surface};
use crate::interaction::Focus;
use crate::theme::Theme;
use crate::typography::TypeStyle;

const SIZE: f32 = 256.0;
const OUTER: f32 = 101.0;
const INNER: f32 = 69.0;
const HANDLE: f32 = 48.0;
const CENTER_DOT: f32 = 8.0;
const TRACK: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Value {
    Hour12(u8),
    Hour24(u8),
    Minute(u8),
}

pub(crate) struct Dial<'a, Message> {
    value: Value,
    on_change: Box<dyn Fn(Value, bool) -> Message + 'a>,
    style: TypeStyle,
}

impl<'a, Message> Dial<'a, Message> {
    pub fn new(
        theme: &Theme,
        value: Value,
        on_change: impl Fn(Value, bool) -> Message + 'a,
    ) -> Self {
        Dial {
            value,
            on_change: Box::new(on_change),
            style: theme.typography.body_large,
        }
    }

    fn texts(&self) -> (Vec<String>, Vec<String>) {
        match self.value {
            Value::Minute(_) => ((0..12).map(|i| (i * 5).to_string()).collect(), Vec::new()),
            Value::Hour12(_) => (hours_outer(), Vec::new()),
            Value::Hour24(_) => (
                hours_outer(),
                (0..12)
                    .map(|i| {
                        if i == 0 {
                            "00".to_string()
                        } else {
                            (i + 12).to_string()
                        }
                    })
                    .collect(),
            ),
        }
    }

    fn selection(&self) -> (f32, f32) {
        match self.value {
            Value::Minute(m) => (f32::from(m) * 6.0, OUTER),
            Value::Hour12(h) => (f32::from(h % 12) * 30.0, OUTER),
            Value::Hour24(h) if h == 0 || h >= 13 => {
                (f32::from(if h == 0 { 0 } else { h - 12 }) * 30.0, INNER)
            }
            Value::Hour24(h) => (f32::from(h % 12) * 30.0, OUTER),
        }
    }

    fn at(&self, bounds: Rectangle, point: Point) -> Value {
        let dx = point.x - bounds.center_x();
        let dy = point.y - bounds.center_y();
        let mut angle = dx.atan2(-dy).to_degrees();
        if angle < 0.0 {
            angle += 360.0;
        }
        match self.value {
            Value::Minute(_) => Value::Minute((angle / 6.0).round() as u8 % 60),
            Value::Hour12(_) => {
                let i = (angle / 30.0).round() as u8 % 12;
                Value::Hour12(if i == 0 { 12 } else { i })
            }
            Value::Hour24(_) => {
                let i = (angle / 30.0).round() as u8 % 12;
                let inner = (dx * dx + dy * dy).sqrt() < (OUTER + INNER) / 2.0;
                Value::Hour24(match (inner, i) {
                    (false, 0) => 12,
                    (false, i) => i,
                    (true, 0) => 0,
                    (true, i) => i + 12,
                })
            }
        }
    }

    fn step(&self, delta: i32) -> Value {
        let wrap = |v: u8, lo: i32, hi: i32| {
            let span = hi - lo + 1;
            (i32::from(v) - lo + delta).rem_euclid(span) as u8 + lo as u8
        };
        match self.value {
            Value::Minute(m) => Value::Minute(wrap(m, 0, 59)),
            Value::Hour12(h) => Value::Hour12(wrap(h, 1, 12)),
            Value::Hour24(h) => Value::Hour24(wrap(h, 0, 23)),
        }
    }
}

fn hours_outer() -> Vec<String> {
    (0..12)
        .map(|i| {
            if i == 0 {
                "12".to_string()
            } else {
                i.to_string()
            }
        })
        .collect()
}

fn point(center: Point, angle: f32, radius: f32) -> Point {
    let a = angle.to_radians();
    Point::new(center.x + radius * a.sin(), center.y - radius * a.cos())
}

struct DialState {
    focus: Focus,
    dragging: bool,
    outer: Vec<Label>,
    inner: Vec<Label>,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Dial<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DialState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DialState {
            focus: Focus::default(),
            dragging: false,
            outer: Vec::new(),
            inner: Vec::new(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(SIZE), Length::Fixed(SIZE))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<DialState>();
        let (outer, inner) = self.texts();
        for (labels, texts) in [(&mut state.outer, outer), (&mut state.inner, inner)] {
            labels.resize_with(texts.len(), Label::default);
            for (label, text) in labels.iter_mut().zip(&texts) {
                label.update(text, self.style);
            }
        }
        layout::Node::new(Size::new(SIZE, SIZE))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let bounds = layout.bounds();
        let (angle, radius) = self.selection();
        let center = point(bounds.center(), angle, radius);
        let handle = Rectangle::new(
            Point::new(center.x - HANDLE / 2.0, center.y - HANDLE / 2.0),
            Size::new(HANDLE, HANDLE),
        );
        operation.focusable(
            None,
            handle,
            &mut tree.state.downcast_mut::<DialState>().focus,
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
        let state = tree.state.downcast_mut::<DialState>();
        let bounds = layout.bounds();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(p) = cursor.position()
                    && (p.x - bounds.center_x()).hypot(p.y - bounds.center_y()) <= SIZE / 2.0
                {
                    state.dragging = true;
                    state.focus.focused = true;
                    state.focus.visible = false;
                    shell.publish((self.on_change)(self.at(bounds, p), false));
                    shell.capture_event();
                    shell.request_redraw();
                } else if state.focus.focused {
                    state.focus.unfocus();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.dragging => {
                shell.publish((self.on_change)(self.at(bounds, *position), false));
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging => {
                state.dragging = false;
                let value = cursor.position().map_or(self.value, |p| self.at(bounds, p));
                shell.publish((self.on_change)(value, true));
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) if state.focus.focused => {
                let delta = match key {
                    Named::ArrowRight | Named::ArrowUp => Some(1),
                    Named::ArrowLeft | Named::ArrowDown => Some(-1),
                    Named::PageUp => Some(5),
                    Named::PageDown => Some(-5),
                    _ => None,
                };
                if let Some(delta) = delta {
                    state.focus.visible = true;
                    shell.publish((self.on_change)(self.step(delta), false));
                    shell.capture_event();
                    shell.request_redraw();
                } else if matches!(key, Named::Enter | Named::Space) {
                    shell.publish((self.on_change)(self.value, true));
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
        let state = tree.state.downcast_ref::<DialState>();
        let bounds = layout.bounds();
        let c = &theme.colors;
        let clip = bounds.intersection(viewport).unwrap_or(bounds);
        let center = bounds.center();
        surface::fill(
            renderer,
            bounds,
            Radius::from(SIZE / 2.0),
            c.surface_container_highest,
        );
        let (angle, radius) = self.selection();
        let handle_center = point(center, angle, radius);
        let draw_label = |renderer: &mut Renderer, label: &Label, at: Point, color| {
            let size = label.size();
            label.draw(
                renderer,
                Point::new(at.x - size.width / 2.0, at.y - size.height / 2.0),
                color,
                clip,
            );
        };
        for (labels, ring) in [(&state.outer, OUTER), (&state.inner, INNER)] {
            for (i, label) in labels.iter().enumerate() {
                draw_label(
                    renderer,
                    label,
                    point(center, i as f32 * 30.0, ring),
                    c.on_surface,
                );
            }
        }
        let mut frame = Frame::new(renderer, bounds.size());
        frame.stroke(
            &Path::line(
                Point::new(bounds.width / 2.0, bounds.height / 2.0),
                Point::new(handle_center.x - bounds.x, handle_center.y - bounds.y),
            ),
            Stroke::default().with_color(c.primary).with_width(TRACK),
        );
        let geometry = frame.into_geometry();
        iced::advanced::Renderer::with_translation(
            renderer,
            iced::Vector::new(bounds.x, bounds.y),
            |renderer| renderer.draw_geometry(geometry),
        );
        let dot = Rectangle::new(
            Point::new(center.x - CENTER_DOT / 2.0, center.y - CENTER_DOT / 2.0),
            Size::new(CENTER_DOT, CENTER_DOT),
        );
        surface::fill(renderer, dot, Radius::from(CENTER_DOT / 2.0), c.primary);
        renderer.with_layer(bounds, |renderer| {
            let handle = Rectangle::new(
                Point::new(
                    handle_center.x - HANDLE / 2.0,
                    handle_center.y - HANDLE / 2.0,
                ),
                Size::new(HANDLE, HANDLE),
            );
            surface::fill(renderer, handle, Radius::from(HANDLE / 2.0), c.primary);
            let selected =
                match self.value {
                    Value::Minute(m) if m % 5 == 0 => state.outer.get(usize::from(m / 5)),
                    Value::Minute(_) => None,
                    Value::Hour12(h) => state.outer.get(usize::from(h % 12)),
                    Value::Hour24(h) if h == 0 || h >= 13 => state
                        .inner
                        .get(usize::from(if h == 0 { 0 } else { h - 12 })),
                    Value::Hour24(h) => state.outer.get(usize::from(h % 12)),
                };
            if let Some(label) = selected {
                draw_label(renderer, label, handle_center, c.on_primary);
            } else {
                let dot = Rectangle::new(
                    Point::new(
                        handle_center.x - CENTER_DOT / 2.0,
                        handle_center.y - CENTER_DOT / 2.0,
                    ),
                    Size::new(CENTER_DOT, CENTER_DOT),
                );
                surface::fill(renderer, dot, Radius::from(CENTER_DOT / 2.0), c.on_primary);
            }
            if state.focus.visible {
                focus_ring::draw(
                    renderer,
                    handle,
                    Radius::from(HANDLE / 2.0),
                    theme.focus_ring.outward_offset,
                    theme.focus_ring.width,
                    c.secondary,
                );
            }
        });
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let b = layout.bounds();
        match cursor.position() {
            Some(p) if (p.x - b.center_x()).hypot(p.y - b.center_y()) <= SIZE / 2.0 => {
                mouse::Interaction::Pointer
            }
            _ => mouse::Interaction::default(),
        }
    }
}

impl<'a, Message: Clone + 'a> From<Dial<'a, Message>> for Element<'a, Message> {
    fn from(dial: Dial<'a, Message>) -> Self {
        IcedElement::new(dial)
    }
}
