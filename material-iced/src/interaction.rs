// SPDX-License-Identifier: LGPL-3.0-only

use std::time::Duration;

use iced::advanced::Shell;
use iced::advanced::widget::operation::Focusable;
use iced::keyboard::{self, key::Named};
use iced::mouse::{self, Cursor};
use iced::time::Instant;
use iced::{Event, Point, Rectangle, window};

use crate::draw::focus_ring;
use crate::draw::ripple::Ripple;
use crate::motion::{Easing, Transition, Tween};
use crate::state::{FocusRing, Ripple as RippleTokens, StateLayers};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    pub ripple: RippleTokens,
    pub state: StateLayers,
    pub focus_ring: FocusRing,
    pub standard: Easing,
    pub emphasized: Easing,
    pub elevation: Duration,
}

impl Tokens {
    pub fn new(theme: &Theme) -> Tokens {
        Tokens {
            ripple: theme.ripple,
            state: theme.state,
            focus_ring: theme.focus_ring,
            standard: theme.motion.easing.standard,
            emphasized: theme.motion.easing.emphasized,
            elevation: theme.elevation.transition,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    Pointer,
    Keyboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Focus {
    pub focused: bool,
    pub visible: bool,
    pub since: Option<Instant>,
}

impl Focusable for Focus {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.visible = true;
        self.since = None;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.visible = false;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interaction {
    pub hovered: bool,
    pub pressed: bool,
    pub focus: Focus,
    pub now: Option<Instant>,
    pub hover: Tween,
    pub ripple: Ripple,
}

impl Default for Interaction {
    fn default() -> Self {
        Interaction {
            hovered: false,
            pressed: false,
            focus: Focus::default(),
            now: None,
            hover: Tween::new(0.0),
            ripple: Ripple::default(),
        }
    }
}

impl Interaction {
    pub fn update<Message>(
        &mut self,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
        enabled: bool,
        tokens: &Tokens,
        shell: &mut Shell<'_, Message>,
    ) -> Option<Activation> {
        if !enabled {
            self.hovered = false;
            self.pressed = false;
            self.focus.unfocus();
            self.hover.set(0.0);
            return None;
        }
        let linear = Easing::CubicBezier(0.0, 0.0, 1.0, 1.0);
        let hovered = cursor.is_over(bounds);
        if hovered != self.hovered {
            self.hovered = hovered;
            self.hover.go(
                if hovered { tokens.state.hover } else { 0.0 },
                Transition {
                    duration: tokens.ripple.hover_transition,
                    easing: linear,
                },
                self.now,
            );
            if !hovered && self.pressed {
                self.ripple.release();
            }
            shell.request_redraw();
        }
        let mut activation = None;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_over(bounds) {
                    self.pressed = true;
                    self.focus.focused = true;
                    self.focus.visible = false;
                    self.ripple.press(
                        Point::new(position.x - bounds.x, position.y - bounds.y),
                        tokens.state.pressed,
                        &tokens.ripple,
                        self.now,
                    );
                    shell.capture_event();
                    shell.request_redraw();
                } else if self.focus.focused {
                    self.focus.unfocus();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if self.pressed => {
                self.pressed = false;
                self.ripple.release();
                if hovered {
                    activation = Some(Activation::Pointer);
                }
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Enter),
                ..
            }) if self.focus.focused => {
                activation = Some(Activation::Keyboard);
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Space),
                ..
            }) if self.focus.focused => {
                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(Named::Space),
                ..
            }) if self.focus.focused => {
                activation = Some(Activation::Keyboard);
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                self.now = Some(*now);
                if self.focus.visible && self.focus.since.is_none() {
                    self.focus.since = Some(*now);
                }
                let ring = self.focus.since.is_some_and(|since| {
                    now.saturating_duration_since(since) < tokens.focus_ring.duration
                });
                let hover = self.hover.tick(*now);
                let ripple = self.ripple.tick(*now, &tokens.ripple);
                if hover || ripple || (self.focus.visible && ring) {
                    shell.request_redraw();
                }
            }
            _ => {}
        }
        if activation == Some(Activation::Keyboard) {
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            self.ripple
                .press(center, tokens.state.pressed, &tokens.ripple, self.now);
            self.ripple.release();
            shell.capture_event();
            shell.request_redraw();
        }
        activation
    }

    pub fn focus_ring_width(&self, tokens: &Tokens) -> f32 {
        if !self.focus.visible {
            return 0.0;
        }
        match (self.focus.since, self.now) {
            (Some(since), Some(now)) => focus_ring::width(
                &tokens.focus_ring,
                &tokens.emphasized,
                now.saturating_duration_since(since),
            ),
            _ => 0.0,
        }
    }
}
