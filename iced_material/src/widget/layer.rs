// SPDX-License-Identifier: LGPL-3.0-only

use std::time::Duration;

use iced::advanced::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay::{self, Overlay};
use iced::advanced::widget::{Operation, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::keyboard::{self, key::Named};
use iced::mouse::{self, Cursor};
use iced::time::Instant;
use iced::{
    Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, Vector, window,
};

use crate::Element;
use crate::draw::surface;
use crate::motion::Easing;
use crate::shape::Shape;
use crate::theme::Theme;
use crate::widget::probe::Probe;

pub(crate) enum Mode<Message> {
    Tooltip {
        delay: Duration,
    },
    Popup {
        open: bool,
        on_dismiss: Option<Message>,
    },
    Modal {
        open: bool,
        on_dismiss: Option<Message>,
        dismiss_on_scrim: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Edge {
    Start,
    End,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Placement {
    Edge(Edge),
    Expand(Shape, Color),
    Above(f32),
    BelowStart(f32),
    Side(f32),
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Phase {
    #[default]
    Closed,
    Opening,
    Open,
    Closing,
}

#[derive(Default)]
pub(crate) struct LayerState {
    phase: Phase,
    since: Option<Instant>,
    hover_since: Option<Instant>,
    shown: bool,
    layer_hovered: bool,
    now: Option<Instant>,
    focus_pending: bool,
}

pub(crate) struct Style {
    pub scrim: Color,
    pub scrim_opacity: f32,
    pub open_easing: Easing,
    pub close_easing: Easing,
    pub slide: bool,
    pub expand: Option<(Shape, Color)>,
    pub open: Duration,
    pub close: Duration,
    pub reveal_from: f32,
    pub drop: f32,
    pub window_margin: f32,
}

impl Style {
    pub fn slide(theme: &Theme) -> Style {
        Style {
            open_easing: theme.motion.easing.emphasized_decelerate,
            slide: true,
            open: theme.motion.duration.medium1,
            close: theme.motion.duration.short4,
            ..Style::new(theme)
        }
    }

    pub fn expand(theme: &Theme, shape: Shape, color: Color) -> Style {
        Style {
            scrim_opacity: 0.0,
            close_easing: theme.motion.easing.emphasized,
            expand: Some((shape, color)),
            open: theme.motion.duration.long1,
            close: theme.motion.duration.medium2,
            ..Style::new(theme)
        }
    }

    pub fn new(theme: &Theme) -> Style {
        Style {
            scrim: theme.colors.scrim,
            scrim_opacity: 0.32,
            open_easing: theme.motion.easing.emphasized,
            close_easing: theme.motion.easing.emphasized_accelerate,
            slide: false,
            expand: None,
            open: Duration::from_millis(500),
            close: theme.motion.duration.short3,
            reveal_from: 0.35,
            drop: 50.0,
            window_margin: 24.0,
        }
    }
}

pub(crate) struct Layered<'a, Message> {
    anchor: Element<'a, Message>,
    layer: Element<'a, Message>,
    mode: Mode<Message>,
    placement: Placement,
    match_anchor_width: bool,
    style: Style,
}

impl<'a, Message> Layered<'a, Message> {
    pub fn new(
        theme: &Theme,
        anchor: Element<'a, Message>,
        layer: Element<'a, Message>,
        mode: Mode<Message>,
        placement: Placement,
    ) -> Self {
        Layered {
            anchor,
            layer,
            mode,
            placement,
            match_anchor_width: false,
            style: match placement {
                Placement::Edge(_) => Style::slide(theme),
                Placement::Expand(shape, color) => Style::expand(theme, shape, color),
                _ => Style::new(theme),
            },
        }
    }

    pub fn match_anchor_width(mut self, value: bool) -> Self {
        self.match_anchor_width = value;
        self
    }
}

impl LayerState {
    pub fn request_focus(&mut self) {
        self.focus_pending = true;
    }

    fn progress(&self, style: &Style) -> (f32, f32) {
        let elapsed = |since: Option<Instant>| {
            match (since, self.now) {
                (Some(since), Some(now)) => now.saturating_duration_since(since),
                _ => Duration::ZERO,
            }
            .as_secs_f32()
        };
        match self.phase {
            Phase::Closed => (0.0, 0.0),
            Phase::Open => (1.0, 1.0),
            Phase::Opening => {
                let p = (elapsed(self.since) / style.open.as_secs_f32()).clamp(0.0, 1.0);
                (style.open_easing.apply(p), p)
            }
            Phase::Closing => {
                let p = (elapsed(self.since) / style.close.as_secs_f32()).clamp(0.0, 1.0);
                (1.0 - style.close_easing.apply(p), 1.0 - p)
            }
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Layered<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LayerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(LayerState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.layer)]
    }

    fn diff(&self, tree: &mut Tree) {
        let elements = [&self.anchor, &self.layer];
        tree.diff_children_custom(
            &elements,
            |tree, element| tree.diff(element.as_widget()),
            |element| Tree::new(*element),
        );
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
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
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        let state = tree.state.downcast_mut::<LayerState>();
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
        }
        let now = state.now.unwrap_or_else(Instant::now);
        match &self.mode {
            Mode::Tooltip { delay } => {
                let hovering = cursor.is_over(layout.bounds());
                if hovering || state.layer_hovered {
                    if hovering && state.hover_since.is_none() {
                        state.hover_since = Some(now);
                    }
                    if !state.shown
                        && state
                            .hover_since
                            .is_some_and(|since| now.saturating_duration_since(since) >= *delay)
                    {
                        state.shown = true;
                        shell.request_redraw();
                    } else if !state.shown {
                        shell.request_redraw();
                    }
                } else if state.shown || state.hover_since.is_some() {
                    state.shown = false;
                    state.hover_since = None;
                    shell.request_redraw();
                }
            }
            Mode::Popup { open, .. } => {
                if *open != state.shown {
                    state.shown = *open;
                    state.focus_pending = *open;
                    shell.request_redraw();
                }
            }
            Mode::Modal { open, .. } => {
                match (*open, state.phase) {
                    (true, Phase::Closed | Phase::Closing) => {
                        state.phase = Phase::Opening;
                        state.since = Some(now);
                        state.focus_pending = true;
                        shell.request_redraw();
                    }
                    (false, Phase::Opening | Phase::Open) => {
                        state.phase = Phase::Closing;
                        state.since = Some(now);
                        shell.request_redraw();
                    }
                    _ => {}
                }
                let elapsed = state
                    .since
                    .map_or(Duration::ZERO, |since| now.saturating_duration_since(since));
                match state.phase {
                    Phase::Opening if elapsed >= self.style.open => state.phase = Phase::Open,
                    Phase::Closing if elapsed >= self.style.close => state.phase = Phase::Closed,
                    _ => {}
                }
                if matches!(state.phase, Phase::Opening | Phase::Closing) {
                    shell.request_redraw();
                }
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
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
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
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.anchor.as_widget().mouse_interaction(
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
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<LayerState>();
        let (anchor_tree, layer_tree) = children.split_at_mut(1);
        let open = matches!(
            self.mode,
            Mode::Popup { open: true, .. } | Mode::Modal { open: true, .. }
        );
        let anchor_overlay = self.anchor.as_widget_mut().overlay(
            &mut anchor_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );
        let visible = match &self.mode {
            Mode::Tooltip { .. } => state.shown,
            Mode::Popup { .. } => open,
            Mode::Modal { .. } => state.phase != Phase::Closed,
        };
        let own = visible.then(|| {
            let (dismiss, modal, on_scrim) = match &self.mode {
                Mode::Tooltip { .. } => (None, false, false),
                Mode::Popup { on_dismiss, .. } => (on_dismiss.clone(), false, false),
                Mode::Modal {
                    on_dismiss,
                    dismiss_on_scrim,
                    ..
                } => (on_dismiss.clone(), true, *dismiss_on_scrim),
            };
            overlay::Element::new(Box::new(LayerOverlay {
                layer: &mut self.layer,
                tree: &mut layer_tree[0],
                state,
                anchor: Rectangle::new(layout.position() + translation, layout.bounds().size()),
                placement: self.placement,
                match_anchor_width: self.match_anchor_width,
                style: &self.style,
                dismiss,
                modal,
                dismiss_on_scrim: on_scrim,
                window: Size::ZERO,
                tooltip: matches!(self.mode, Mode::Tooltip { .. }),
                passive: false,
            }))
        });
        match (anchor_overlay, own) {
            (None, None) => None,
            (a, b) => {
                Some(overlay::Group::with_children(a.into_iter().chain(b).collect()).overlay())
            }
        }
    }
}

impl<'a, Message: Clone + 'a> From<Layered<'a, Message>> for Element<'a, Message> {
    fn from(layered: Layered<'a, Message>) -> Self {
        IcedElement::new(layered)
    }
}

struct LayerOverlay<'a, 'b, Message> {
    layer: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    state: &'b mut LayerState,
    anchor: Rectangle,
    placement: Placement,
    match_anchor_width: bool,
    style: &'b Style,
    dismiss: Option<Message>,
    modal: bool,
    dismiss_on_scrim: bool,
    window: Size,
    tooltip: bool,
    passive: bool,
}

fn place(placement: Placement, anchor: Rectangle, size: Size, window: Size) -> Point {
    let (mut x, mut y) = match placement {
        Placement::Edge(_) | Placement::Expand(..) => (0.0, 0.0),
        Placement::Above(gap) => {
            let y = anchor.y - gap - size.height;
            let y = if y < 0.0 {
                anchor.y + anchor.height + gap
            } else {
                y
            };
            (anchor.center_x() - size.width / 2.0, y)
        }
        Placement::BelowStart(gap) => {
            let y = anchor.y + anchor.height + gap;
            let y = if y + size.height > window.height && anchor.y - gap - size.height >= 0.0 {
                anchor.y - gap - size.height
            } else {
                y
            };
            (anchor.x, y)
        }
        Placement::Side(gap) => {
            let x = anchor.x + anchor.width + gap;
            let x = if x + size.width > window.width && anchor.x - gap - size.width >= 0.0 {
                anchor.x - gap - size.width
            } else {
                x
            };
            (x, anchor.y)
        }
        Placement::Center => (
            (window.width - size.width) / 2.0,
            (window.height - size.height) / 2.0,
        ),
    };
    x = x.min(window.width - size.width).max(0.0);
    y = y.min(window.height - size.height).max(0.0);
    Point::new(x, y)
}

pub(crate) fn passive_popup<'a, 'b, Message: Clone + 'a>(
    layer: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    state: &'b mut LayerState,
    style: &'b Style,
    anchor: Rectangle,
    placement: Placement,
) -> overlay::Element<'b, Message, Theme, Renderer> {
    overlay::Element::new(Box::new(LayerOverlay {
        layer,
        tree,
        state,
        anchor,
        placement,
        match_anchor_width: false,
        style,
        dismiss: None,
        modal: false,
        dismiss_on_scrim: false,
        window: Size::ZERO,
        tooltip: false,
        passive: true,
    }))
}

struct Chain<'a, 'b, Message> {
    layer: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
}

impl<Message> Chain<'_, '_, Message> {
    fn run(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.layer
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
        if let operation::Outcome::Chain(mut next) = operation.finish() {
            self.run(layout, renderer, next.as_mut());
        }
    }
}

impl<Message: Clone> Overlay<Message, Theme, Renderer> for LayerOverlay<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.window = bounds;
        let expand = self.style.expand.is_some();
        let margin = if self.modal && !expand {
            self.style.window_margin * 2.0
        } else {
            0.0
        };
        let max = Size::new(
            (bounds.width - margin).max(0.0),
            (bounds.height - margin).max(0.0),
        );
        let min = if expand {
            bounds
        } else if self.match_anchor_width {
            Size::new(self.anchor.width.min(max.width), 0.0)
        } else if matches!(self.placement, Placement::Edge(_)) {
            Size::new(0.0, bounds.height)
        } else {
            Size::ZERO
        };
        let node =
            self.layer
                .as_widget_mut()
                .layout(self.tree, renderer, &layout::Limits::new(min, max));
        let (reveal, _) = self.state.progress(self.style);
        let width = node.size().width;
        let at = match self.placement {
            Placement::Edge(Edge::Start) => Point::new(-width * (1.0 - reveal), 0.0),
            Placement::Edge(Edge::End) => {
                Point::new(bounds.width - width + width * (1.0 - reveal), 0.0)
            }
            Placement::Expand(..) => Point::new(
                self.anchor.x * (1.0 - reveal),
                self.anchor.y * (1.0 - reveal),
            ),
            placement => {
                let mut at = place(placement, self.anchor, node.size(), bounds);
                if self.modal {
                    at.y -= self.style.drop * (1.0 - reveal);
                }
                at
            }
        };
        layout::Node::with_children(bounds, vec![node.move_to(at)])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        let Some(content) = layout.children().next() else {
            return;
        };
        let (reveal, scrim) = self.state.progress(self.style);
        if self.modal {
            surface::fill(
                renderer,
                layout.bounds(),
                Default::default(),
                crate::state::alpha(self.style.scrim, self.style.scrim_opacity * scrim),
            );
        }
        let viewport = Rectangle::with_size(Size::INFINITE);
        if let Some((from, color)) = self.style.expand {
            let (_, linear) = self.state.progress(self.style);
            let window = layout.bounds();
            let mix = |a: f32, b: f32| a + (b - a) * reveal;
            let rect = Rectangle::new(
                Point::new(mix(self.anchor.x, window.x), mix(self.anchor.y, window.y)),
                Size::new(
                    mix(self.anchor.width, window.width),
                    mix(self.anchor.height, window.height),
                ),
            );
            let none = Shape::all(crate::shape::Corner::Fixed(0.0));
            let radius = from.lerp(&none, (linear / 0.75).clamp(0.0, 1.0), rect.size());
            surface::fill(renderer, rect, radius, color);
            renderer.with_layer(rect, |renderer| {
                self.layer.as_widget().draw(
                    self.tree, renderer, theme, style, content, cursor, &viewport,
                );
            });
            renderer.with_layer(rect, |renderer| {
                surface::fill(
                    renderer,
                    rect,
                    radius,
                    crate::state::alpha(color, 1.0 - (linear / 0.25).clamp(0.0, 1.0)),
                );
            });
            return;
        }
        let draw = |renderer: &mut Renderer| {
            self.layer.as_widget().draw(
                self.tree, renderer, theme, style, content, cursor, &viewport,
            );
        };
        if self.modal && !self.style.slide && reveal < 1.0 {
            let b = content.bounds();
            let fraction = self.style.reveal_from + (1.0 - self.style.reveal_from) * reveal;
            let clip = Rectangle::new(b.position(), Size::new(b.width + 32.0, b.height * fraction));
            renderer.with_layer(clip, draw);
        } else {
            draw(renderer);
        }
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if let Some(content) = layout.children().next() {
            self.layer
                .as_widget_mut()
                .operate(self.tree, content, renderer, operation);
        }
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(content) = layout.children().next() else {
            return;
        };
        let bounds = content.bounds();
        if self.tooltip {
            self.state.layer_hovered = cursor.is_over(bounds);
        }
        if self.state.focus_pending {
            self.state.focus_pending = false;
            let mut chain = Chain {
                layer: self.layer,
                tree: self.tree,
            };
            chain.run(
                content,
                renderer,
                &mut operation::focusable::focus_next::<()>(),
            );
            shell.request_redraw();
        }
        if !self.tooltip && !self.passive {
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Escape),
                    ..
                }) => {
                    if let Some(message) = &self.dismiss {
                        shell.publish(message.clone());
                    }
                    shell.capture_event();
                    return;
                }
                Event::Mouse(mouse::Event::ButtonPressed(_))
                    if !cursor.is_over(bounds) && cursor.position().is_some() =>
                {
                    if (!self.modal || self.dismiss_on_scrim)
                        && let Some(message) = &self.dismiss
                    {
                        shell.publish(message.clone());
                    }
                    shell.capture_event();
                    return;
                }
                _ => {}
            }
        }
        self.layer.as_widget_mut().update(
            self.tree,
            event,
            content,
            cursor,
            renderer,
            clipboard,
            shell,
            &Rectangle::with_size(Size::INFINITE),
        );
        if self.modal && !shell.is_event_captured() {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Tab),
                modifiers,
                ..
            }) = event
            {
                let mut chain = Chain {
                    layer: self.layer,
                    tree: self.tree,
                };
                for _ in 0..2 {
                    if modifiers.shift() {
                        chain.run(
                            content,
                            renderer,
                            &mut operation::focusable::focus_previous::<()>(),
                        );
                    } else {
                        chain.run(
                            content,
                            renderer,
                            &mut operation::focusable::focus_next::<()>(),
                        );
                    }
                    let mut probe = Probe(false);
                    chain.run(content, renderer, &mut probe);
                    if probe.0 {
                        break;
                    }
                }
                shell.request_redraw();
            }
            if matches!(
                event,
                Event::Mouse(_) | Event::Keyboard(_) | Event::Touch(_)
            ) {
                shell.capture_event();
            }
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(content) = layout.children().next() else {
            return mouse::Interaction::None;
        };
        let inner = self.layer.as_widget().mouse_interaction(
            self.tree,
            content,
            cursor,
            &Rectangle::with_size(Size::INFINITE),
            renderer,
        );
        if inner != mouse::Interaction::None {
            inner
        } else if self.modal || cursor.is_over(content.bounds()) {
            mouse::Interaction::Idle
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let content = layout.children().next()?;
        self.layer.as_widget_mut().overlay(
            self.tree,
            content,
            renderer,
            &Rectangle::with_size(Size::INFINITE),
            Vector::ZERO,
        )
    }
}

pub(crate) fn edge_modal<'a, Message: Clone + 'a>(
    theme: &Theme,
    base: Element<'a, Message>,
    sheet: Element<'a, Message>,
    open: bool,
    edge: Edge,
    on_dismiss: Option<Message>,
    dismiss_on_scrim: bool,
) -> Element<'a, Message> {
    Layered::new(
        theme,
        base,
        sheet,
        Mode::Modal {
            open,
            on_dismiss,
            dismiss_on_scrim,
        },
        Placement::Edge(edge),
    )
    .into()
}
