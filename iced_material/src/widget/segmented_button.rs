// SPDX-License-Identifier: LGPL-3.0-only

//! Outlined segmented buttons for single and multi select.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::keyboard::{self, key::Named};
use iced::widget::svg;
use iced::{
    Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse, window,
};

use crate::Element;
use crate::draw::text::Label;
use crate::icon::{symbol, tinted};
use crate::interaction::Tokens;
use crate::motion::{Transition, Tween};
use crate::shape::{Shape, ShapeScale};
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::{self, Status, Style};

/// Segmented button dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Container height.
    pub height: f32,
    /// Start and end padding of a segment.
    pub padding: f32,
    /// Icon and checkmark size.
    pub icon_size: f32,
    /// Space between the icon or checkmark and the label.
    pub icon_label_space: f32,
    /// Outline width. Adjacent segments overlap by this width.
    pub outline_width: f32,
    /// Minimum height of the pointer target.
    pub target_height: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            height: 40.0,
            padding: 12.0,
            icon_size: 18.0,
            icon_label_space: 8.0,
            outline_width: 1.0,
            target_height: 48.0,
        }
    }
}

/// The appearance catalog of segmented buttons.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a segment in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, bool) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(outlined_style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style {
        class(self, status, selected)
    }
}

/// Outlined segmented button style.
pub fn outlined_style(theme: &Theme, status: Status, selected: bool) -> Style {
    let c = &theme.colors;
    let container = if selected {
        c.secondary_container
    } else {
        Color::TRANSPARENT
    };
    if status == Status::Disabled {
        let content = alpha(c.on_surface, theme.disabled.content);
        return Style {
            container,
            outline: alpha(c.on_surface, theme.disabled.outline),
            outline_width: 1.0,
            ..Style::content(theme, content, content, c.on_surface)
        };
    }
    let content = if selected {
        c.on_secondary_container
    } else {
        c.on_surface
    };
    Style {
        container,
        outline: c.outline,
        outline_width: 1.0,
        ..Style::content(theme, content, content, content)
    }
}

/// One segment of a segmented button.
pub struct Segment<Message> {
    label: Option<String>,
    icon: Option<svg::Handle>,
    selected: bool,
    on_press: Option<Message>,
}

impl<Message> Segment<Message> {
    /// A segment with a label.
    pub fn new(label: impl Into<String>) -> Self {
        Segment {
            label: Some(label.into()),
            icon: None,
            selected: false,
            on_press: None,
        }
    }

    /// A segment with only an icon.
    pub fn icon_only(icon: svg::Handle) -> Self {
        Segment {
            label: None,
            icon: Some(icon),
            selected: false,
            on_press: None,
        }
    }

    /// Adds an icon shown while the segment is not selected.
    pub fn icon(mut self, icon: svg::Handle) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the message produced on press. Without one the segment is disabled.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message produced on press, or disables the segment with `None`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }
}

/// A Material outlined segmented button.
///
/// Selection is owned by the application: single select sets `selected` on
/// one segment, multi select on any number of them.
pub struct SegmentedButton<'a, Message> {
    segments: Vec<Segment<Message>>,
    width: Length,
    metrics: Metrics,
    tokens: Tokens,
    selection: Transition,
    type_style: TypeStyle,
    shape: ShapeScale,
    class: StyleFn<'a>,
}

/// A segmented button with the given segments.
pub fn segmented_button<'a, Message>(
    theme: &Theme,
    segments: impl IntoIterator<Item = Segment<Message>>,
) -> SegmentedButton<'a, Message> {
    SegmentedButton {
        segments: segments.into_iter().collect(),
        width: Length::Shrink,
        metrics: theme.components.segmented_button,
        tokens: Tokens::new(theme),
        selection: Transition {
            duration: theme.motion.duration.short3,
            easing: theme.motion.easing.standard,
        },
        type_style: theme.typography.label_large,
        shape: theme.shape,
        class: Box::new(outlined_style),
    }
}

impl<'a, Message> SegmentedButton<'a, Message> {
    /// Sets the width. `Shrink` sizes every segment to the widest one.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, bool) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn shape(&self, index: usize) -> Shape {
        let full = self.shape.full.top_left;
        match (index, self.segments.len()) {
            (_, 1) => self.shape.full,
            (0, _) => Shape::start(full),
            (i, n) if i + 1 == n => Shape::end(full),
            _ => self.shape.none,
        }
    }
}

struct SegmentState {
    press: pressable::State,
    label: Label,
    selection: Tween,
    selected: bool,
}

#[derive(Default)]
struct State {
    segments: Vec<SegmentState>,
}

impl State {
    fn resize<Message>(&mut self, segments: &[Segment<Message>]) {
        self.segments.truncate(segments.len());
        for segment in &segments[self.segments.len()..] {
            let value = if segment.selected { 1.0 } else { 0.0 };
            self.segments.push(SegmentState {
                press: pressable::State::default(),
                label: Label::default(),
                selection: Tween::new(value),
                selected: segment.selected,
            });
        }
    }

    fn focused(&self) -> Option<usize> {
        self.segments
            .iter()
            .position(|s| s.press.interaction.focus.focused)
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for SegmentedButton<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let mut state = State::default();
        state.resize(&self.segments);
        tree::State::new(state)
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.metrics.height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        state.resize(&self.segments);
        let m = &self.metrics;
        let mut widest: f32 = 0.0;
        for (segment, s) in self.segments.iter().zip(&mut state.segments) {
            let content = match &segment.label {
                Some(label) => {
                    m.icon_size + m.icon_label_space + s.label.update(label, self.type_style).width
                }
                None => m.icon_size,
            };
            widest = widest.max(m.padding * 2.0 + content);
        }
        let n = self.segments.len() as f32;
        let overlap = m.outline_width * (n - 1.0).max(0.0);
        let size = limits.resolve(
            self.width,
            Length::Fixed(m.height),
            Size::new(widest * n - overlap, m.height),
        );
        layout::Node::new(size)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        for (i, (segment, s)) in self.segments.iter().zip(&mut state.segments).enumerate() {
            if segment.on_press.is_some() {
                let b = segment_bounds(bounds, i, self.segments.len(), self.metrics.outline_width);
                operation.focusable(None, b, &mut s.press.interaction.focus);
            }
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
        let state = tree.state.downcast_mut::<State>();
        state.resize(&self.segments);
        let bounds = layout.bounds();
        let n = self.segments.len();

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key @ (Named::ArrowLeft | Named::ArrowRight)),
            ..
        }) = event
            && let Some(current) = state.focused()
        {
            let step = if *key == Named::ArrowRight { 1 } else { n - 1 };
            let mut next = (current + step) % n;
            while next != current && self.segments[next].on_press.is_none() {
                next = (next + step) % n;
            }
            if next != current {
                state.segments[current].press.interaction.focus.focused = false;
                state.segments[current].press.interaction.focus.visible = false;
                let focus = &mut state.segments[next].press.interaction.focus;
                focus.focused = true;
                focus.visible = true;
                focus.since = None;
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        for (i, (segment, s)) in self.segments.iter().zip(&mut state.segments).enumerate() {
            if segment.selected != s.selected {
                s.selected = segment.selected;
                s.selection.go(
                    if s.selected { 1.0 } else { 0.0 },
                    self.selection,
                    s.press.now(),
                );
                shell.request_redraw();
            }
            if let Event::Window(window::Event::RedrawRequested(now)) = event
                && s.selection.tick(*now)
            {
                shell.request_redraw();
            }
            let b = segment_bounds(bounds, i, n, self.metrics.outline_width);
            let hit = pressable::hit_area(b, self.metrics.target_height);
            let enabled = segment.on_press.is_some();
            if s.press
                .update(event, hit, cursor, enabled, false, &self.tokens, shell)
                .is_some()
                && let Some(message) = &segment.on_press
            {
                shell.publish(message.clone());
            }
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
        let n = self.segments.len();
        let m = &self.metrics;
        for (i, (segment, s)) in self.segments.iter().zip(&state.segments).enumerate() {
            let b = segment_bounds(bounds, i, n, m.outline_width);
            let radius = self.shape(i).radius(b.size());
            let enabled = segment.on_press.is_some();
            let (style, elevation) = s.press.style(enabled, false, |status| {
                theme.style(&self.class, status, segment.selected)
            });
            pressable::draw_container(renderer, theme, b, radius, &style, elevation, &s.press);

            let t = s.selection.value(s.press.now());
            let clip = b.intersection(viewport).unwrap_or(b);
            let label = s.label.size();
            let graphic = match (&segment.label, &segment.icon) {
                (Some(_), Some(_)) => m.icon_size + m.icon_label_space,
                (Some(_), None) => (m.icon_size + m.icon_label_space) * t,
                (None, _) => m.icon_size,
            };
            let content = graphic
                + if segment.label.is_some() {
                    label.width
                } else {
                    0.0
                };
            let x = b.center_x() - content / 2.0;
            let icon_bounds = Rectangle::new(
                Point::new(x, b.center_y() - m.icon_size / 2.0),
                Size::new(m.icon_size, m.icon_size),
            );
            let graphic_clip = Rectangle::new(
                Point::new(x, b.y),
                Size::new(graphic.min(m.icon_size), b.height),
            )
            .intersection(&clip)
            .unwrap_or(clip);
            if let Some(icon) = &segment.icon {
                renderer.draw_svg(
                    tinted(icon.clone(), style.icon, 1.0 - t),
                    icon_bounds,
                    graphic_clip,
                );
            }
            if t > 0.0 {
                renderer.draw_svg(
                    tinted(symbol::check(false), style.icon, t),
                    icon_bounds,
                    graphic_clip,
                );
            }
            if segment.label.is_some() {
                s.label.draw(
                    renderer,
                    Point::new(x + graphic, b.center_y() - label.height / 2.0),
                    style.label,
                    clip,
                );
            }
        }
        for (i, s) in state.segments.iter().enumerate() {
            let b = segment_bounds(bounds, i, n, m.outline_width);
            let radius = self.shape(i).radius(b.size());
            pressable::draw_focus(renderer, theme, b, radius, &s.press, &self.tokens);
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
        let bounds = layout.bounds();
        let n = self.segments.len();
        for (i, segment) in self.segments.iter().enumerate() {
            let b = segment_bounds(bounds, i, n, self.metrics.outline_width);
            if segment.on_press.is_some()
                && cursor.is_over(pressable::hit_area(b, self.metrics.target_height))
            {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

fn segment_bounds(bounds: Rectangle, index: usize, count: usize, overlap: f32) -> Rectangle {
    let n = count as f32;
    let width = (bounds.width + overlap * (n - 1.0)) / n;
    Rectangle {
        x: bounds.x + index as f32 * (width - overlap),
        y: bounds.y,
        width,
        height: bounds.height,
    }
}

impl<'a, Message: Clone + 'a> From<SegmentedButton<'a, Message>> for Element<'a, Message> {
    fn from(button: SegmentedButton<'a, Message>) -> Self {
        IcedElement::new(button)
    }
}
