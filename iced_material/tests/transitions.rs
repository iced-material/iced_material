// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, ink, rgb, solid};
use iced::widget::{Space, container};
use iced::{Color, Length, Point, Rectangle, Size};
use material_iced::widget::transition::{
    Axis, container_transform, fade, fade_through, shared_axis,
};
use material_iced::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Done,
}

fn filled(color: Color) -> Element<'static, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(color.into()),
            ..container::Style::default()
        })
        .into()
}

fn block(color: Color) -> Element<'static, Message> {
    container(
        container(Space::new())
            .width(100)
            .height(Length::Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(color.into()),
                ..container::Style::default()
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn size() -> Size {
    Size::new(400.0, 300.0)
}

fn play(
    h: &mut Harness,
    view: impl Fn() -> Element<'static, Message>,
    times: &[u64],
) -> Vec<Message> {
    let mut messages = Vec::new();
    for ms in times {
        messages.extend(h.frame(view(), Duration::from_millis(*ms)));
    }
    messages
}

#[test]
fn fade_through_swaps_content_and_reports_once() {
    let theme = Theme::light();
    let c = theme.colors;
    let (a, b) = (c.tertiary_container, c.primary_container);
    let view = || {
        fade_through(&theme, c.surface, filled(b), Some(filled(a)))
            .on_finished(Message::Done)
            .into()
    };
    let mut h = Harness::new(size());
    let probe = Point::new(200.0, 150.0);
    let mut messages = play(&mut h, view, &[0, 16]);
    assert!(distance(h.screenshot(view(), &theme, 1.0).at(probe), rgb(a)) <= 6);
    let mut closest = u32::MAX;
    for ms in (30..440).step_by(10) {
        messages.extend(h.frame(view(), Duration::from_millis(ms)));
        closest = closest.min(distance(
            h.screenshot(view(), &theme, 1.0).at(probe),
            rgb(c.surface),
        ));
    }
    assert!(closest <= 12, "{closest}");
    assert!(messages.is_empty());
    messages.extend(play(&mut h, view, &[460, 500, 600]));
    assert_eq!(messages, vec![Message::Done]);
    assert_eq!(h.screenshot(view(), &theme, 1.0).at(probe), rgb(b));
}

#[test]
fn without_a_previous_view_only_the_current_one_is_drawn() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || fade_through(&theme, c.surface, filled(c.primary_container), None).into();
    let mut h = Harness::new(size());
    h.frame(view(), Duration::ZERO);
    assert_eq!(
        h.screenshot(view(), &theme, 1.0).at(Point::new(10.0, 10.0)),
        rgb(c.primary_container)
    );
}

fn shared(theme: &Theme, axis: Axis, forward: bool) -> Element<'static, Message> {
    let c = theme.colors;
    shared_axis(
        theme,
        c.surface,
        axis,
        forward,
        block(c.primary),
        Some(block(c.tertiary)),
    )
    .on_finished(Message::Done)
    .into()
}

fn fading(theme: &Theme, visible: bool) -> Element<'static, Message> {
    fade(
        theme,
        theme.colors.surface,
        filled(theme.colors.primary),
        visible,
    )
    .into()
}

#[test]
fn shared_axis_slides_the_new_content_into_place() {
    let theme = Theme::light();
    let c = theme.colors;
    let area = Rectangle::with_size(size());
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let mut h = Harness::new(size());
        play(&mut h, || shared(&theme, axis, true), &[0, 16]);
        let mut seen = false;
        for ms in (30..440).step_by(10) {
            h.frame(shared(&theme, axis, true), Duration::from_millis(ms));
            let image = h.screenshot(shared(&theme, axis, true), &theme, 1.0);
            if solid(&image, area, rgb(c.primary)).is_some() {
                seen = true;
                break;
            }
        }
        assert!(seen, "{axis:?}");
        play(&mut h, || shared(&theme, axis, true), &[500, 600]);
        let end = h.screenshot(shared(&theme, axis, true), &theme, 1.0);
        let block = solid(&end, area, rgb(c.primary)).unwrap();
        assert_eq!((block.x, block.width), (0.0, 100.0), "{axis:?}");
    }
}

#[test]
fn shared_axis_x_direction_decides_the_side_the_content_comes_from() {
    let theme = Theme::light();
    let c = theme.colors;
    let area = Rectangle::with_size(size());
    let mut forward = Harness::new(size());
    play(
        &mut forward,
        || shared(&theme, Axis::X, true),
        &[0, 16, 150],
    );
    let image = forward.screenshot(shared(&theme, Axis::X, true), &theme, 1.0);
    let incoming = ink(&image, area, rgb(c.surface)).unwrap();
    assert!(incoming.x > 0.0, "{incoming:?}");
    let mut backward = Harness::new(size());
    play(
        &mut backward,
        || shared(&theme, Axis::X, false),
        &[0, 16, 150],
    );
    let image = backward.screenshot(shared(&theme, Axis::X, false), &theme, 1.0);
    let incoming = ink(&image, area, rgb(c.surface)).unwrap();
    assert_eq!(incoming.x, 0.0);
    assert!(incoming.width < 100.0, "{incoming:?}");
}

#[test]
fn fade_hides_and_shows_with_the_flag() {
    let theme = Theme::light();
    let c = theme.colors;
    let probe = Point::new(100.0, 100.0);
    let mut h = Harness::new(size());
    play(&mut h, || fading(&theme, false), &[0, 16]);
    assert_eq!(
        h.screenshot(fading(&theme, false), &theme, 1.0).at(probe),
        rgb(c.surface)
    );
    play(&mut h, || fading(&theme, true), &[100, 110]);
    assert_ne!(
        h.screenshot(fading(&theme, true), &theme, 1.0).at(probe),
        rgb(c.primary)
    );
    play(&mut h, || fading(&theme, true), &[600, 700]);
    assert_eq!(
        h.screenshot(fading(&theme, true), &theme, 1.0).at(probe),
        rgb(c.primary)
    );
    play(&mut h, || fading(&theme, false), &[800, 810, 1100, 1200]);
    assert_eq!(
        h.screenshot(fading(&theme, false), &theme, 1.0).at(probe),
        rgb(c.surface)
    );
}

#[test]
fn container_transform_grows_from_the_anchor_to_the_window() {
    let theme = Theme::light();
    let c = theme.colors;
    let anchor = || -> Element<'static, Message> {
        container(
            container(Space::new())
                .width(100)
                .height(60)
                .style(move |_: &Theme| container::Style {
                    background: Some(c.secondary_container.into()),
                    ..container::Style::default()
                }),
        )
        .padding(50)
        .into()
    };
    let view = |open: bool| {
        container_transform(anchor(), filled(c.primary_container), open)
            .color(c.primary_container)
            .build(&Theme::light())
    };
    let area = Rectangle::with_size(size());
    let mut h = Harness::new(size());
    let closed = || view(false);
    play(&mut h, closed, &[0, 16]);
    assert!(
        solid(
            &h.screenshot(closed(), &theme, 1.0),
            area,
            rgb(c.primary_container)
        )
        .is_none()
    );
    let open = || view(true);
    play(&mut h, open, &[100, 110, 230]);
    let mid = solid(
        &h.screenshot(open(), &theme, 1.0),
        area,
        rgb(c.primary_container),
    )
    .unwrap();
    assert!(mid.width > 100.0 && mid.width < 400.0, "{mid:?}");
    assert!(mid.height > 60.0 && mid.height < 300.0, "{mid:?}");
    play(&mut h, open, &[1000, 1100]);
    let end = h.screenshot(open(), &theme, 1.0);
    assert_eq!(end.at(Point::new(2.0, 2.0)), rgb(c.primary_container));
    assert_eq!(end.at(Point::new(398.0, 298.0)), rgb(c.primary_container));
    play(&mut h, closed, &[1200, 1210, 1700, 1800]);
    assert!(
        solid(
            &h.screenshot(closed(), &theme, 1.0),
            area,
            rgb(c.primary_container)
        )
        .is_none()
    );
}

fn labelled(theme: &Theme, visible: bool) -> Element<'static, Message> {
    fade(
        theme,
        theme.colors.surface,
        container(iced::widget::text("Fading text").size(32)).padding(20),
        visible,
    )
    .into()
}

#[test]
fn fade_hides_text_as_well_as_fills() {
    let theme = Theme::light();
    let c = theme.colors;
    let area = Rectangle::with_size(size());
    let mut h = Harness::new(size());
    play(&mut h, || labelled(&theme, true), &[0, 16, 600, 700]);
    let shown = h.screenshot(labelled(&theme, true), &theme, 1.0);
    assert!(ink(&shown, area, rgb(c.surface)).is_some());
    play(&mut h, || labelled(&theme, false), &[800, 810, 945]);
    let almost_gone = h.screenshot(labelled(&theme, false), &theme, 1.0);
    assert!(ink(&almost_gone, area, rgb(c.surface)).is_none());
}
