// SPDX-License-Identifier: LGPL-3.0-only

//! Checkboxes with an indeterminate and an error state.

use iced::advanced::layout::{self, Layout};
use iced::advanced::svg::Renderer as _;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, renderer};
use iced::border::Radius;
use iced::{Color, Element as IcedElement, Event, Length, Rectangle, Renderer, Size, mouse};

use crate::Element;
use crate::draw::{focus_ring, surface};
use crate::icon::{symbol, tinted};
use crate::interaction::Tokens;
use crate::motion::Transition;
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::pressable::Status;
use crate::widget::selection::{self, State, centered};

/// Checkbox dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Width and height of the widget, which is also the pointer target.
    pub target_size: f32,
    /// Size of the container.
    pub container_size: f32,
    /// Corner radius of the container.
    pub corner: f32,
    /// Outline width of an unselected container.
    pub outline_width: f32,
    /// Size of the checkmark and the indeterminate mark.
    pub icon_size: f32,
    /// Diameter of the state layer.
    pub state_layer_size: f32,
    /// Width and height of the indeterminate mark.
    pub dash: (f32, f32),
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            target_size: 48.0,
            container_size: 18.0,
            corner: 2.0,
            outline_width: 2.0,
            icon_size: 18.0,
            state_layer_size: 40.0,
            dash: (10.0, 2.0),
        }
    }
}

/// The mark inside a checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// Empty.
    Unchecked,
    /// A checkmark.
    Checked,
    /// A dash.
    Indeterminate,
}

/// What a checkbox style depends on besides the status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variant {
    /// The mark.
    pub check: Check,
    /// Whether the checkbox shows an error.
    pub error: bool,
}

/// The appearance of a checkbox in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Container fill. Transparent when unselected.
    pub container: Color,
    /// Outline color of an unselected container.
    pub outline: Color,
    /// Color of the mark.
    pub icon: Color,
    /// Hover and focus state layer color.
    pub hover_layer: Color,
    /// Press ripple color.
    pub pressed_layer: Color,
}

/// The appearance catalog of checkboxes.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status.
    fn style(&self, class: &Self::Class<'_>, status: Status, variant: Variant) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, Variant) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, variant: Variant) -> Style {
        class(self, status, variant)
    }
}

/// The baseline checkbox style.
pub fn style(theme: &Theme, status: Status, variant: Variant) -> Style {
    let c = &theme.colors;
    let selected = variant.check != Check::Unchecked;
    let accent = if variant.error { c.error } else { c.primary };
    if status == Status::Disabled {
        let disabled = alpha(c.on_surface, theme.disabled.content);
        return Style {
            container: if selected {
                disabled
            } else {
                Color::TRANSPARENT
            },
            outline: disabled,
            icon: c.surface,
            hover_layer: c.on_surface,
            pressed_layer: c.on_surface,
        };
    }
    let engaged = status != Status::Active;
    let outline = match (variant.error, engaged) {
        (true, _) => c.error,
        (false, true) => c.on_surface,
        (false, false) => c.on_surface_variant,
    };
    let (hover_layer, pressed_layer) = match (variant.error, selected) {
        (true, _) => (c.error, c.error),
        (false, true) => (c.primary, c.on_surface),
        (false, false) => (c.on_surface, c.primary),
    };
    Style {
        container: if selected { accent } else { Color::TRANSPARENT },
        outline,
        icon: if variant.error {
            c.on_error
        } else {
            c.on_primary
        },
        hover_layer,
        pressed_layer,
    }
}

/// A Material checkbox.
pub struct Checkbox<'a, Message> {
    checked: bool,
    indeterminate: bool,
    error: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    metrics: Metrics,
    tokens: Tokens,
    enter: Transition,
    exit: Transition,
    class: StyleFn<'a>,
}

/// A checkbox. Without `on_toggle` it is disabled.
pub fn checkbox<'a, Message>(theme: &Theme, checked: bool) -> Checkbox<'a, Message> {
    let motion = &theme.motion;
    Checkbox {
        checked,
        indeterminate: false,
        error: false,
        on_toggle: None,
        metrics: theme.components.checkbox,
        tokens: Tokens::new(theme),
        enter: Transition {
            duration: std::time::Duration::from_millis(350),
            easing: motion.easing.emphasized_decelerate,
        },
        exit: Transition {
            duration: motion.duration.short3,
            easing: motion.easing.emphasized_accelerate,
        },
        class: Box::new(style),
    }
}

impl<'a, Message> Checkbox<'a, Message> {
    /// Sets the message produced with the new value when toggled.
    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    /// Shows the indeterminate mark instead of the checkmark.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Shows the error colors.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, Variant) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    fn selected(&self) -> bool {
        self.checked || self.indeterminate
    }

    fn variant(&self) -> Variant {
        Variant {
            check: match (self.indeterminate, self.checked) {
                (true, _) => Check::Indeterminate,
                (false, true) => Check::Checked,
                (false, false) => Check::Unchecked,
            },
            error: self.error,
        }
    }

    fn layer(&self, bounds: Rectangle) -> Rectangle {
        let s = self.metrics.state_layer_size;
        centered(bounds, Size::new(s, s))
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Checkbox<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.selected()))
    }

    fn size(&self) -> Size<Length> {
        let s = Length::Fixed(self.metrics.target_size);
        Size::new(s, s)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let s = self.metrics.target_size;
        layout::Node::new(Size::new(s, s))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_toggle.is_some() {
            let state = tree.state.downcast_mut::<State>();
            operation.focusable(
                None,
                self.layer(layout.bounds()),
                &mut state.press.interaction.focus,
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
        let state = tree.state.downcast_mut::<State>();
        let transition = if self.selected() {
            self.enter
        } else {
            self.exit
        };
        let activation = state.update(
            event,
            self.selected(),
            transition,
            layout.bounds(),
            cursor,
            self.on_toggle.is_some(),
            &self.tokens,
            shell,
        );
        if activation.is_some()
            && let Some(on_toggle) = &self.on_toggle
        {
            shell.publish(on_toggle(!self.checked));
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
        let m = &self.metrics;
        let enabled = self.on_toggle.is_some();
        let bounds = layout.bounds();
        let layer = self.layer(bounds);
        let variant = self.variant();
        let style = theme.style(&self.class, state.press.status(enabled, false), variant);

        if enabled {
            selection::draw_state_layer(
                renderer,
                theme,
                state,
                layer,
                style.hover_layer,
                style.pressed_layer,
            );
        }

        let t = state.progress();
        let fill = (t * 7.0).clamp(0.0, 1.0);
        let scale = 0.6 + 0.4 * t.clamp(0.0, 1.0);
        let size = m.container_size;
        let box_bounds = centered(bounds, Size::new(size, size));
        let radius = Radius::from(m.corner);
        if fill < 1.0 {
            surface::outline(
                renderer,
                box_bounds,
                radius,
                m.outline_width,
                alpha(style.outline, 1.0 - fill),
            );
        }
        if fill > 0.0 {
            let scaled = centered(bounds, Size::new(size * scale, size * scale));
            surface::fill(
                renderer,
                scaled,
                Radius::from(m.corner * scale),
                alpha(style.container, fill),
            );
            let clip = bounds.intersection(viewport).unwrap_or(bounds);
            match variant.check {
                Check::Indeterminate => {
                    let (w, h) = m.dash;
                    let dash = centered(bounds, Size::new(w * scale, h));
                    surface::fill(renderer, dash, Radius::default(), alpha(style.icon, fill));
                }
                _ => {
                    let icon = m.icon_size * scale;
                    renderer.draw_svg(
                        tinted(symbol::check(false), style.icon, fill),
                        centered(bounds, Size::new(icon, icon)),
                        clip,
                    );
                }
            }
        }

        let ring = state.press.interaction.focus_ring_width(&self.tokens);
        focus_ring::draw(
            renderer,
            layer,
            Radius::from(layer.width / 2.0),
            theme.focus_ring.outward_offset,
            ring,
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

impl<'a, Message: Clone + 'a> From<Checkbox<'a, Message>> for Element<'a, Message> {
    fn from(checkbox: Checkbox<'a, Message>) -> Self {
        IcedElement::new(checkbox)
    }
}
