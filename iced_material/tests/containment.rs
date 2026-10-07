// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use common::{Harness, distance, rgb};
use iced::widget::{column, container, space};
use iced::{Length, Point, Size};
use iced_material::draw::text::Label;
use iced_material::icon::{icon, symbol};
use iced_material::widget::badge::{self, badged};
use iced_material::widget::card::{self, Kind};
use iced_material::widget::divider;
use iced_material::widget::pressable::Status;
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Open,
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content).padding(20).into()
}

#[test]
fn card_styles_follow_tokens() {
    let theme = Theme::light();
    let c = theme.colors;
    let e = |status| card::style(&theme, status, Kind::Elevated);
    assert_eq!(
        (e(Status::Active).container, e(Status::Active).elevation),
        (c.surface_container_low, 1.0)
    );
    assert_eq!(e(Status::Hovered).elevation, 2.0);
    assert_eq!(e(Status::Dragged).elevation, 4.0);
    let f = |status| card::style(&theme, status, Kind::Filled);
    assert_eq!(
        (f(Status::Active).container, f(Status::Hovered).elevation),
        (c.surface_container_highest, 1.0)
    );
    let o = |status| card::style(&theme, status, Kind::Outlined);
    assert_eq!(
        (o(Status::Active).outline, o(Status::Active).outline_width),
        (c.outline_variant, 1.0)
    );
    assert_eq!(o(Status::Focused).outline, c.on_surface);
}

#[test]
fn pressable_card_publishes_and_draws_medium_corners() {
    let theme = Theme::light();
    let view = || {
        place(
            card::filled(&theme, space().width(200).height(100))
                .on_press(Message::Open)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(300.0, 200.0));
    let bounds = h.focusables(view());
    assert_eq!(bounds[0].size(), Size::new(200.0, 100.0));
    assert_eq!(h.click(view, Point::new(100.0, 60.0)), vec![Message::Open]);
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(20.5, 20.5)), rgb(theme.colors.surface)) <= 2);
    assert!(
        distance(
            image.at(Point::new(40.0, 40.0)),
            rgb(theme.colors.surface_container_highest)
        ) <= 1
    );
}

#[test]
fn divider_is_one_outline_variant_line() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(200.0, 100.0));
    let view: Element<'static, Message> = place(
        column![
            space().height(10),
            divider::horizontal(&theme).inset(16.0, 0.0),
            space().height(10)
        ]
        .width(Length::Fill)
        .into(),
    );
    let image = h.screenshot(view, &theme, 1.0);
    assert!(
        distance(
            image.at(Point::new(60.0, 30.5)),
            rgb(theme.colors.outline_variant)
        ) <= 1
    );
    assert!(distance(image.at(Point::new(30.0, 30.5)), rgb(theme.colors.surface)) <= 1);
    assert!(distance(image.at(Point::new(60.0, 31.5)), rgb(theme.colors.surface)) <= 1);
}

#[test]
fn badges_sit_on_the_top_end_of_the_anchor() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(200.0, 100.0));
    let anchor = || icon(symbol::menu(false), 24.0, theme.colors.on_surface_variant);
    let small: Element<'static, Message> = place(badged(anchor(), badge::small(&theme)).into());
    let image = h.screenshot(small, &theme, 1.0);
    assert!(
        distance(
            image.at(Point::new(20.0 + 12.0 + 6.0 + 3.0, 20.0 + 4.0 + 3.0)),
            rgb(theme.colors.error)
        ) <= 1
    );
    let large: Element<'static, Message> =
        place(badged(anchor(), badge::large(&theme, "999+")).into());
    let image = h.screenshot(large, &theme, 1.0);
    let width = Label::default()
        .update("999+", theme.typography.label_small)
        .width
        + 8.0;
    let left = 20.0 + 12.0 + 2.0;
    assert!(
        distance(
            image.at(Point::new(left + 1.5, 20.0 + 1.0 + 8.0)),
            rgb(theme.colors.error)
        ) <= 2
    );
    assert!(
        distance(
            image.at(Point::new(left + width - 1.5, 20.0 + 1.0 + 8.0)),
            rgb(theme.colors.error)
        ) <= 2
    );
}
