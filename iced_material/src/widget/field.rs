// SPDX-License-Identifier: LGPL-3.0-only

use iced::advanced::Renderer as _;
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay, renderer};
use iced::border::Radius;
use iced::mouse::{self, Cursor};
use iced::time::Instant;
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{
    Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, Vector, window,
};

use crate::Element;
use crate::draw::text::Label;
use crate::draw::{focus_ring, surface};
use crate::icon::tinted;
use crate::interaction::{Interaction, Tokens};
use crate::motion::{Easing, Transition, Tween};
use crate::state::alpha;
use crate::theme::Theme;
use crate::typography::TypeStyle;
use crate::widget::pressable::Status;
use crate::widget::probe::Probe;
use crate::widget::text_field::{Catalog as _, Kind, Metrics, StyleFn, Variant};

pub(crate) struct Parts {
    pub kind: Kind,
    pub label: Option<String>,
    pub placeholder: String,
    pub supporting: Option<String>,
    pub counter: Option<(usize, usize)>,
    pub error: bool,
    pub enabled: bool,
    pub populated: bool,
    pub active: bool,
    pub rows: usize,
    pub leading_icon: Option<iced::widget::svg::Handle>,
    pub trailing_icon: Option<iced::widget::svg::Handle>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub width: Length,
    pub body: TypeStyle,
}

pub(crate) struct Field<'a, Message> {
    input: Element<'a, Message>,
    parts: Parts,
    on_trailing_icon: Option<Message>,
    metrics: Metrics,
    tokens: Tokens,
    label_motion: Transition,
    hover_motion: Transition,
    label_style: TypeStyle,
    floating_style: TypeStyle,
    supporting_style: TypeStyle,
    class: StyleFn<'a>,
}

impl<'a, Message> Field<'a, Message> {
    pub fn new(
        theme: &Theme,
        input: Element<'a, Message>,
        parts: Parts,
        on_trailing_icon: Option<Message>,
        class: StyleFn<'a>,
    ) -> Self {
        Field {
            input,
            parts,
            on_trailing_icon,
            metrics: theme.components.text_field,
            tokens: Tokens::new(theme),
            label_motion: Transition {
                duration: theme.motion.duration.short3,
                easing: theme.motion.easing.standard,
            },
            hover_motion: Transition {
                duration: theme.ripple.hover_transition,
                easing: Easing::CubicBezier(0.0, 0.0, 1.0, 1.0),
            },
            label_style: theme.typography.body_large,
            floating_style: theme.typography.body_small,
            supporting_style: theme.typography.body_small,
            class,
        }
    }
}

struct Geometry {
    container: Rectangle,
    content: Rectangle,
    leading: Option<Rectangle>,
    trailing: Option<Rectangle>,
    prefix: Option<Point>,
    suffix: Option<Point>,
    rest_x: f32,
    float_x: f32,
    rest_center: f32,
    float_center: f32,
    supporting_y: f32,
    height: f32,
}

#[derive(Default)]
struct FieldState {
    rest: Label,
    float: Label,
    placeholder: Label,
    prefix: Label,
    suffix: Label,
    supporting: Label,
    counter: Label,
    label_t: Option<Tween>,
    hover_t: Option<Tween>,
    hovered: bool,
    focused: bool,
    now: Option<Instant>,
    trailing: Interaction,
}

impl FieldState {
    fn label_t(&mut self) -> &mut Tween {
        self.label_t.get_or_insert_with(|| Tween::new(0.0))
    }
}

impl<Message> Field<'_, Message> {
    fn floating(&self, state: &FieldState) -> bool {
        state.focused || self.parts.active || self.parts.populated
    }

    fn status(&self, state: &FieldState) -> Status {
        if !self.parts.enabled {
            Status::Disabled
        } else if state.focused || self.parts.active {
            Status::Focused
        } else if state.hovered {
            Status::Hovered
        } else {
            Status::Active
        }
    }

    fn geometry(&self, width: f32, state: &FieldState) -> Geometry {
        let m = &self.metrics;
        let p = &self.parts;
        let filled_label = p.kind == Kind::Filled && p.label.is_some();
        let top = if filled_label {
            m.label_vertical_padding + m.floating_label_line_height
        } else {
            m.vertical_padding
        };
        let bottom = if filled_label {
            m.label_vertical_padding
        } else {
            m.vertical_padding
        };
        let content_height = m.line_height * p.rows as f32;
        let height = top + content_height + bottom;
        let start = if p.leading_icon.is_some() {
            m.icon_padding
        } else {
            m.padding
        };
        let end = if p.trailing_icon.is_some() {
            m.icon_padding
        } else {
            m.padding
        };
        let icon_y = (m.height - m.icon_size) / 2.0;

        let mut x = start;
        let leading = p.leading_icon.is_some().then(|| {
            let r = Rectangle::new(Point::new(x, icon_y), Size::new(m.icon_size, m.icon_size));
            x += m.icon_size + m.icon_text_space;
            r
        });
        let rest_x = x;
        let prefix = p.prefix.is_some().then(|| {
            let at = Point::new(x, top);
            x += state.prefix.size().width + m.affix_space;
            at
        });
        let mut right = width - end;
        let trailing = p.trailing_icon.is_some().then(|| {
            let r = Rectangle::new(
                Point::new(right - m.icon_size, icon_y),
                Size::new(m.icon_size, m.icon_size),
            );
            right = r.x - m.icon_text_space;
            r
        });
        let suffix = p.suffix.is_some().then(|| {
            let w = state.suffix.size().width;
            right -= w;
            let at = Point::new(right, top);
            right -= m.affix_space;
            at
        });
        let float_x = match p.kind {
            Kind::Filled => rest_x,
            Kind::Outlined => start,
        };
        let float_center = match p.kind {
            Kind::Filled => m.label_vertical_padding + m.floating_label_line_height / 2.0,
            Kind::Outlined => 0.0,
        };
        let supporting = p.supporting.is_some() || p.counter.is_some();
        Geometry {
            container: Rectangle::new(Point::ORIGIN, Size::new(width, height)),
            content: Rectangle::new(
                Point::new(x, top),
                Size::new((right - x).max(0.0), content_height),
            ),
            leading,
            trailing,
            prefix,
            suffix,
            rest_x,
            float_x,
            rest_center: m.vertical_padding + m.line_height / 2.0,
            float_center,
            supporting_y: height + m.supporting_text_top_space,
            height: if supporting {
                height + m.supporting_text_top_space + self.supporting_style.line_height
            } else {
                height
            },
        }
    }

    fn translated(rect: Rectangle, origin: Point) -> Rectangle {
        Rectangle::new(
            Point::new(rect.x + origin.x, rect.y + origin.y),
            rect.size(),
        )
    }

    fn trailing_target(&self, icon: Rectangle) -> Rectangle {
        let s = 40.0;
        Rectangle::new(
            Point::new(icon.center_x() - s / 2.0, icon.center_y() - s / 2.0),
            Size::new(s, s),
        )
    }
}

fn outline_path(size: Size, radius: f32, width: f32, gap: (f32, f32)) -> Path {
    let i = width / 2.0;
    let (l, t, r, b) = (i, i, size.width - i, size.height - i);
    let k = (radius - i).max(0.0);
    let g0 = gap.0.clamp(l + k, r - k);
    let g1 = gap.1.clamp(g0, r - k);
    Path::new(|p| {
        p.move_to(Point::new(g1, t));
        p.line_to(Point::new(r - k, t));
        p.arc_to(Point::new(r, t), Point::new(r, t + k), k);
        p.line_to(Point::new(r, b - k));
        p.arc_to(Point::new(r, b), Point::new(r - k, b), k);
        p.line_to(Point::new(l + k, b));
        p.arc_to(Point::new(l, b), Point::new(l, b - k), k);
        p.line_to(Point::new(l, t + k));
        p.arc_to(Point::new(l, t), Point::new(l + k, t), k);
        p.line_to(Point::new(g0, t));
    })
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Field<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FieldState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(FieldState {
            label_t: Some(Tween::new(if self.parts.populated { 1.0 } else { 0.0 })),
            ..FieldState::default()
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.input)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.input));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.parts.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<FieldState>();
        let p = &self.parts;
        if let Some(label) = &p.label {
            state.rest.update(label, self.label_style);
            state.float.update(label, self.floating_style);
        }
        state.placeholder.update(&p.placeholder, p.body);
        state
            .prefix
            .update(p.prefix.as_deref().unwrap_or(""), p.body);
        state
            .suffix
            .update(p.suffix.as_deref().unwrap_or(""), p.body);
        state
            .supporting
            .update(p.supporting.as_deref().unwrap_or(""), self.supporting_style);
        let counter = p
            .counter
            .map(|(n, max)| format!("{n} / {max}"))
            .unwrap_or_default();
        state.counter.update(&counter, self.supporting_style);

        let intrinsic = Size::new(self.metrics.min_width, 0.0);
        let width = limits.resolve(p.width, Length::Shrink, intrinsic).width;
        let geometry = self.geometry(width, state);
        let content = geometry.content;
        let child = self.input.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, content.size()),
        );
        layout::Node::with_children(
            Size::new(width, geometry.height),
            vec![child.move_to(content.position())],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let Some(child) = layout.children().next() else {
            return;
        };
        let (state_tree, children) = (&mut tree.state, &mut tree.children);
        let state = state_tree.downcast_mut::<FieldState>();
        let bounds = layout.bounds();
        self.input
            .as_widget_mut()
            .operate(&mut children[0], child, renderer, operation);
        if self.on_trailing_icon.is_some() && self.parts.enabled {
            let geometry = self.geometry(bounds.width, state);
            if let Some(icon) = geometry.trailing {
                let target = self.trailing_target(Self::translated(icon, bounds.position()));
                operation.focusable(None, target, &mut state.trailing.focus);
            }
        }
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
        let Some(child) = layout.children().next() else {
            return;
        };
        let bounds = layout.bounds();
        let child_layout = child;
        let (state_tree, children) = (&mut tree.state, &mut tree.children);
        let state = state_tree.downcast_mut::<FieldState>();
        let geometry = self.geometry(bounds.width, state);
        let container = Self::translated(geometry.container, bounds.position());
        let enabled = self.parts.enabled;

        if let (Some(message), Some(icon)) = (&self.on_trailing_icon, geometry.trailing) {
            let target = self.trailing_target(Self::translated(icon, bounds.position()));
            if state
                .trailing
                .update(event, target, cursor, enabled, &self.tokens, shell)
                .is_some()
            {
                shell.publish(message.clone());
            }
            if shell.is_event_captured() {
                return;
            }
        }

        let mut input_cursor = cursor;
        if enabled
            && let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(p) = cursor.position_over(container)
        {
            let c = child_layout.bounds();
            if !c.contains(p) {
                input_cursor = Cursor::Available(Point::new(
                    p.x.clamp(c.x, c.x + c.width - 0.5),
                    p.y.clamp(c.y, c.y + c.height - 0.5),
                ));
            }
        }
        self.input.as_widget_mut().update(
            &mut children[0],
            event,
            child_layout,
            input_cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let mut probe = Probe(false);
        self.input
            .as_widget_mut()
            .operate(&mut children[0], child_layout, renderer, &mut probe);
        let now = state.trailing.now;
        let hovered = enabled && cursor.is_over(container);
        if hovered != state.hovered {
            state.hovered = hovered;
            state.hover_t.get_or_insert_with(|| Tween::new(0.0)).go(
                if hovered {
                    self.tokens.state.hover
                } else {
                    0.0
                },
                self.hover_motion,
                now,
            );
            shell.request_redraw();
        }
        if probe.0 != state.focused {
            state.focused = probe.0;
            shell.request_redraw();
        }
        let floating = self.floating(state);
        state
            .label_t()
            .go(if floating { 1.0 } else { 0.0 }, self.label_motion, now);
        if let Event::Window(window::Event::RedrawRequested(at)) = event {
            state.now = Some(*at);
            let mut animating = state.label_t().tick(*at);
            animating |= state.hover_t.as_mut().is_some_and(|t| t.tick(*at));
            if animating {
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
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let Some(child) = layout.children().next() else {
            return;
        };
        let state = tree.state.downcast_ref::<FieldState>();
        let bounds = layout.bounds();
        let origin = bounds.position();
        let geometry = self.geometry(bounds.width, state);
        let container = Self::translated(geometry.container, origin);
        let p = &self.parts;
        let variant = Variant {
            kind: p.kind,
            error: p.error,
        };
        let look = theme.style(&self.class, self.status(state), variant);
        let now = state.now;
        let t = state.label_t.map_or(0.0, |t| t.value(now)).clamp(0.0, 1.0);
        let hover = state.hover_t.map_or(0.0, |t| t.value(now));
        let clip = bounds.intersection(viewport).unwrap_or(bounds);

        match p.kind {
            Kind::Filled => {
                let radius = theme.shape.extra_small_top.radius(container.size());
                surface::fill(renderer, container, radius, look.container);
                surface::state_layer(renderer, container, radius, look.state_layer, hover);
                let h = look.indicator_height;
                surface::fill(
                    renderer,
                    Rectangle::new(
                        Point::new(container.x, container.y + container.height - h),
                        Size::new(container.width, h),
                    ),
                    Radius::default(),
                    look.indicator,
                );
            }
            Kind::Outlined => {
                let radius = theme.shape.extra_small.radius(container.size());
                if p.label.is_some() && t > 0.0 {
                    let float = state.float.size();
                    let pad = self.metrics.outline_label_padding;
                    let gap = (
                        geometry.float_x - pad,
                        geometry.float_x - pad + (float.width + pad * 2.0) * t,
                    );
                    let mut frame = Frame::new(renderer, container.size());
                    frame.stroke(
                        &outline_path(container.size(), radius.top_left, look.outline_width, gap),
                        Stroke::default()
                            .with_color(look.outline)
                            .with_width(look.outline_width),
                    );
                    let geometry = frame.into_geometry();
                    renderer.with_translation(Vector::new(container.x, container.y), |r| {
                        r.draw_geometry(geometry);
                    });
                } else {
                    surface::outline(
                        renderer,
                        container,
                        radius,
                        look.outline_width,
                        look.outline,
                    );
                }
            }
        }

        if p.label.is_some() {
            let x = geometry.rest_x + (geometry.float_x - geometry.rest_x) * t;
            let center = geometry.rest_center + (geometry.float_center - geometry.rest_center) * t;
            for (label, alpha_t) in [(&state.rest, 1.0 - t), (&state.float, t)] {
                if alpha_t > 0.0 {
                    let size = label.size();
                    label.draw(
                        renderer,
                        Point::new(origin.x + x, origin.y + center - size.height / 2.0),
                        alpha(look.label, alpha_t),
                        *viewport,
                    );
                }
            }
        }

        let content_visible = if p.label.is_some() { t } else { 1.0 };
        let content = Self::translated(geometry.content, origin);
        if content_visible > 0.0 {
            if !p.populated && !p.placeholder.is_empty() {
                state.placeholder.draw(
                    renderer,
                    content.position(),
                    alpha(look.placeholder, content_visible),
                    clip,
                );
            }
            for (affix, label, at) in [
                (&p.prefix, &state.prefix, geometry.prefix),
                (&p.suffix, &state.suffix, geometry.suffix),
            ] {
                if let (Some(_), Some(at)) = (affix, at) {
                    label.draw(
                        renderer,
                        Point::new(origin.x + at.x, origin.y + at.y),
                        alpha(look.affix, content_visible),
                        clip,
                    );
                }
            }
        }

        let size = self.metrics.icon_size;
        for (icon, handle, color) in [
            (geometry.leading, &p.leading_icon, look.leading_icon),
            (geometry.trailing, &p.trailing_icon, look.trailing_icon),
        ] {
            if let (Some(icon), Some(handle)) = (icon, handle) {
                let r = Self::translated(icon, origin);
                renderer.draw_svg(
                    tinted(handle.clone(), color, 1.0),
                    Rectangle::new(r.position(), Size::new(size, size)),
                    clip,
                );
            }
        }
        if self.on_trailing_icon.is_some()
            && let Some(icon) = geometry.trailing
        {
            let target = self.trailing_target(Self::translated(icon, origin));
            let round = Radius::from(target.width / 2.0);
            let i = &state.trailing;
            if p.enabled {
                surface::state_layer(
                    renderer,
                    target,
                    round,
                    look.trailing_icon,
                    i.hover.value(i.now),
                );
                i.ripple
                    .draw(renderer, target, round, look.trailing_icon, theme, i.now);
            }
            focus_ring::draw(
                renderer,
                target,
                round,
                theme.focus_ring.outward_offset,
                i.focus_ring_width(&self.tokens),
                theme.colors.secondary,
            );
        }

        self.input.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            child,
            cursor,
            &clip,
        );

        let sx = origin.x + self.metrics.supporting_text_padding;
        let sy = origin.y + geometry.supporting_y;
        if p.supporting.is_some() {
            state
                .supporting
                .draw(renderer, Point::new(sx, sy), look.supporting_text, clip);
        }
        if p.counter.is_some() {
            let w = state.counter.size().width;
            state.counter.draw(
                renderer,
                Point::new(
                    origin.x + bounds.width - self.metrics.supporting_text_padding - w,
                    sy,
                ),
                look.supporting_text,
                clip,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(child) = layout.children().next() else {
            return mouse::Interaction::default();
        };
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<FieldState>();
        let geometry = self.geometry(bounds.width, state);
        if self.on_trailing_icon.is_some()
            && self.parts.enabled
            && let Some(icon) = geometry.trailing
            && cursor.is_over(self.trailing_target(Self::translated(icon, bounds.position())))
        {
            return mouse::Interaction::Pointer;
        }
        let inner = self.input.as_widget().mouse_interaction(
            &tree.children[0],
            child,
            cursor,
            viewport,
            renderer,
        );
        if inner == mouse::Interaction::default()
            && self.parts.enabled
            && cursor.is_over(Self::translated(geometry.container, bounds.position()))
        {
            mouse::Interaction::Text
        } else {
            inner
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
        let child = layout.children().next()?;
        self.input.as_widget_mut().overlay(
            &mut tree.children[0],
            child,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<Field<'a, Message>> for Element<'a, Message> {
    fn from(field: Field<'a, Message>) -> Self {
        IcedElement::new(field)
    }
}
