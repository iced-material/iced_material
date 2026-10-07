// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, rgb};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::time::Instant;
use iced::{Length, Point, Rectangle, Renderer, Size, mouse};
use material_iced::draw::ripple::Ripple;
use material_iced::draw::text::Label;
use material_iced::draw::{focus_ring, shadow, surface};
use material_iced::{Element, Theme};

const BOUNDS: Rectangle = Rectangle {
    x: 40.0,
    y: 40.0,
    width: 120.0,
    height: 40.0,
};

struct Probe {
    ripple: Ripple,
    now: Instant,
    elevation: f32,
    ring: f32,
}

impl Widget<(), Theme, Renderer> for Probe {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let radius = theme.shape.full.radius(BOUNDS.size());
        let c = &theme.colors;
        shadow::draw(
            renderer,
            BOUNDS,
            radius,
            self.elevation,
            c.shadow,
            &theme.elevation,
        );
        surface::fill(renderer, BOUNDS, radius, c.primary);
        self.ripple.draw(
            renderer,
            BOUNDS,
            radius,
            c.on_primary,
            theme,
            Some(self.now),
        );
        focus_ring::draw(
            renderer,
            BOUNDS,
            radius,
            theme.focus_ring.outward_offset,
            self.ring,
            c.secondary,
        );
        let outline = Rectangle::new(Point::new(40.0, 120.0), Size::new(120.0, 40.0));
        surface::outline(renderer, outline, 0.0.into(), 1.0, c.outline);
    }
}

fn probe(theme: &Theme, ripple_at: Duration, elevation: f32, ring: f32) -> Element<'static, ()> {
    let start = Instant::now();
    let mut ripple = Ripple::default();
    ripple.press(
        Point::new(20.0, 20.0),
        theme.state.pressed,
        &theme.ripple,
        None,
    );
    ripple.tick(start, &theme.ripple);
    ripple.tick(start + theme.ripple.fade_in, &theme.ripple);
    Element::new(Probe {
        ripple,
        now: start + ripple_at,
        elevation,
        ring,
    })
}

#[test]
fn ripple_is_clipped_to_rounded_shape() {
    let theme = Theme::light();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let mut h = Harness::new(Size::new(200.0, 200.0));
        let image = h.screenshot(
            probe(&theme, Duration::from_millis(450), 0.0, 0.0),
            &theme,
            scale,
        );
        let pressed = material_iced::state::overlay(
            theme.colors.primary,
            theme.colors.on_primary,
            theme.state.pressed,
        );
        let center = image.at(Point::new(100.0, 60.0));
        assert!(
            distance(center, rgb(pressed)) <= 2,
            "scale {scale}: center {center:?} vs {:?}",
            rgb(pressed)
        );
        let corner = image.at(Point::new(41.0, 41.0));
        assert!(
            distance(corner, rgb(theme.colors.surface)) <= 2,
            "scale {scale}: corner {corner:?}"
        );
    }
}

#[test]
fn ripple_grows_from_press_point() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(200.0, 200.0));
    let image = h.screenshot(probe(&theme, Duration::ZERO, 0.0, 0.0), &theme, 1.0);
    let near = image.at(Point::new(60.0, 60.0));
    let far = image.at(Point::new(150.0, 60.0));
    assert_ne!(near, rgb(theme.colors.primary));
    assert_eq!(distance(far, rgb(theme.colors.primary)), 0);
}

#[test]
fn shadow_falls_below_the_shape() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(200.0, 200.0));
    let image = h.screenshot(probe(&theme, Duration::from_secs(2), 3.0, 0.0), &theme, 1.0);
    let surface = rgb(theme.colors.surface);
    let below = image.at(Point::new(100.0, 82.5));
    let above = image.at(Point::new(100.0, 38.5));
    assert!(
        distance(below, surface) > distance(above, surface),
        "below {below:?} above {above:?}"
    );
    assert!(distance(below, surface) >= 8);
}

#[test]
fn focus_ring_sits_outside_the_shape() {
    let theme = Theme::light();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let mut h = Harness::new(Size::new(200.0, 200.0));
        let image = h.screenshot(
            probe(&theme, Duration::from_secs(2), 0.0, 3.0),
            &theme,
            scale,
        );
        let ring = image.at(Point::new(100.0, 40.0 - 2.0 - 1.5));
        assert!(
            distance(ring, rgb(theme.colors.secondary)) <= 2,
            "scale {scale}: {ring:?}"
        );
        let gap = image.at(Point::new(100.0, 39.0));
        assert!(
            distance(gap, rgb(theme.colors.surface)) <= 2,
            "scale {scale}: gap {gap:?}"
        );
    }
}

#[test]
fn one_dp_outline_has_a_solid_pixel_row() {
    let theme = Theme::light();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let mut h = Harness::new(Size::new(200.0, 200.0));
        let image = h.screenshot(
            probe(&theme, Duration::from_secs(2), 0.0, 0.0),
            &theme,
            scale,
        );
        let x = (100.0 * scale) as u32;
        let top = (120.0 * scale).floor() as u32;
        let solid = (top.saturating_sub(1)..top + 2)
            .any(|y| distance(image.pixel(x, y), rgb(theme.colors.outline)) <= 1);
        assert!(solid, "scale {scale}");
    }
}

#[test]
fn focus_ring_animates_like_material_web() {
    let theme = Theme::light();
    let easing = theme.motion.easing.emphasized;
    let width = |ms: u64| focus_ring::width(&theme.focus_ring, &easing, Duration::from_millis(ms));
    assert_eq!(width(0), 0.0);
    assert_eq!(width(150), 8.0);
    assert!(width(300) < 8.0 && width(300) > 3.0);
    assert_eq!(width(600), 3.0);
    assert_eq!(width(5000), 3.0);
}

#[test]
fn label_adds_tracking_after_each_grapheme() {
    let _h = Harness::new(Size::new(10.0, 10.0));
    let theme = Theme::light();
    let style = theme.typography.label_large;
    let mut tracked = Label::default();
    let mut plain = Label::default();
    let a = tracked.update("Button", style);
    let b = plain.update(
        "Button",
        material_iced::typography::TypeStyle {
            tracking: 0.0,
            ..style
        },
    );
    assert!(
        (a.width - b.width - 6.0 * style.tracking).abs() < 1e-3,
        "{a:?} {b:?}"
    );
    assert_eq!(a.height, 20.0);
}

#[test]
fn requested_backend_is_used() {
    use iced::advanced::renderer::Headless;
    let h = Harness::new(Size::new(10.0, 10.0));
    let requested =
        std::env::var("ICED_MATERIAL_BACKEND").unwrap_or_else(|_| "tiny-skia".to_string());
    assert!(
        h.renderer.name().contains(&requested),
        "{}",
        h.renderer.name()
    );
}
