// SPDX-License-Identifier: LGPL-3.0-only

//! The pressed state ripple of Material web, clipped to a rounded shape.

use std::f32::consts::{FRAC_PI_2, PI};

use iced::advanced::Renderer as _;
use iced::advanced::graphics::geometry::Renderer as _;
use iced::border::Radius;
use iced::time::Instant;
use iced::widget::canvas::fill::Rule;
use iced::widget::canvas::{Fill, Frame, Path, Style};
use iced::{Color, Point, Rectangle, Renderer, Vector};

use crate::motion::{Easing, Transition, Tween};
use crate::state::{Ripple as Tokens, alpha};
use crate::theme::Theme;

const RINGS: usize = 16;

/// Runtime state of one ripple.
#[derive(Debug, Clone, PartialEq)]
pub struct Ripple {
    origin: Point,
    start: Option<Instant>,
    growing: bool,
    release_requested: bool,
    opacity: Tween,
}

impl Default for Ripple {
    fn default() -> Self {
        Ripple {
            origin: Point::ORIGIN,
            start: None,
            growing: false,
            release_requested: false,
            opacity: Tween::new(0.0),
        }
    }
}

impl Ripple {
    /// Starts a ripple at `origin`, relative to the top left of the shape.
    pub fn press(&mut self, origin: Point, opacity: f32, tokens: &Tokens, now: Option<Instant>) {
        self.origin = origin;
        self.start = None;
        self.growing = true;
        self.release_requested = false;
        self.opacity.go(
            opacity,
            Transition {
                duration: tokens.fade_in,
                easing: Easing::CubicBezier(0.0, 0.0, 1.0, 1.0),
            },
            now,
        );
    }

    /// Releases the ripple. It fades out once the minimum press time has passed.
    pub fn release(&mut self) {
        if self.growing || self.opacity.target() > 0.0 {
            self.release_requested = true;
        }
    }

    /// Advances the ripple to frame time `now`. Returns `true` while it animates.
    pub fn tick(&mut self, now: Instant, tokens: &Tokens) -> bool {
        if self.growing {
            let start = *self.start.get_or_insert(now);
            let elapsed = now.saturating_duration_since(start);
            if self.release_requested && elapsed >= tokens.minimum_press {
                self.release_requested = false;
                self.opacity.go(
                    0.0,
                    Transition {
                        duration: tokens.fade_out,
                        easing: Easing::CubicBezier(0.0, 0.0, 1.0, 1.0),
                    },
                    Some(now),
                );
            }
        }
        let fading = self.opacity.tick(now);
        if !fading && self.opacity.target() == 0.0 && !self.release_requested {
            self.growing = false;
        }
        fading || self.growing
    }

    /// Draws the ripple inside a shape with `bounds` and `radius`.
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        radius: Radius,
        color: Color,
        theme: &Theme,
        now: Option<Instant>,
    ) {
        let tokens = &theme.ripple;
        let opacity = self.opacity.value(now);
        if opacity <= 0.0 {
            return;
        }
        let elapsed = match (self.start, now) {
            (Some(start), Some(now)) => now.saturating_duration_since(start),
            _ => std::time::Duration::ZERO,
        };
        let progress = Transition {
            duration: tokens.press_grow,
            easing: theme.motion.easing.standard,
        }
        .progress(elapsed);

        let (width, height) = (bounds.width, bounds.height);
        let max_dim = width.max(height);
        let soft_edge =
            (tokens.soft_edge_container_ratio * max_dim).max(tokens.soft_edge_minimum_size);
        let initial = (max_dim * tokens.initial_origin_scale).floor();
        let max_radius = (width * width + height * height).sqrt() + tokens.padding;
        let scale = (max_radius + soft_edge) / initial;
        let center = Point::new(
            self.origin.x + (width / 2.0 - self.origin.x) * progress,
            self.origin.y + (height / 2.0 - self.origin.y) * progress,
        );
        let outer = initial * (1.0 + (scale - 1.0) * progress) / 2.0;
        let unscaled = initial / 2.0;
        let solid_fraction = (1.0 - tokens.soft_edge_falloff / unscaled).max(tokens.solid_fraction);
        let solid = outer * solid_fraction;

        let clip = rounded_rect(width, height, radius);
        let mut frame = Frame::new(renderer, bounds.size());
        let edge = |ring: usize| outer - (outer - solid) * ring as f32 / RINGS as f32;
        for ring in 0..=RINGS {
            let coverage = if ring == RINGS {
                1.0
            } else {
                let mid = outer - (outer - solid) * (ring as f32 + 0.5) / RINGS as f32;
                (outer - mid) / (outer - solid)
            };
            let outside = clip_polygon(&circle(center, edge(ring)), &clip);
            if outside.len() < 3 {
                continue;
            }
            let inside = if ring == RINGS {
                Vec::new()
            } else {
                clip_polygon(&circle(center, edge(ring + 1)), &clip)
            };
            let path = Path::new(|b| {
                for polygon in [&outside, &inside] {
                    if polygon.len() < 3 {
                        continue;
                    }
                    b.move_to(polygon[0]);
                    for point in &polygon[1..] {
                        b.line_to(*point);
                    }
                    b.close();
                }
            });
            frame.fill(
                &path,
                Fill {
                    style: Style::Solid(alpha(color, opacity * coverage)),
                    rule: Rule::EvenOdd,
                },
            );
        }
        let geometry = frame.into_geometry();
        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_geometry(geometry);
        });
    }
}

fn circle(center: Point, radius: f32) -> Vec<Point> {
    let segments = ((2.0 * PI * radius) / 2.0).ceil().clamp(32.0, 256.0) as usize;
    (0..segments)
        .map(|i| {
            let angle = 2.0 * PI * i as f32 / segments as f32;
            Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}

fn rounded_rect(width: f32, height: f32, radius: Radius) -> Vec<Point> {
    let limit = width.min(height) / 2.0;
    let corners = [
        (
            radius.top_right.min(limit),
            Point::new(width, 0.0),
            -FRAC_PI_2,
            Vector::new(-1.0, 1.0),
        ),
        (
            radius.bottom_right.min(limit),
            Point::new(width, height),
            0.0,
            Vector::new(-1.0, -1.0),
        ),
        (
            radius.bottom_left.min(limit),
            Point::new(0.0, height),
            FRAC_PI_2,
            Vector::new(1.0, -1.0),
        ),
        (
            radius.top_left.min(limit),
            Point::new(0.0, 0.0),
            PI,
            Vector::new(1.0, 1.0),
        ),
    ];
    let mut points = Vec::new();
    for (r, corner, start, inward) in corners {
        if r <= 0.0 {
            points.push(corner);
            continue;
        }
        let center = Point::new(corner.x + inward.x * r, corner.y + inward.y * r);
        let steps = (r * FRAC_PI_2).ceil().clamp(4.0, 64.0) as usize;
        for i in 0..=steps {
            let angle = start + FRAC_PI_2 * i as f32 / steps as f32;
            points.push(Point::new(
                center.x + r * angle.cos(),
                center.y + r * angle.sin(),
            ));
        }
    }
    points
}

fn clip_polygon(subject: &[Point], clip: &[Point]) -> Vec<Point> {
    let mut output = subject.to_vec();
    for i in 0..clip.len() {
        if output.is_empty() {
            break;
        }
        let a = clip[i];
        let b = clip[(i + 1) % clip.len()];
        let inside = |p: Point| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x) >= 0.0;
        let intersect = |p: Point, q: Point| {
            let d1 = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
            let d2 = (b.x - a.x) * (q.y - a.y) - (b.y - a.y) * (q.x - a.x);
            let t = d1 / (d1 - d2);
            Point::new(p.x + (q.x - p.x) * t, p.y + (q.y - p.y) * t)
        };
        let input = std::mem::take(&mut output);
        for j in 0..input.len() {
            let current = input[j];
            let previous = input[(j + input.len() - 1) % input.len()];
            match (inside(current), inside(previous)) {
                (true, true) => output.push(current),
                (true, false) => {
                    output.push(intersect(previous, current));
                    output.push(current);
                }
                (false, true) => output.push(intersect(previous, current)),
                (false, false) => {}
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_keeps_circle_inside_rounded_rect() {
        let clip = rounded_rect(100.0, 40.0, Radius::from(20.0));
        let polygon = clip_polygon(&circle(Point::new(50.0, 20.0), 80.0), &clip);
        for p in &polygon {
            assert!(p.x >= -0.01 && p.x <= 100.01 && p.y >= -0.01 && p.y <= 40.01);
            let corner_center = Point::new(20.0, 20.0);
            if p.x < 20.0 {
                assert!(p.distance(corner_center) <= 20.01, "{p:?}");
            }
        }
        let inner = clip_polygon(&circle(Point::new(50.0, 20.0), 5.0), &clip);
        assert_eq!(inner.len(), circle(Point::new(50.0, 20.0), 5.0).len());
    }
}
