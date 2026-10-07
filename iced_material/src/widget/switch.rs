// SPDX-License-Identifier: LGPL-3.0-only

//! Switches with optional icons in the handle.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::time::Instant;
use iced::{
    Color, Element as IcedElement, Event, Length, Point, Rectangle, Renderer, Size, mouse, window,
};

use crate::Element;
use crate::draw::{focus_ring, surface};
use crate::icon::{symbol, tinted};
use crate::interaction::Tokens;
use crate::motion::{Easing, Transition, Tween};
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::pressable::Status;
use crate::widget::selection::{self, centered};

/// Switch dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Width of the track.
    pub track_width: f32,
    /// Height of the track.
    pub track_height: f32,
    /// Outline width of the track while unselected.
    pub outline_width: f32,
    /// Handle diameter while unselected without an icon.
    pub unselected_handle: f32,
    /// Handle diameter while selected or showing an icon.
    pub selected_handle: f32,
    /// Handle diameter while pressed.
    pub pressed_handle: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Diameter of the state layer.
    pub state_layer_size: f32,
    /// Height of the widget, which is also the pointer target.
    pub target_height: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            track_width: 52.0,
            track_height: 32.0,
            outline_width: 2.0,
            unselected_handle: 16.0,
            selected_handle: 24.0,
            pressed_handle: 28.0,
            icon_size: 16.0,
            state_layer_size: 40.0,
            target_height: 48.0,
        }
    }
}

/// The appearance of a switch in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Track fill.
    pub track: Color,
    /// Track outline. Transparent while selected.
    pub outline: Color,
    /// Handle fill.
    pub handle: Color,
    /// Icon color.
    pub icon: Color,
    /// Hover and focus state layer color.
    pub hover_layer: Color,
    /// Press ripple color.
    pub pressed_layer: Color,
}

/// The appearance catalog of switches.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, bool) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, selected: bool) -> Style {
        class(self, status, selected)
    }
}

/// The baseline switch style.
pub fn style(theme: &Theme, status: Status, selected: bool) -> Style {
    let c = &theme.colors;
    let disabled_track = theme.disabled.container;
    let disabled_content = theme.disabled.content;
    match (status, selected) {
        (Status::Disabled, true) => Style {
            track: alpha(c.on_surface, disabled_track),
            outline: Color::TRANSPARENT,
            handle: c.surface,
            icon: alpha(c.on_surface, disabled_content),
            hover_layer: c.on_surface,
            pressed_layer: c.on_surface,
        },
        (Status::Disabled, false) => Style {
            track: alpha(c.surface_container_highest, disabled_track),
            outline: alpha(c.on_surface, disabled_track),
            handle: alpha(c.on_surface, disabled_content),
            icon: alpha(c.surface_container_highest, disabled_content),
            hover_layer: c.on_surface,
            pressed_layer: c.on_surface,
        },
        (status, true) => Style {
            track: c.primary,
            outline: Color::TRANSPARENT,
            handle: if status == Status::Active {
                c.on_primary
            } else {
                c.primary_container
            },
            icon: c.on_primary_container,
            hover_layer: c.primary,
            pressed_layer: c.primary,
        },
        (status, false) => Style {
            track: c.surface_container_highest,
            outline: c.outline,
            handle: if status == Status::Active {
                c.outline
            } else {
                c.on_surface_variant
            },
            icon: c.surface_container_highest,
            hover_layer: c.on_surface,
            pressed_layer: c.on_surface,
        },
    }
}

/// A Material switch.
pub struct Switch<'a, Message> {
    selected: bool,
    icons: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    metrics: Metrics,
    tokens: Tokens,
    slide: Transition,
    resize: Transition,
    class: StyleFn<'a>,
}

/// A switch. Without `on_toggle` it is disabled.
pub fn switch<'a, Message>(theme: &Theme, selected: bool) -> Switch<'a, Message> {
    Switch {
        selected,
        icons: false,
        on_toggle: None,
        metrics: theme.components.switch,
        tokens: Tokens::new(theme),
        slide: Transition {
            duration: theme.motion.duration.medium2,
            easing: Easing::CubicBezier(0.175, 0.885, 0.32, 1.275),
        },
        resize: Transition {
            duration: theme.motion.duration.medium1,
            easing: theme.motion.easing.standard,
        },
        class: Box::new(style),
    }
}

impl<'a, Message> Switch<'a, Message> {
    /// Sets the message produced with the new value when toggled.
    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    /// Shows a checkmark or a cross in the handle.
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, bool) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn track(&self, bounds: Rectangle) -> Rectangle {
        centered(
            bounds,
            Size::new(self.metrics.track_width, self.metrics.track_height),
        )
    }

    fn handle_size(&self, pressed: bool) -> f32 {
        let m = &self.metrics;
        if pressed {
            m.pressed_handle
        } else if self.selected || self.icons {
            m.selected_handle
        } else {
            m.unselected_handle
        }
    }

    fn handle_center(&self, track: Rectangle, progress: f32) -> Point {
        let m = &self.metrics;
        let travel = m.track_width - m.track_height;
        Point::new(
            track.x + m.track_height / 2.0 + travel * progress,
            track.center_y(),
        )
    }
}

struct SwitchState {
    selection: selection::State,
    size: Tween,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Switch<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SwitchState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SwitchState {
            selection: selection::State::new(self.selected),
            size: Tween::new(self.handle_size(false)),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.metrics.track_width),
            Length::Fixed(self.metrics.target_height),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(
            self.metrics.track_width,
            self.metrics.target_height,
        ))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_toggle.is_some() {
            let state = tree.state.downcast_mut::<SwitchState>();
            operation.focusable(
                None,
                self.track(layout.bounds()),
                &mut state.selection.press.interaction.focus,
            );
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
        let state = tree.state.downcast_mut::<SwitchState>();
        let enabled = self.on_toggle.is_some();
        let slide = if enabled {
            self.slide
        } else {
            Transition {
                duration: std::time::Duration::ZERO,
                ..self.slide
            }
        };
        let activation = state.selection.update(
            event,
            self.selected,
            slide,
            layout.bounds(),
            cursor,
            enabled,
            &self.tokens,
            shell,
        );
        let now: Option<Instant> = state.selection.now();
        let pressed = state.selection.press.interaction.pressed;
        state.size.go(self.handle_size(pressed), self.resize, now);
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && state.size.tick(*now)
        {
            shell.request_redraw();
        }
        if activation.is_some()
            && let Some(on_toggle) = &self.on_toggle
        {
            shell.publish(on_toggle(!self.selected));
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
        let state = tree.state.downcast_ref::<SwitchState>();
        let m = &self.metrics;
        let enabled = self.on_toggle.is_some();
        let bounds = layout.bounds();
        let track = self.track(bounds);
        let t = state.selection.progress();
        let style = theme.style(
            &self.class,
            state.selection.press.status(enabled, false),
            self.selected,
        );

        let radius = Radius::from(m.track_height / 2.0);
        surface::fill(renderer, track, radius, style.track);
        surface::outline(renderer, track, radius, m.outline_width, style.outline);

        let center = self.handle_center(track, t);
        if enabled {
            let s = m.state_layer_size;
            selection::draw_state_layer(
                renderer,
                theme,
                &state.selection,
                centered(Rectangle::new(center, Size::ZERO), Size::new(s, s)),
                style.hover_layer,
                style.pressed_layer,
            );
        }
        let size = state.size.value(state.selection.now());
        surface::fill(
            renderer,
            centered(Rectangle::new(center, Size::ZERO), Size::new(size, size)),
            Radius::from(size / 2.0),
            style.handle,
        );
        if self.icons {
            let clip = bounds.intersection(viewport).unwrap_or(bounds);
            let icon = Rectangle::new(
                Point::new(center.x - m.icon_size / 2.0, center.y - m.icon_size / 2.0),
                Size::new(m.icon_size, m.icon_size),
            );
            let t = t.clamp(0.0, 1.0);
            for (handle, opacity) in [(symbol::check(false), t), (symbol::close(false), 1.0 - t)] {
                if opacity > 0.0 {
                    renderer.draw_svg(tinted(handle, style.icon, opacity), icon, clip);
                }
            }
        }
        focus_ring::draw(
            renderer,
            track,
            radius,
            theme.focus_ring.outward_offset,
            state
                .selection
                .press
                .interaction
                .focus_ring_width(&self.tokens),
            theme.colors.secondary,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_toggle.is_some() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<Switch<'a, Message>> for Element<'a, Message> {
    fn from(switch: Switch<'a, Message>) -> Self {
        IcedElement::new(switch)
    }
}
