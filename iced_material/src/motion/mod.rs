// SPDX-License-Identifier: LGPL-3.0-only

//! Material motion tokens: easing curves and durations.

use std::time::Duration;

/// A cubic Bezier segment from the previous point, as `[x1, y1, x2, y2, x3, y3]`.
pub type Segment = [f32; 6];

/// An easing curve mapping linear progress to eased progress.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// CSS `cubic-bezier(x1, y1, x2, y2)`.
    CubicBezier(f32, f32, f32, f32),
    /// A path of cubic segments starting at (0, 0) and ending at (1, 1).
    Path(&'static [Segment]),
}

const EMPHASIZED_PATH: [Segment; 2] = [
    [0.05, 0.0, 0.133333, 0.06, 0.166666, 0.4],
    [0.208333, 0.82, 0.25, 1.0, 1.0, 1.0],
];

fn bezier(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

fn bezier_derivative(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * (p1 - p0) + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

fn solve(start: (f32, f32), segment: &Segment, x: f32) -> f32 {
    let [x1, y1, x2, y2, x3, y3] = *segment;
    let (x0, y0) = start;
    let mut t = (x - x0) / (x3 - x0);
    for _ in 0..8 {
        let error = bezier(x0, x1, x2, x3, t) - x;
        if error.abs() < 1e-6 {
            return bezier(y0, y1, y2, y3, t);
        }
        let slope = bezier_derivative(x0, x1, x2, x3, t);
        if slope.abs() < 1e-6 {
            break;
        }
        t -= error / slope;
    }
    let (mut low, mut high) = (0.0, 1.0);
    t = 0.5;
    for _ in 0..32 {
        let value = bezier(x0, x1, x2, x3, t);
        if (value - x).abs() < 1e-6 {
            break;
        }
        if value < x {
            low = t;
        } else {
            high = t;
        }
        t = (low + high) / 2.0;
    }
    bezier(y0, y1, y2, y3, t)
}

impl Easing {
    /// Evaluates the curve at linear progress `x` in `[0, 1]`.
    pub fn apply(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        if x == 0.0 || x == 1.0 {
            return x;
        }
        match self {
            Easing::CubicBezier(x1, y1, x2, y2) => {
                solve((0.0, 0.0), &[*x1, *y1, *x2, *y2, 1.0, 1.0], x)
            }
            Easing::Path(segments) => {
                let mut start = (0.0, 0.0);
                for segment in segments.iter() {
                    if x <= segment[4] {
                        return solve(start, segment, x);
                    }
                    start = (segment[4], segment[5]);
                }
                1.0
            }
        }
    }
}

/// Easing tokens.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Easings {
    pub emphasized: Easing,
    pub emphasized_accelerate: Easing,
    pub emphasized_decelerate: Easing,
    pub standard: Easing,
    pub standard_accelerate: Easing,
    pub standard_decelerate: Easing,
    pub legacy: Easing,
    pub legacy_accelerate: Easing,
    pub legacy_decelerate: Easing,
    pub linear: Easing,
}

impl Default for Easings {
    fn default() -> Self {
        Easings {
            emphasized: Easing::Path(&EMPHASIZED_PATH),
            emphasized_accelerate: Easing::CubicBezier(0.3, 0.0, 0.8, 0.15),
            emphasized_decelerate: Easing::CubicBezier(0.05, 0.7, 0.1, 1.0),
            standard: Easing::CubicBezier(0.2, 0.0, 0.0, 1.0),
            standard_accelerate: Easing::CubicBezier(0.3, 0.0, 1.0, 1.0),
            standard_decelerate: Easing::CubicBezier(0.0, 0.0, 0.0, 1.0),
            legacy: Easing::CubicBezier(0.4, 0.0, 0.2, 1.0),
            legacy_accelerate: Easing::CubicBezier(0.4, 0.0, 1.0, 1.0),
            legacy_decelerate: Easing::CubicBezier(0.0, 0.0, 0.2, 1.0),
            linear: Easing::CubicBezier(0.0, 0.0, 1.0, 1.0),
        }
    }
}

/// Duration tokens.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Durations {
    pub short1: Duration,
    pub short2: Duration,
    pub short3: Duration,
    pub short4: Duration,
    pub medium1: Duration,
    pub medium2: Duration,
    pub medium3: Duration,
    pub medium4: Duration,
    pub long1: Duration,
    pub long2: Duration,
    pub long3: Duration,
    pub long4: Duration,
    pub extra_long1: Duration,
    pub extra_long2: Duration,
    pub extra_long3: Duration,
    pub extra_long4: Duration,
}

impl Default for Durations {
    fn default() -> Self {
        let ms = Duration::from_millis;
        Durations {
            short1: ms(50),
            short2: ms(100),
            short3: ms(150),
            short4: ms(200),
            medium1: ms(250),
            medium2: ms(300),
            medium3: ms(350),
            medium4: ms(400),
            long1: ms(450),
            long2: ms(500),
            long3: ms(550),
            long4: ms(600),
            extra_long1: ms(700),
            extra_long2: ms(800),
            extra_long3: ms(900),
            extra_long4: ms(1000),
        }
    }
}

/// A timed transition: a duration and an easing curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    /// Total duration.
    pub duration: Duration,
    /// Easing curve.
    pub easing: Easing,
}

impl Transition {
    /// Eased progress in `[0, 1]` after `elapsed` time.
    pub fn progress(&self, elapsed: Duration) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        self.easing
            .apply(elapsed.as_secs_f32() / self.duration.as_secs_f32())
    }
}

/// Motion tokens.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Motion {
    /// Easing curves.
    pub easing: Easings,
    /// Durations.
    pub duration: Durations,
}

mod tween;

pub use tween::Tween;
