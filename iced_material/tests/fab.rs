// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use common::{Harness, distance, rgb};
use iced::widget::container;
use iced::{Point, Size};
use iced_material::draw::text::Label;
use iced_material::icon::symbol;
use iced_material::widget::fab::{self, Color};
use iced_material::widget::pressable::Status;
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Pressed,
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content).padding(20).into()
}

#[test]
fn sizes_match_tokens() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(300.0, 200.0));
    let cases: [(fab::Fab<'static, Message>, Size); 3] = [
        (
            fab::small(&theme, symbol::add(false)),
            Size::new(40.0, 40.0),
        ),
        (fab::fab(&theme, symbol::add(false)), Size::new(56.0, 56.0)),
        (
            fab::large(&theme, symbol::add(false)),
            Size::new(96.0, 96.0),
        ),
    ];
    for (fab, size) in cases {
        let bounds = h.focusables(place(fab.on_press(Message::Pressed).into()));
        assert_eq!(bounds[0].size(), size);
    }
    let mut label = Label::default();
    let width = label.update("Compose", theme.typography.label_large).width;
    let extended = h.focusables(place(
        fab::extended(&theme, "Compose")
            .icon(symbol::edit(false))
            .on_press(Message::Pressed)
            .into(),
    ));
    assert!((extended[0].width - (16.0 + 24.0 + 12.0 + width + 20.0)).abs() < 0.01);
    assert_eq!(extended[0].height, 56.0);
    let plain = h.focusables(place(
        fab::extended(&theme, "Compose")
            .on_press(Message::Pressed)
            .into(),
    ));
    assert!((plain[0].width - (20.0 + width + 20.0)).abs() < 0.01);
}

#[test]
fn color_sets_and_elevation_follow_tokens() {
    let theme = Theme::light();
    let c = theme.colors;
    let cases = [
        (Color::Primary, c.primary_container, c.on_primary_container),
        (
            Color::Secondary,
            c.secondary_container,
            c.on_secondary_container,
        ),
        (
            Color::Tertiary,
            c.tertiary_container,
            c.on_tertiary_container,
        ),
        (Color::Surface, c.surface_container_high, c.primary),
    ];
    for (color, container, icon) in cases {
        let style = fab::style(&theme, Status::Active, color, false);
        assert_eq!(
            (style.container, style.icon, style.elevation),
            (container, icon, 3.0)
        );
        assert_eq!(
            fab::style(&theme, Status::Hovered, color, false).elevation,
            4.0
        );
        assert_eq!(
            fab::style(&theme, Status::Pressed, color, false).elevation,
            3.0
        );
        assert_eq!(
            fab::style(&theme, Status::Hovered, color, true).elevation,
            2.0
        );
    }
    assert_eq!(
        fab::style(&theme, Status::Active, Color::Surface, true).container,
        c.surface_container_low
    );
}

#[test]
fn regular_fab_has_large_corners() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(200.0, 200.0));
    let image = h.screenshot(
        place(
            fab::fab(&theme, symbol::add(false))
                .on_press(Message::Pressed)
                .into(),
        ),
        &theme,
        1.0,
    );
    let corner = image.at(Point::new(21.0, 21.0));
    assert!(
        distance(corner, rgb(theme.colors.surface))
            < distance(corner, rgb(theme.colors.primary_container))
    );
    assert!(
        distance(
            image.at(Point::new(30.0, 30.0)),
            rgb(theme.colors.primary_container)
        ) <= 1
    );
}
