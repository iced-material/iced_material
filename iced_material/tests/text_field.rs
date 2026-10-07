// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, ink, rgb, type_text};
use iced::widget::{container, text_editor};
use iced::{Length, Point, Rectangle, Size};
use material_iced::icon::symbol;
use material_iced::widget::focus_scope;
use material_iced::widget::pressable::Status;
use material_iced::widget::text_field::{self, Kind, Variant};
use material_iced::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Input(String),
    Clear,
}

fn place(content: Element<'_, Message>) -> Element<'_, Message> {
    focus_scope(container(content).padding(20)).into()
}

fn filled<'a>(theme: &Theme, value: &str) -> text_field::TextField<'a, Message> {
    text_field::filled(theme, value)
        .label("Label")
        .width(Length::Fixed(300.0))
        .on_input(Message::Input)
}

fn outlined<'a>(theme: &Theme, value: &str) -> text_field::TextField<'a, Message> {
    text_field::outlined(theme, value)
        .label("Label")
        .width(Length::Fixed(300.0))
        .on_input(Message::Input)
}

#[test]
fn geometry_matches_tokens() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let f = h.focusables(place(filled(&theme, "").into()));
    assert_eq!(
        f[0],
        Rectangle::new(Point::new(36.0, 44.0), Size::new(268.0, 24.0))
    );
    let o = h.focusables(place(outlined(&theme, "").into()));
    assert_eq!(
        o[0],
        Rectangle::new(Point::new(36.0, 36.0), Size::new(268.0, 24.0))
    );
    let icons = h.focusables(place(
        filled(&theme, "")
            .leading_icon(symbol::search(false))
            .trailing_icon(symbol::close(false))
            .into(),
    ));
    assert_eq!(icons[0].x, 20.0 + 12.0 + 24.0 + 16.0);
    assert_eq!(
        icons[0].width,
        300.0 - 12.0 - 24.0 - 16.0 - 16.0 - 24.0 - 12.0
    );
    let plain = h.focusables(place(
        text_field::filled(&theme, "")
            .width(Length::Fixed(300.0))
            .on_input(Message::Input)
            .into(),
    ));
    assert_eq!(plain[0].y, 36.0);
    let area_content = text_editor::Content::with_text("one\ntwo");
    let area = h.focusables(place(
        text_field::filled_area(&theme, &area_content)
            .label("Label")
            .width(Length::Fixed(300.0))
            .on_action(|_| Message::Clear)
            .into(),
    ));
    assert_eq!(area[0].size(), Size::new(268.0, 48.0));
}

#[test]
fn filled_colors_and_indicator() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let view = || place(filled(&theme, "").into());
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(
        distance(
            image.at(Point::new(300.0, 30.0)),
            rgb(c.surface_container_highest)
        ) <= 1
    );
    assert!(distance(image.at(Point::new(100.0, 75.0)), rgb(c.on_surface_variant)) <= 1);
    assert!(
        distance(
            image.at(Point::new(100.0, 74.0)),
            rgb(c.surface_container_highest)
        ) <= 1
    );
    h.click(view, Point::new(100.0, 50.0));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(16));
    h.frame(view(), Duration::from_millis(1000));
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(100.0, 75.0)), rgb(c.primary)) <= 1);
    assert!(distance(image.at(Point::new(100.0, 74.0)), rgb(c.primary)) <= 1);
}

#[test]
fn label_floats_when_focused_and_when_populated() {
    let theme = Theme::light();
    let c = theme.colors;
    let area = Rectangle::new(Point::new(30.0, 22.0), Size::new(120.0, 52.0));
    let bg = rgb(c.surface_container_highest);
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let view = || place(filled(&theme, "").into());
    let image = h.screenshot(view(), &theme, 1.0);
    let rest = ink(&image, area, bg).unwrap();
    assert!((rest.center().y - 48.0).abs() <= 3.0, "{rest:?}");
    h.click(view, Point::new(100.0, 50.0));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(16));
    h.frame(view(), Duration::from_millis(1000));
    let image = h.screenshot(view(), &theme, 1.0);
    let label = Rectangle::new(Point::new(30.0, 22.0), Size::new(120.0, 20.0));
    let floated = ink(&image, label, bg).unwrap();
    assert!((floated.center().y - 36.0).abs() <= 3.0, "{floated:?}");

    let mut h = Harness::new(Size::new(400.0, 200.0));
    let populated = h.screenshot(place(filled(&theme, "text").into()), &theme, 1.0);
    let floated = ink(&populated, label, bg).unwrap();
    assert!((floated.center().y - 36.0).abs() <= 3.0, "{floated:?}");
}

#[test]
fn outlined_outline_and_notch() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let view = || place(outlined(&theme, "").into());
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(200.0, 20.0)), rgb(c.outline)) <= 1);
    assert!(distance(image.at(Point::new(200.0, 21.0)), rgb(c.surface)) <= 1);
    assert!(distance(image.at(Point::new(40.0, 20.0)), rgb(c.outline)) <= 1);
    h.click(view, Point::new(100.0, 40.0));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(16));
    h.frame(view(), Duration::from_millis(1000));
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(200.0, 20.0)), rgb(c.primary)) <= 40);
    assert!(distance(image.at(Point::new(200.0, 21.0)), rgb(c.primary)) <= 40);
    assert!(distance(image.at(Point::new(40.0, 20.0)), rgb(c.surface)) <= 8);
    assert!(distance(image.at(Point::new(40.0, 21.0)), rgb(c.surface)) <= 8);
}

#[test]
fn typing_produces_input_and_max_length_truncates() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let view = || place(filled(&theme, "ab").max_length(3).into());
    h.click(view, Point::new(100.0, 50.0));
    assert_eq!(
        h.update(view(), &type_text("c")),
        vec![Message::Input("abc".to_string())]
    );
    let long = || place(filled(&theme, "abc").max_length(3).into());
    let mut h = Harness::new(Size::new(400.0, 200.0));
    h.click(long, Point::new(100.0, 50.0));
    assert_eq!(
        h.update(long(), &type_text("d")),
        vec![Message::Input("abc".to_string())]
    );
}

#[test]
fn clicking_the_padding_focuses_the_input() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let view = || place(filled(&theme, "").into());
    h.click(view, Point::new(25.0, 70.0));
    assert_eq!(
        h.update(view(), &type_text("x")),
        vec![Message::Input("x".to_string())]
    );
}

#[test]
fn trailing_icon_press_publishes_and_is_a_tab_stop() {
    let theme = Theme::light();
    let view = || {
        place(
            filled(&theme, "x")
                .trailing_icon(symbol::close(false))
                .on_trailing_icon_press(Message::Clear)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let stops = h.focusables(view());
    assert_eq!(stops.len(), 2);
    let center = stops[1].center();
    assert_eq!(h.click(view, center), vec![Message::Clear]);
}

#[test]
fn error_and_disabled_styles() {
    let theme = Theme::light();
    let c = theme.colors;
    let v = |kind, error| Variant { kind, error };
    let ok = text_field::style(&theme, Status::Active, v(Kind::Filled, false));
    assert_eq!(
        (ok.indicator, ok.label, ok.supporting_text),
        (
            c.on_surface_variant,
            c.on_surface_variant,
            c.on_surface_variant
        )
    );
    let hover = text_field::style(&theme, Status::Hovered, v(Kind::Filled, false));
    assert_eq!(
        (hover.indicator, hover.label),
        (c.on_surface, c.on_surface_variant)
    );
    let hover_outlined = text_field::style(&theme, Status::Hovered, v(Kind::Outlined, false));
    assert_eq!(
        (hover_outlined.outline, hover_outlined.label),
        (c.on_surface, c.on_surface)
    );
    let focus = text_field::style(&theme, Status::Focused, v(Kind::Outlined, false));
    assert_eq!(
        (focus.outline, focus.outline_width, focus.label),
        (c.primary, 2.0, c.primary)
    );
    let error = text_field::style(&theme, Status::Active, v(Kind::Filled, true));
    assert_eq!(
        (
            error.indicator,
            error.label,
            error.trailing_icon,
            error.supporting_text
        ),
        (c.error, c.error, c.error, c.error)
    );
    assert_eq!(error.leading_icon, c.on_surface_variant);
    let error_hover = text_field::style(&theme, Status::Hovered, v(Kind::Outlined, true));
    assert_eq!(
        (error_hover.outline, error_hover.label),
        (c.on_error_container, c.on_error_container)
    );
    let disabled = text_field::style(&theme, Status::Disabled, v(Kind::Filled, false));
    assert_eq!(disabled.container.a, 0.04);
    assert_eq!(disabled.label.a, theme.disabled.content);
}

#[test]
fn supporting_text_and_counter_extend_the_field() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 200.0));
    let image = h.screenshot(
        place(
            filled(&theme, "ab")
                .supporting_text("Help")
                .max_length(10)
                .into(),
        ),
        &theme,
        1.0,
    );
    let help = ink(
        &image,
        Rectangle::new(Point::new(30.0, 78.0), Size::new(150.0, 20.0)),
        rgb(theme.colors.surface),
    );
    assert!(help.is_some());
    let counter = ink(
        &image,
        Rectangle::new(Point::new(250.0, 78.0), Size::new(70.0, 20.0)),
        rgb(theme.colors.surface),
    )
    .unwrap();
    assert!(counter.x + counter.width <= 20.0 + 300.0 - 16.0 + 1.0);
    assert!(counter.x + counter.width >= 20.0 + 300.0 - 16.0 - 3.0);
}
