// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use common::{Harness, distance, rgb};
use iced::widget::{column, container};
use iced::{Length, Point, Size};
use iced_material::icon::symbol;
use iced_material::widget::checkbox::checkbox;
use iced_material::widget::focus_scope;
use iced_material::widget::list::{Leading, Trailing, list_item};
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Press(usize),
    Check(bool),
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    focus_scope(container(content).padding(20).width(Length::Fixed(360.0))).into()
}

#[test]
fn heights_follow_the_line_count() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 400.0));
    let view = place(
        column![
            list_item(&theme, "One line").on_press(Message::Press(0)),
            list_item(&theme, "Two lines")
                .supporting_text("Supporting text")
                .on_press(Message::Press(1)),
            list_item(&theme, "Three lines")
                .overline("Overline")
                .supporting_text("Supporting text")
                .on_press(Message::Press(2)),
        ]
        .into(),
    );
    let bounds = h.focusables(view);
    let heights: Vec<f32> = bounds.iter().map(|b| b.height).collect();
    assert_eq!(heights, vec![56.0, 72.0, 88.0]);
    assert!(bounds.iter().all(|b| b.width == 320.0));
}

#[test]
fn colors_and_leading_avatar() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let view = || {
        place(
            column![
                list_item(&theme, "Avatar")
                    .leading(Leading::Avatar("A".into()))
                    .trailing(Trailing::Text("12:30".into()))
                    .on_press(Message::Press(0)),
                list_item(&theme, "Selected")
                    .leading(Leading::Icon(symbol::search(false)))
                    .selected(true)
                    .on_press(Message::Press(1)),
            ]
            .into(),
        )
    };
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(300.0, 24.0)), rgb(c.surface)) <= 1);
    let avatar = Point::new(20.0 + 16.0 + 3.0, 20.0 + 28.0);
    assert!(distance(image.at(avatar), rgb(c.primary_container)) <= 1);
    assert!(
        distance(
            image.at(Point::new(200.0, 20.0 + 56.0 + 28.0)),
            rgb(c.secondary_container)
        ) <= 1
    );
}

#[test]
fn press_publishes_and_disabled_does_not() {
    let theme = Theme::light();
    let view = || {
        place(
            column![
                list_item(&theme, "Active").on_press(Message::Press(0)),
                list_item(&theme, "Disabled")
                    .disabled(true)
                    .on_press(Message::Press(1)),
            ]
            .into(),
        )
    };
    let mut h = Harness::new(Size::new(400.0, 200.0));
    assert_eq!(h.focusables(view()).len(), 1);
    assert_eq!(
        h.click(view, Point::new(100.0, 40.0)),
        vec![Message::Press(0)]
    );
    assert!(
        h.click(view, Point::new(100.0, 20.0 + 56.0 + 28.0))
            .is_empty()
    );
}

#[test]
fn trailing_widget_receives_events() {
    let theme = Theme::light();
    let view = || {
        place(
            list_item(&theme, "Check")
                .trailing(Trailing::Element(
                    checkbox(&theme, false).on_toggle(Message::Check).into(),
                ))
                .on_press(Message::Press(0))
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let stops = h.focusables(view());
    assert_eq!(stops.len(), 2);
    let target = stops[1].center();
    assert_eq!(h.click(view, target), vec![Message::Check(true)]);
}
