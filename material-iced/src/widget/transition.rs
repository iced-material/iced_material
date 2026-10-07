// SPDX-License-Identifier: LGPL-3.0-only

//! The fade, fade through, shared axis and container transform motion patterns.
//!
//! Iced cannot change the opacity of a widget. A fade draws the content and
//! then a layer of `background` over it with the opposite alpha. That equals
//! a real fade only over a plain background of that color.

use std::time::Duration;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay, renderer};
use iced::time::Instant;
use iced::{
    Color, Element as IcedElement, Event, Length, Rectangle, Renderer, Size, Transformation,
    Vector, mouse, window,
};

use crate::Element;
use crate::draw::surface;
use crate::motion::Easing;
use crate::shape::Shape;
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::layer::{Layered, Mode, Placement};

/// Axis of a shared axis transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// The content slides sideways.
    X,
    /// The content slides up or down.
    Y,
    /// The content scales.
    Z,
}

/// Motion tokens of the patterns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Share of a fade through and shared axis transition where the old content fades out.
    pub fade_threshold: f32,
    /// Scale the new content of a fade through grows from.
    pub fade_through_scale: f32,
    /// Distance of the slide of a shared axis transition on the X and Y axis.
    pub slide_distance: f32,
    /// Scale the new content of a forward Z transition grows from.
    pub z_incoming: f32,
    /// Scale the old content of a forward Z transition grows to.
    pub z_outgoing: f32,
    /// Share of the fade in of a fade where the content is fully visible.
    pub fade_in_end: f32,
    /// Scale the content of a fade grows from.
    pub fade_scale: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            fade_threshold: 0.35,
            fade_through_scale: 0.92,
            slide_distance: 30.0,
            z_incoming: 0.8,
            z_outgoing: 1.1,
            fade_in_end: 0.3,
            fade_scale: 0.8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pattern {
    FadeThrough,
    SharedAxis { axis: Axis, forward: bool },
}

/// Content that replaces other content with a motion pattern.
pub struct Transition<'a, Message> {
    current: Element<'a, Message>,
    previous: Option<Element<'a, Message>>,
    pattern: Pattern,
    background: Color,
    on_finished: Option<Message>,
    metrics: Metrics,
    duration: Duration,
    easing: Easing,
}

fn new<'a, Message>(
    theme: &Theme,
    pattern: Pattern,
    background: Color,
    current: Element<'a, Message>,
    previous: Option<Element<'a, Message>>,
) -> Transition<'a, Message> {
    Transition {
        current,
        previous,
        pattern,
        background,
        on_finished: None,
        metrics: theme.components.transition,
        duration: theme.motion.duration.long1,
        easing: theme.motion.easing.emphasized,
    }
}

/// A fade through from `previous` to `current`. The old content fades out, then the new content fades in and scales up.
///
/// While `previous` is some, the widget runs the animation and then publishes
/// the message of [`Transition::on_finished`]. The application then drops
/// `previous`.
pub fn fade_through<'a, Message>(
    theme: &Theme,
    background: Color,
    current: impl Into<Element<'a, Message>>,
    previous: Option<Element<'a, Message>>,
) -> Transition<'a, Message> {
    new(
        theme,
        Pattern::FadeThrough,
        background,
        current.into(),
        previous,
    )
}

/// A shared axis transition between related content. `forward` moves along the axis, otherwise back.
pub fn shared_axis<'a, Message>(
    theme: &Theme,
    background: Color,
    axis: Axis,
    forward: bool,
    current: impl Into<Element<'a, Message>>,
    previous: Option<Element<'a, Message>>,
) -> Transition<'a, Message> {
    new(
        theme,
        Pattern::SharedAxis { axis, forward },
        background,
        current.into(),
        previous,
    )
}

impl<'a, Message> Transition<'a, Message> {
    /// Sets the message published once when the animation ended.
    pub fn on_finished(mut self, message: Message) -> Self {
        self.on_finished = Some(message);
        self
    }

    /// Sets the duration of the animation.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

#[derive(Default)]
struct State {
    start: Option<Instant>,
    now: Option<Instant>,
    finished: bool,
    was_animating: bool,
}

impl State {
    fn progress(&self, duration: Duration) -> f32 {
        match (self.start, self.now) {
            (Some(start), Some(now)) => (now.saturating_duration_since(start).as_secs_f32()
                / duration.as_secs_f32())
            .clamp(0.0, 1.0),
            _ => 0.0,
        }
    }
}

fn ramp(value: f32, from: f32, to: f32) -> f32 {
    ((value - from) / (to - from)).clamp(0.0, 1.0)
}

fn scaled(bounds: Rectangle, scale: f32, offset: Vector) -> Transformation {
    let c = bounds.center();
    Transformation::translate(c.x + offset.x, c.y + offset.y)
        * Transformation::scale(scale)
        * Transformation::translate(-c.x, -c.y)
}

impl<Message> Transition<'_, Message> {
    fn frames(&self, e: f32) -> [(f32, f32, Vector); 2] {
        let m = &self.metrics;
        let out = 1.0 - ramp(e, 0.0, m.fade_threshold);
        let incoming = ramp(e, m.fade_threshold, 1.0);
        match self.pattern {
            Pattern::FadeThrough => [
                (out, 1.0, Vector::ZERO),
                (
                    incoming,
                    m.fade_through_scale + (1.0 - m.fade_through_scale) * e,
                    Vector::ZERO,
                ),
            ],
            Pattern::SharedAxis { axis, forward } => {
                let sign = if forward { 1.0 } else { -1.0 };
                let d = m.slide_distance;
                match axis {
                    Axis::X => [
                        (out, 1.0, Vector::new(-sign * d * e, 0.0)),
                        (incoming, 1.0, Vector::new(sign * d * (1.0 - e), 0.0)),
                    ],
                    Axis::Y => [
                        (out, 1.0, Vector::new(0.0, -sign * d * e)),
                        (incoming, 1.0, Vector::new(0.0, sign * d * (1.0 - e))),
                    ],
                    Axis::Z => {
                        let (out_end, in_start) = if forward {
                            (m.z_outgoing, m.z_incoming)
                        } else {
                            (m.z_incoming, m.z_outgoing)
                        };
                        [
                            (out, 1.0 + (out_end - 1.0) * e, Vector::ZERO),
                            (incoming, in_start + (1.0 - in_start) * e, Vector::ZERO),
                        ]
                    }
                }
            }
        }
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Transition<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(&self.current)
            .chain(self.previous.as_ref())
            .map(Tree::new)
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements: Vec<&Element<'_, Message>> = std::iter::once(&self.current)
            .chain(self.previous.as_ref())
            .collect();
        tree.diff_children_custom(
            &elements,
            |tree, element| tree.diff(element.as_widget()),
            |element| Tree::new(*element),
        );
    }

    fn size(&self) -> Size<Length> {
        self.current.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let current = self
            .current
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = current.size();
        let mut children = vec![current];
        if let Some(previous) = &mut self.previous {
            let fixed = layout::Limits::new(size, size);
            children.push(
                previous
                    .as_widget_mut()
                    .layout(&mut tree.children[1], renderer, &fixed),
            );
        }
        layout::Node::with_children(size, children)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.previous.is_none()
            && let Some(child) = layout.children().next()
        {
            self.current
                .as_widget_mut()
                .operate(&mut tree.children[0], child, renderer, operation);
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
        let (state_tree, children) = (&mut tree.state, &mut tree.children);
        let state = state_tree.downcast_mut::<State>();
        let animating = self.previous.is_some();
        if !animating {
            *state = State::default();
        } else if !state.was_animating {
            *state = State {
                was_animating: true,
                ..State::default()
            };
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if animating {
                state.start.get_or_insert(*now);
            }
        }
        if animating {
            if state.progress(self.duration) < 1.0 {
                shell.request_redraw();
            } else if !state.finished {
                state.finished = true;
                if let Some(message) = &self.on_finished {
                    shell.publish(message.clone());
                }
            }
        }
        let Some(child) = layout.children().next() else {
            return;
        };
        let passes =
            !animating || state.progress(self.duration) >= 1.0 || matches!(event, Event::Window(_));
        if passes {
            self.current.as_widget_mut().update(
                &mut children[0],
                event,
                child,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let mut children = layout.children();
        let (Some(current), previous) = (children.next(), children.next()) else {
            return;
        };
        let progress = state.progress(self.duration);
        let animating = self.previous.is_some() && progress < 1.0;
        if !animating {
            self.current.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                current,
                cursor,
                viewport,
            );
            return;
        }
        let e = self.easing.apply(progress);
        let [out, incoming] = self.frames(e);
        let draw = |renderer: &mut Renderer,
                    frame: (f32, f32, Vector),
                    element: &Element<'a, Message>,
                    tree: &Tree,
                    layout: Layout<'_>| {
            let (opacity, scale, offset) = frame;
            if opacity <= 0.0 {
                return;
            }
            renderer.with_layer(bounds, |renderer| {
                renderer.with_transformation(scaled(bounds, scale, offset), |renderer| {
                    element.as_widget().draw(
                        tree,
                        renderer,
                        theme,
                        style,
                        layout,
                        mouse::Cursor::Unavailable,
                        &bounds,
                    );
                });
            });
            renderer.with_layer(bounds, |renderer| {
                surface::fill(
                    renderer,
                    bounds,
                    Default::default(),
                    alpha(self.background, 1.0 - opacity),
                );
            });
        };
        if let (Some(previous), Some(element)) = (previous, &self.previous) {
            draw(renderer, out, element, &tree.children[1], previous);
        }
        draw(
            renderer,
            incoming,
            &self.current,
            &tree.children[0],
            current,
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
        match (self.previous.is_some(), layout.children().next()) {
            (false, Some(child)) => self.current.as_widget().mouse_interaction(
                &tree.children[0],
                child,
                cursor,
                viewport,
                renderer,
            ),
            _ => mouse::Interaction::default(),
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
        if self.previous.is_some() {
            return None;
        }
        self.current.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<Transition<'a, Message>> for Element<'a, Message> {
    fn from(transition: Transition<'a, Message>) -> Self {
        IcedElement::new(transition)
    }
}

/// A container transform: `detail` grows out of the bounds of `anchor` to fill the window.
pub struct ContainerTransform<'a, Message> {
    anchor: Element<'a, Message>,
    detail: Element<'a, Message>,
    open: bool,
    shape: Shape,
    color: Color,
    on_dismiss: Option<Message>,
}

/// A container transform between `anchor` and `detail`.
pub fn container_transform<'a, Message>(
    anchor: impl Into<Element<'a, Message>>,
    detail: impl Into<Element<'a, Message>>,
    open: bool,
) -> ContainerTransform<'a, Message> {
    ContainerTransform {
        anchor: anchor.into(),
        detail: detail.into(),
        open,
        shape: Shape::all(crate::shape::Corner::Fixed(12.0)),
        color: Color::TRANSPARENT,
        on_dismiss: None,
    }
}

impl<'a, Message> ContainerTransform<'a, Message> {
    /// Sets the shape of the container at the start of the transform.
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the color of the container, which is also the color the detail fades in from.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the message produced on Escape.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }
}

impl<'a, Message: Clone + 'a> ContainerTransform<'a, Message> {
    /// Builds the anchor with its detail layer.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        Layered::new(
            theme,
            self.anchor,
            self.detail,
            Mode::Modal {
                open: self.open,
                on_dismiss: self.on_dismiss,
                dismiss_on_scrim: false,
            },
            Placement::Expand(self.shape, self.color),
        )
        .into()
    }
}

/// Content that fades and scales in when `visible` turns true and fades out when it turns false.
pub struct Fade<'a, Message> {
    content: Element<'a, Message>,
    visible: bool,
    background: Color,
    metrics: Metrics,
    incoming: (Duration, Easing),
    outgoing: Duration,
}

/// A fade of `content` over `background`.
pub fn fade<'a, Message>(
    theme: &Theme,
    background: Color,
    content: impl Into<Element<'a, Message>>,
    visible: bool,
) -> Fade<'a, Message> {
    Fade {
        content: content.into(),
        visible,
        background,
        metrics: theme.components.transition,
        incoming: (
            theme.motion.duration.medium4,
            theme.motion.easing.emphasized,
        ),
        outgoing: theme.motion.duration.short3,
    }
}

#[derive(Default)]
struct FadeState {
    shown: bool,
    start: Option<Instant>,
    now: Option<Instant>,
    entering: bool,
    animating: bool,
}

impl<'a, Message> Fade<'a, Message> {
    fn frame(&self, state: &FadeState) -> Option<(f32, f32)> {
        let elapsed = match (state.start, state.now) {
            (Some(start), Some(now)) => now.saturating_duration_since(start).as_secs_f32(),
            _ => 0.0,
        };
        if !state.animating {
            return state.shown.then_some((1.0, 1.0));
        }
        if state.entering {
            let p = (elapsed / self.incoming.0.as_secs_f32()).clamp(0.0, 1.0);
            let e = self.incoming.1.apply(p);
            let m = &self.metrics;
            Some((
                ramp(e, 0.0, m.fade_in_end),
                m.fade_scale + (1.0 - m.fade_scale) * e,
            ))
        } else {
            let p = (elapsed / self.outgoing.as_secs_f32()).clamp(0.0, 1.0);
            Some((1.0 - p, 1.0))
        }
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Fade<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FadeState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(FadeState {
            shown: self.visible,
            ..FadeState::default()
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
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
        if self.visible {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
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
        let (state_tree, children) = (&mut tree.state, &mut tree.children);
        let state = state_tree.downcast_mut::<FadeState>();
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if state.animating {
                state.start.get_or_insert(*now);
            }
        }
        if self.visible != state.shown && !(state.animating && state.entering == self.visible) {
            state.entering = self.visible;
            state.animating = true;
            state.start = None;
            shell.request_redraw();
        }
        if state.animating {
            let limit = if state.entering {
                self.incoming.0
            } else {
                self.outgoing
            };
            let done = match (state.start, state.now) {
                (Some(start), Some(now)) => now.saturating_duration_since(start) >= limit,
                _ => false,
            };
            if done {
                state.animating = false;
                state.shown = state.entering;
            } else {
                shell.request_redraw();
            }
        }
        if self.visible && !state.animating {
            self.content.as_widget_mut().update(
                &mut children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        let state = tree.state.downcast_ref::<FadeState>();
        let Some((opacity, scale)) = self.frame(state) else {
            return;
        };
        if opacity <= 0.0 {
            return;
        }
        let bounds = layout.bounds();
        if !state.animating {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
            return;
        }
        renderer.with_layer(bounds, |renderer| {
            renderer.with_transformation(scaled(bounds, scale, Vector::ZERO), |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    mouse::Cursor::Unavailable,
                    &bounds,
                );
            });
        });
        renderer.with_layer(bounds, |renderer| {
            surface::fill(
                renderer,
                bounds,
                Default::default(),
                alpha(self.background, 1.0 - opacity),
            );
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.visible {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<Fade<'a, Message>> for Element<'a, Message> {
    fn from(fade: Fade<'a, Message>) -> Self {
        IcedElement::new(fade)
    }
}
