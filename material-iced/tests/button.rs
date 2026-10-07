// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, key, rgb};
use iced::keyboard::key::Named;
use iced::widget::container;
use iced::{Point, Size};
use material_iced::draw::text::Label;
use material_iced::icon::symbol;
use material_iced::state::{alpha, overlay};
use material_iced::widget::{button, focus_scope};
use material_iced::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Pressed,
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    focus_scope(container(content).padding(20)).into()
}

fn label_width(theme: &Theme, text: &str) -> f32 {
    let mut label = Label::default();
    label.update(text, theme.typography.label_large).width
}

#[test]
fn geometry_matches_tokens() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let label = label_width(&theme, "Button");
    let cases: [(button::Button<'static, Message>, f32); 4] = [
        (
            button::filled(&theme, "Button").on_press(Message::Pressed),
            48.0 + label,
        ),
        (
            button::filled(&theme, "Button")
                .leading_icon(symbol::add(false))
                .on_press(Message::Pressed),
            16.0 + 18.0 + 8.0 + label + 24.0,
        ),
        (
            button::outlined(&theme, "Button")
                .trailing_icon(symbol::add(false))
                .on_press(Message::Pressed),
            24.0 + label + 8.0 + 18.0 + 16.0,
        ),
        (
            button::text(&theme, "Button").on_press(Message::Pressed),
            24.0 + label,
        ),
    ];
    for (button, width) in cases {
        let bounds = h.focusables(place(button.into()));
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].height, 40.0);
        assert!(
            (bounds[0].width - width.max(64.0)).abs() < 0.01,
            "{} vs {width}",
            bounds[0].width
        );
    }
    let short = h.focusables(place(
        button::text(&theme, "OK").on_press(Message::Pressed).into(),
    ));
    assert_eq!(short[0].width, 64.0);
}

#[test]
fn colors_follow_state() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        place(
            button::filled(&theme, "Button")
                .on_press(Message::Pressed)
                .into(),
        )
    };
    let probe = Point::new(23.0, 40.0);
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let mut h = Harness::new(Size::new(300.0, 120.0));
        let idle = h.screenshot(view(), &theme, scale);
        assert!(
            distance(idle.at(probe), rgb(c.primary)) <= 1,
            "scale {scale}"
        );
        h.move_cursor(view(), Point::new(60.0, 40.0));
        h.frame(view(), Duration::ZERO);
        h.frame(view(), Duration::from_millis(100));
        let hovered = h.screenshot(view(), &theme, scale);
        let expected = overlay(c.primary, c.on_primary, theme.state.hover);
        assert!(
            distance(hovered.at(probe), rgb(expected)) <= 1,
            "scale {scale}: {:?} vs {:?}",
            hovered.at(probe),
            rgb(expected)
        );
    }
}

#[test]
fn disabled_uses_on_surface_opacities() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(300.0, 120.0));
    let image = h.screenshot(place(button::filled(&theme, "Button").into()), &theme, 1.0);
    let expected = overlay(c.surface, c.on_surface, theme.disabled.container);
    assert!(distance(image.at(Point::new(23.0, 40.0)), rgb(expected)) <= 1);
    assert!(
        h.focusables(place(button::filled(&theme, "Button").into()))
            .is_empty()
    );
    let _ = alpha(c.on_surface, theme.disabled.content);
}

#[test]
fn pointer_press_publishes_once_and_ripples() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        place(
            button::filled(&theme, "Button")
                .on_press(Message::Pressed)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(300.0, 120.0));
    h.frame(view(), Duration::ZERO);
    let mut messages = h.click(view, Point::new(60.0, 40.0));
    messages.extend(h.frame(view(), Duration::from_millis(16)));
    assert_eq!(messages, vec![Message::Pressed]);
    h.frame(view(), Duration::from_millis(120));
    let image = h.screenshot(view(), &theme, 1.0);
    let pressed = image.at(Point::new(60.0, 30.0));
    assert_ne!(
        pressed,
        rgb(overlay(c.primary, c.on_primary, theme.state.hover))
    );
    assert_ne!(pressed, rgb(c.primary));
    h.frame(view(), Duration::from_millis(2000));
    h.frame(view(), Duration::from_millis(2500));
    let settled = h.screenshot(view(), &theme, 1.0);
    let hover = overlay(c.primary, c.on_primary, theme.state.hover);
    assert!(distance(settled.at(Point::new(60.0, 30.0)), rgb(hover)) <= 1);
}

#[test]
fn press_outside_the_target_does_nothing() {
    let theme = Theme::light();
    let view = || {
        place(
            button::filled(&theme, "Button")
                .on_press(Message::Pressed)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(300.0, 160.0));
    assert!(h.click(view, Point::new(250.0, 40.0)).is_empty());
    assert_eq!(
        h.click(view, Point::new(60.0, 17.0)),
        vec![Message::Pressed]
    );
}

#[test]
fn keyboard_focus_and_activation() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        place(
            button::filled(&theme, "Button")
                .on_press(Message::Pressed)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(300.0, 120.0));
    h.update(view(), &key(Named::Tab, false));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(700));
    let image = h.screenshot(view(), &theme, 1.0);
    let ring = image.at(Point::new(60.0, 20.0 - 2.0 - 1.5));
    assert!(distance(ring, rgb(c.secondary)) <= 2, "{ring:?}");
    assert_eq!(
        h.update(view(), &key(Named::Enter, false)),
        vec![Message::Pressed]
    );
    assert_eq!(
        h.update(view(), &key(Named::Space, false)),
        vec![Message::Pressed]
    );
}

#[test]
fn mouse_focus_shows_no_ring() {
    let theme = Theme::light();
    let view = || {
        place(
            button::filled(&theme, "Button")
                .on_press(Message::Pressed)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(300.0, 120.0));
    h.click(view, Point::new(60.0, 40.0));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(700));
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(60.0, 16.5)), rgb(theme.colors.surface)) <= 1);
}

#[test]
fn elevated_button_casts_a_shadow() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(300.0, 120.0));
    let image = h.screenshot(
        place(
            button::elevated(&theme, "Button")
                .on_press(Message::Pressed)
                .into(),
        ),
        &theme,
        1.0,
    );
    let below = image.at(Point::new(60.0, 61.0));
    assert!(distance(below, rgb(theme.colors.surface)) >= 4, "{below:?}");
}
