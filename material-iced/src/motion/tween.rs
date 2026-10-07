// SPDX-License-Identifier: LGPL-3.0-only

use iced::time::Instant;

use super::{Easing, Transition};

/// An animated scalar driven by frame timestamps.
///
/// A new target starts on the next frame passed to [`Tween::tick`], so the
/// animation clock only depends on redraw timestamps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween {
    from: f32,
    to: f32,
    start: Option<Instant>,
    transition: Transition,
}

impl Tween {
    /// A tween resting at `value`.
    pub fn new(value: f32) -> Tween {
        Tween {
            from: value,
            to: value,
            start: None,
            transition: Transition {
                duration: std::time::Duration::ZERO,
                easing: Easing::CubicBezier(0.0, 0.0, 1.0, 1.0),
            },
        }
    }

    /// The value the tween is moving to.
    pub fn target(&self) -> f32 {
        self.to
    }

    /// Value at time `now`.
    pub fn value(&self, now: Option<Instant>) -> f32 {
        match (self.start, now) {
            (Some(start), Some(now)) => {
                let t = self
                    .transition
                    .progress(now.saturating_duration_since(start));
                self.from + (self.to - self.from) * t
            }
            _ => self.from,
        }
    }

    /// Starts moving to `to` from the value at `now`.
    pub fn go(&mut self, to: f32, transition: Transition, now: Option<Instant>) {
        if to == self.to {
            return;
        }
        self.from = self.value(now);
        self.to = to;
        self.start = None;
        self.transition = transition;
    }

    /// Jumps to `value` without animating.
    pub fn set(&mut self, value: f32) {
        *self = Tween::new(value);
    }

    /// Records the frame time. Returns `true` while the tween still animates.
    pub fn tick(&mut self, now: Instant) -> bool {
        if self.from == self.to {
            return false;
        }
        let start = *self.start.get_or_insert(now);
        if now.saturating_duration_since(start) >= self.transition.duration {
            self.from = self.to;
            self.start = None;
            return false;
        }
        true
    }
}
