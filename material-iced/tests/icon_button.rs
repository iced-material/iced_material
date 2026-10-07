// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, rgb};
use iced::widget::container;
use iced::{Point, Size};
use material_iced::icon::symbol;
use material_iced::state::overlay;
use material_iced::widget::icon_button::{self, Toggle};
use material_iced::widget::pressable::Status;
use material_iced::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Pressed,
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content).padding(20).into()
}

#[test]
fn geometry_and_target() {
    let theme = Theme::light();
    let view = || {
        place(
            icon_button::standard(&theme, symbol::menu(false))
                .on_press(Message::Pressed)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(200.0, 120.0));
    let bounds = h.focusables(view());
    assert_eq!(bounds[0].size(), Size::new(40.0, 40.0));
    assert_eq!(
        h.click(view, Point::new(17.0, 17.0)),
        vec![Message::Pressed]
    );
    assert!(h.click(view, Point::new(15.0, 15.0)).is_empty());
}

#[test]
fn toggle_styles_follow_tokens() {
    let theme = Theme::light();
    let c = theme.colors;
    let filled = icon_button::filled_style(&theme, Status::Active, Toggle::Unselected);
    assert_eq!(
        (filled.container, filled.icon),
        (c.surface_container_highest, c.primary)
    );
    let filled = icon_button::filled_style(&theme, Status::Active, Toggle::Selected);
    assert_eq!((filled.container, filled.icon), (c.primary, c.on_primary));
    let tonal = icon_button::filled_tonal_style(&theme, Status::Active, Toggle::Unselected);
    assert_eq!(
        (tonal.container, tonal.icon),
        (c.surface_container_highest, c.on_surface_variant)
    );
    let outlined = icon_button::outlined_style(&theme, Status::Active, Toggle::Selected);
    assert_eq!(
        (outlined.container, outlined.icon, outlined.outline_width),
        (c.inverse_surface, c.inverse_on_surface, 0.0)
    );
    let outlined = icon_button::outlined_style(&theme, Status::Active, Toggle::Unselected);
    assert_eq!(
        (outlined.hover_layer, outlined.pressed_layer),
        (c.on_surface_variant, c.on_surface)
    );
    let standard = icon_button::standard_style(&theme, Status::Active, Toggle::Selected);
    assert_eq!(standard.icon, c.primary);
}

#[test]
fn hover_layer_is_drawn_on_the_state_layer_circle() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        place(
            icon_button::filled(&theme, symbol::menu(false))
                .on_press(Message::Pressed)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(200.0, 120.0));
    h.move_cursor(view(), Point::new(40.0, 40.0));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(100));
    let image = h.screenshot(view(), &theme, 1.0);
    let expected = overlay(c.primary, c.on_primary, theme.state.hover);
    assert!(distance(image.at(Point::new(40.0, 24.0)), rgb(expected)) <= 1);
    assert!(distance(image.at(Point::new(21.0, 21.0)), rgb(c.surface)) <= 1);
}
