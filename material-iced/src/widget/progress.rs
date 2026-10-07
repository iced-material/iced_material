// SPDX-License-Identifier: LGPL-3.0-only

//! Linear and circular progress indicators, determinate and indeterminate.

use std::time::Duration;

use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::time::Instant;
use iced::widget::canvas::path::Arc;
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{
    Color, Element as IcedElement, Event, Length, Point, Radians, Rectangle, Renderer, Size, mouse,
    window,
};

use crate::Element;
use crate::draw::surface;
use crate::motion::{Easing, Transition, Tween};
use crate::theme::Theme;

/// Progress indicator dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of the linear track and indicator.
    pub linear_height: f32,
    /// Minimum width of the linear indicator.
    pub linear_min_width: f32,
    /// Outer size of the circular indicator including its padding.
    pub circular_size: f32,
    /// Space between the circular indicator and the edge of its box.
    pub circular_padding: f32,
    /// Stroke width of the circular indicator.
    pub circular_stroke: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            linear_height: 4.0,
            linear_min_width: 80.0,
            circular_size: 48.0,
            circular_padding: 4.0,
            circular_stroke: 4.0,
        }
    }
}

/// The appearance of a progress indicator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Color of the indicator.
    pub indicator: Color,
    /// Color of the track. The circular track is not drawn.
    pub track: Color,
}

/// The appearance catalog of progress indicators.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The baseline progress indicator style.
pub fn style(theme: &Theme) -> Style {
    Style {
        indicator: theme.colors.primary,
        track: theme.colors.surface_container_highest,
    }
}

const fn ease(a: f32, b: f32, c: f32, d: f32) -> Easing {
    Easing::CubicBezier(a, b, c, d)
}

struct Key {
    at: f32,
    value: f32,
    easing: Easing,
}

const fn key(at: f32, value: f32, easing: Easing) -> Key {
    Key { at, value, easing }
}

fn sample(keys: &[Key], x: f32) -> f32 {
    for pair in keys.windows(2) {
        let (from, to) = (&pair[0], &pair[1]);
        if x <= to.at {
            let span = to.at - from.at;
            if span <= 0.0 {
                return to.value;
            }
            let eased = from.easing.apply((x - from.at) / span);
            return from.value + (to.value - from.value) * eased;
        }
    }
    keys.last().map_or(0.0, |k| k.value)
}

const LINEAR: Easing = ease(0.0, 0.0, 1.0, 1.0);

const PRIMARY_TRANSLATE: [Key; 4] = [
    key(0.0, 0.0, LINEAR),
    key(0.2, 0.0, ease(0.5, 0.0, 0.701732, 0.495819)),
    key(0.5915, 83.6714, ease(0.302435, 0.381352, 0.55, 0.956352)),
    key(1.0, 200.611, LINEAR),
];

const PRIMARY_SCALE: [Key; 4] = [
    key(0.0, 0.08, LINEAR),
    key(0.3665, 0.08, ease(0.334731, 0.12482, 0.785844, 1.0)),
    key(0.6915, 0.661479, ease(0.06, 0.11, 0.6, 1.0)),
    key(1.0, 0.08, LINEAR),
];

const SECONDARY_TRANSLATE: [Key; 4] = [
    key(0.0, 0.0, ease(0.15, 0.0, 0.515058, 0.409685)),
    key(0.25, 37.6519, ease(0.31033, 0.284058, 0.8, 0.733712)),
    key(0.4835, 84.3862, ease(0.4, 0.627035, 0.6, 0.902026)),
    key(1.0, 160.278, LINEAR),
];

const SECONDARY_SCALE: [Key; 4] = [
    key(0.0, 0.08, ease(0.205028, 0.057051, 0.57661, 0.453971)),
    key(
        0.1915,
        0.457104,
        ease(0.152313, 0.196432, 0.648374, 1.00432),
    ),
    key(
        0.4415,
        0.72796,
        ease(0.257759, -0.003163, 0.211762, 1.38179),
    ),
    key(1.0, 0.08, LINEAR),
];

const LINEAR_PERIOD: f32 = 2000.0;
const ARC_PERIOD: f32 = 1333.0;
const CYCLE_PERIOD: f32 = ARC_PERIOD * 4.0;
const ROTATE_PERIOD: f32 = ARC_PERIOD * 360.0 / 306.0;
const CIRCULAR_EASING: Easing = ease(0.4, 0.0, 0.2, 1.0);

const EXPAND_ARC: [Key; 3] = [
    key(0.0, 265.0, CIRCULAR_EASING),
    key(0.5, 130.0, CIRCULAR_EASING),
    key(1.0, 265.0, LINEAR),
];

const SPIN: [Key; 9] = [
    key(0.0, 0.0, CIRCULAR_EASING),
    key(0.125, 135.0, CIRCULAR_EASING),
    key(0.25, 270.0, CIRCULAR_EASING),
    key(0.375, 405.0, CIRCULAR_EASING),
    key(0.5, 540.0, CIRCULAR_EASING),
    key(0.625, 675.0, CIRCULAR_EASING),
    key(0.75, 810.0, CIRCULAR_EASING),
    key(0.875, 945.0, CIRCULAR_EASING),
    key(1.0, 1080.0, LINEAR),
];

struct State {
    value: Tween,
    start: Option<Instant>,
    now: Option<Instant>,
}

impl State {
    fn new(progress: Option<f32>) -> State {
        State {
            value: Tween::new(progress.unwrap_or(0.0)),
            start: None,
            now: None,
        }
    }

    fn elapsed_ms(&self) -> f32 {
        match (self.start, self.now) {
            (Some(start), Some(now)) => now.saturating_duration_since(start).as_secs_f32() * 1000.0,
            _ => 0.0,
        }
    }

    fn update<Message>(
        &mut self,
        event: &Event,
        progress: Option<f32>,
        transition: Transition,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(value) = progress {
            self.value.go(value.clamp(0.0, 1.0), transition, self.now);
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            self.now = Some(*now);
            self.start.get_or_insert(*now);
            if progress.is_none() || self.value.tick(*now) {
                shell.request_redraw();
            }
        }
    }
}

/// A linear progress indicator.
pub struct Linear<'a> {
    progress: Option<f32>,
    width: Length,
    metrics: Metrics,
    transition: Transition,
    class: StyleFn<'a>,
}

/// A linear progress indicator. `None` shows the indeterminate animation.
pub fn linear<'a>(theme: &Theme, progress: Option<f32>) -> Linear<'a> {
    Linear {
        progress,
        width: Length::Fill,
        metrics: theme.components.progress,
        transition: Transition {
            duration: Duration::from_millis(250),
            easing: ease(0.4, 0.0, 0.6, 1.0),
        },
        class: Box::new(style),
    }
}

impl<'a> Linear<'a> {
    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Linear<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.progress))
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.metrics.linear_height))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let h = self.metrics.linear_height;
        layout::Node::new(limits.resolve(
            self.width,
            Length::Fixed(h),
            Size::new(self.metrics.linear_min_width, h),
        ))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        tree.state
            .downcast_mut::<State>()
            .update(event, self.progress, self.transition, shell);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let look = theme.style(&self.class);
        let bounds = layout.bounds();
        let w = bounds.width;
        let bar = |from: f32, to: f32, color: Color, renderer: &mut Renderer| {
            let (from, to) = (from.max(0.0), to.min(w));
            if to > from {
                surface::fill(
                    renderer,
                    Rectangle::new(
                        Point::new(bounds.x + from, bounds.y),
                        Size::new(to - from, bounds.height),
                    ),
                    Radius::default(),
                    color,
                );
            }
        };
        bar(0.0, w, look.track, renderer);
        match self.progress {
            Some(_) => bar(
                0.0,
                w * state.value.value(state.now),
                look.indicator,
                renderer,
            ),
            None => {
                let p = (state.elapsed_ms() % LINEAR_PERIOD) / LINEAR_PERIOD;
                for (offset, translate, scale) in [
                    (-1.45167, &PRIMARY_TRANSLATE, &PRIMARY_SCALE),
                    (-0.548889, &SECONDARY_TRANSLATE, &SECONDARY_SCALE),
                ] {
                    let from = (offset + sample(translate, p) / 100.0) * w;
                    bar(from, from + sample(scale, p) * w, look.indicator, renderer);
                }
            }
        }
    }
}

impl<'a, Message: 'a> From<Linear<'a>> for Element<'a, Message> {
    fn from(indicator: Linear<'a>) -> Self {
        IcedElement::new(indicator)
    }
}

/// A circular progress indicator.
pub struct Circular<'a> {
    progress: Option<f32>,
    metrics: Metrics,
    transition: Transition,
    class: StyleFn<'a>,
}

/// A circular progress indicator. `None` shows the indeterminate animation.
pub fn circular<'a>(theme: &Theme, progress: Option<f32>) -> Circular<'a> {
    Circular {
        progress,
        metrics: theme.components.progress,
        transition: Transition {
            duration: Duration::from_millis(500),
            easing: ease(0.0, 0.0, 0.2, 1.0),
        },
        class: Box::new(style),
    }
}

impl<'a> Circular<'a> {
    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

fn clip_to_half(start: f32, left: bool) -> [Option<(f32, f32)>; 2] {
    let start = start.rem_euclid(360.0);
    let end = start + 180.0;
    let (low, high) = if left { (180.0, 360.0) } else { (0.0, 180.0) };
    let mut out = [None, None];
    let mut push = |slot: usize, a: f32, b: f32| {
        let (a, b) = (a.max(low), b.min(high));
        if b > a {
            out[slot] = Some((a, b));
        }
    };
    push(0, start, end.min(360.0));
    if end > 360.0 {
        push(1, 0.0, end - 360.0);
    }
    out
}

impl<Message> Widget<Message, Theme, Renderer> for Circular<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.progress))
    }

    fn size(&self) -> Size<Length> {
        let s = Length::Fixed(self.metrics.circular_size);
        Size::new(s, s)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let s = self.metrics.circular_size;
        layout::Node::new(Size::new(s, s))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        tree.state
            .downcast_mut::<State>()
            .update(event, self.progress, self.transition, shell);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let look = theme.style(&self.class);
        let bounds = layout.bounds();
        let m = &self.metrics;
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius =
            (m.circular_size / 2.0 - m.circular_padding - m.circular_stroke / 2.0).max(0.0);
        let stroke = Stroke::default()
            .with_color(look.indicator)
            .with_width(m.circular_stroke);
        let mut frame = Frame::new(renderer, bounds.size());
        let mut arc = |from: f32, to: f32| {
            let top = -std::f32::consts::FRAC_PI_2;
            frame.stroke(
                &Path::new(|b| {
                    b.arc(Arc {
                        center,
                        radius,
                        start_angle: Radians(top + from.to_radians()),
                        end_angle: Radians(top + to.to_radians()),
                    });
                }),
                stroke,
            );
        };
        match self.progress {
            Some(_) => {
                let sweep = state.value.value(state.now).clamp(0.0, 1.0) * 360.0;
                if sweep > 0.0 {
                    arc(0.0, sweep.min(359.99));
                }
            }
            None => {
                let t = state.elapsed_ms();
                let spin = sample(&SPIN, (t % CYCLE_PERIOD) / CYCLE_PERIOD)
                    + 360.0 * (t % ROTATE_PERIOD) / ROTATE_PERIOD;
                for (base, phase, left) in [(135.0, 0.0, true), (100.0, 0.5, false)] {
                    let p = ((t / ARC_PERIOD) + phase).rem_euclid(1.0);
                    let rotation = base + sample(&EXPAND_ARC, p);
                    for (from, to) in clip_to_half(rotation - 45.0, left).into_iter().flatten() {
                        arc(from + spin, to + spin);
                    }
                }
            }
        }
        let geometry = frame.into_geometry();
        iced::advanced::Renderer::with_translation(
            renderer,
            iced::Vector::new(bounds.x, bounds.y),
            |renderer| renderer.draw_geometry(geometry),
        );
    }
}

impl<'a, Message: 'a> From<Circular<'a>> for Element<'a, Message> {
    fn from(indicator: Circular<'a>) -> Self {
        IcedElement::new(indicator)
    }
}
