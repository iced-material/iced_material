// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, ink, rgb};
use iced::widget::container;
use iced::{Length, Point, Rectangle, Size};
use material_iced::widget::progress::{circular, linear};
use material_iced::{Element, Theme};

fn place(content: Element<'static, ()>) -> Element<'static, ()> {
    container(content).padding(20).into()
}

fn settle(h: &mut Harness, view: impl Fn() -> Element<'static, ()>, until: u64) {
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(16));
    h.frame(view(), Duration::from_millis(until));
}

#[test]
fn linear_determinate_fills_from_the_start() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || place(linear(&theme, Some(0.5)).width(Length::Fixed(200.0)).into());
    let mut h = Harness::new(Size::new(300.0, 100.0));
    settle(&mut h, view, 2000);
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(60.0, 21.0)), rgb(c.primary)) <= 1);
    assert!(
        distance(
            image.at(Point::new(180.0, 21.0)),
            rgb(c.surface_container_highest)
        ) <= 1
    );
    assert!(distance(image.at(Point::new(60.0, 25.0)), rgb(c.surface)) <= 1);
    assert!(distance(image.at(Point::new(60.0, 19.0)), rgb(c.surface)) <= 1);
}

#[test]
fn linear_determinate_animates_to_the_new_value() {
    let theme = Theme::light();
    let c = theme.colors;
    let build = |value| {
        place(
            linear(&theme, Some(value))
                .width(Length::Fixed(200.0))
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(300.0, 100.0));
    settle(&mut h, || build(0.0), 100);
    h.frame(build(1.0), Duration::from_millis(200));
    h.frame(build(1.0), Duration::from_millis(210));
    h.frame(build(1.0), Duration::from_millis(335));
    let mid = h.screenshot(build(1.0), &theme, 1.0);
    let ink_mid = ink(
        &mid,
        Rectangle::new(Point::new(20.0, 20.0), Size::new(200.0, 4.0)),
        rgb(c.surface_container_highest),
    )
    .unwrap();
    assert!(ink_mid.width > 20.0 && ink_mid.width < 190.0, "{ink_mid:?}");
    h.frame(build(1.0), Duration::from_millis(1000));
    let done = h.screenshot(build(1.0), &theme, 1.0);
    assert!(distance(done.at(Point::new(215.0, 21.0)), rgb(c.primary)) <= 1);
}

#[test]
fn linear_indeterminate_moves_along_the_track() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || place(linear(&theme, None).width(Length::Fixed(200.0)).into());
    let area = Rectangle::new(Point::new(20.0, 20.0), Size::new(200.0, 4.0));
    let mut h = Harness::new(Size::new(300.0, 100.0));
    h.frame(view(), Duration::ZERO);
    let at_start = h.screenshot(view(), &theme, 1.0);
    assert!(ink(&at_start, area, rgb(c.surface_container_highest)).is_none());
    let mut seen = Vec::new();
    for ms in [400, 800, 1000, 1200, 1500] {
        h.frame(view(), Duration::from_millis(ms));
        let image = h.screenshot(view(), &theme, 1.0);
        seen.push(ink(&image, area, rgb(c.surface_container_highest)));
    }
    assert!(seen.iter().any(Option::is_some));
    assert!(seen.windows(2).any(|w| w[0] != w[1]));
}

#[test]
fn circular_determinate_sweeps_clockwise_from_the_top() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || place(circular(&theme, Some(0.5)).into());
    let mut h = Harness::new(Size::new(120.0, 120.0));
    settle(&mut h, view, 2000);
    let image = h.screenshot(view(), &theme, 1.0);
    let center = Point::new(44.0, 44.0);
    assert!(
        distance(
            image.at(Point::new(center.x + 18.0, center.y)),
            rgb(c.primary)
        ) <= 1
    );
    assert!(
        distance(
            image.at(Point::new(center.x, center.y + 18.0)),
            rgb(c.primary)
        ) <= 1
    );
    assert!(
        distance(
            image.at(Point::new(center.x - 18.0, center.y)),
            rgb(c.surface)
        ) <= 1
    );
    assert!(
        distance(
            image.at(Point::new(center.x - 9.0, center.y - 15.6)),
            rgb(c.surface)
        ) <= 1
    );
    assert!(distance(image.at(center), rgb(c.surface)) <= 1);
}

#[test]
fn circular_indeterminate_draws_a_moving_arc() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || place(circular(&theme, None).into());
    let area = Rectangle::new(Point::new(20.0, 20.0), Size::new(48.0, 48.0));
    let mut h = Harness::new(Size::new(120.0, 120.0));
    h.frame(view(), Duration::ZERO);
    let mut seen = Vec::new();
    for ms in [100, 500, 1000, 2000, 3000] {
        h.frame(view(), Duration::from_millis(ms));
        let image = h.screenshot(view(), &theme, 1.0);
        seen.push(ink(&image, area, rgb(c.surface)));
    }
    assert!(seen.iter().all(Option::is_some));
    assert!(seen.windows(2).any(|w| w[0] != w[1]));
}
