// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, key, rgb};
use iced::keyboard::key::Named;
use iced::widget::container;
use iced::{Point, Size};
use material_iced::state::overlay;
use material_iced::widget::checkbox::{self, Check, Variant, checkbox};
use material_iced::widget::focus_scope;
use material_iced::widget::pressable::Status;
use material_iced::widget::radio::{self, radio_group};
use material_iced::widget::switch::{self, switch};
use material_iced::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Toggle(bool),
    Select(usize),
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    focus_scope(container(content).padding(20)).into()
}

fn checkbox_view(theme: &Theme, checked: bool) -> Element<'static, Message> {
    place(checkbox(theme, checked).on_toggle(Message::Toggle).into())
}

#[test]
fn geometry_matches_tokens() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(300.0, 200.0));
    let checkbox = h.focusables(checkbox_view(&theme, false));
    assert_eq!(checkbox[0].size(), Size::new(40.0, 40.0));
    let switch = h.focusables(place(
        switch(&theme, false).on_toggle(Message::Toggle).into(),
    ));
    assert_eq!(switch[0].size(), Size::new(52.0, 32.0));
    let radio = h.focusables(place(
        radio_group(&theme, ["A", "B", "C"], Some(1))
            .on_select(Message::Select)
            .into(),
    ));
    assert_eq!(radio.len(), 1);
    assert_eq!(radio[0].size(), Size::new(40.0, 40.0));
    assert_eq!(radio[0].y, 20.0 + 48.0 + 4.0);
}

#[test]
fn checkbox_styles_follow_tokens() {
    let theme = Theme::light();
    let c = theme.colors;
    let v = |check, error| Variant { check, error };
    let off = checkbox::style(&theme, Status::Active, v(Check::Unchecked, false));
    assert_eq!((off.container.a, off.outline), (0.0, c.on_surface_variant));
    let on = checkbox::style(&theme, Status::Active, v(Check::Checked, false));
    assert_eq!((on.container, on.icon), (c.primary, c.on_primary));
    let hover = checkbox::style(&theme, Status::Hovered, v(Check::Unchecked, false));
    assert_eq!(
        (hover.outline, hover.hover_layer, hover.pressed_layer),
        (c.on_surface, c.on_surface, c.primary)
    );
    let error = checkbox::style(&theme, Status::Active, v(Check::Checked, true));
    assert_eq!(
        (error.container, error.icon, error.hover_layer),
        (c.error, c.on_error, c.error)
    );
    let disabled = checkbox::style(&theme, Status::Disabled, v(Check::Checked, false));
    assert_eq!(disabled.icon, c.surface);
}

#[test]
fn checkbox_toggles_and_animates_to_the_fill() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(200.0, 120.0));
    let center = Point::new(44.0, 44.0);
    assert_eq!(
        h.click(|| checkbox_view(&theme, false), center),
        vec![Message::Toggle(true)]
    );
    assert_eq!(
        h.click(|| checkbox_view(&theme, true), center),
        vec![Message::Toggle(false)]
    );
    assert!(
        h.click(|| checkbox_view(&theme, false), Point::new(5.0, 5.0))
            .is_empty()
    );
    let mut h = Harness::new(Size::new(200.0, 120.0));
    let probe = Point::new(37.0, 37.0);
    let unchecked = h.screenshot(checkbox_view(&theme, false), &theme, 1.0);
    assert!(distance(unchecked.at(probe), rgb(theme.colors.surface)) <= 1);
    h.frame(checkbox_view(&theme, true), Duration::ZERO);
    h.frame(checkbox_view(&theme, true), Duration::from_millis(16));
    h.frame(checkbox_view(&theme, true), Duration::from_millis(1000));
    let checked = h.screenshot(checkbox_view(&theme, true), &theme, 1.0);
    assert!(distance(checked.at(probe), rgb(theme.colors.primary)) <= 1);
}

#[test]
fn checkbox_keyboard_activation() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(200.0, 120.0));
    h.update(checkbox_view(&theme, false), &key(Named::Tab, false));
    assert_eq!(
        h.update(checkbox_view(&theme, false), &key(Named::Space, false)),
        vec![Message::Toggle(true)]
    );
    let disabled = place(checkbox(&theme, false).into());
    assert!(h.focusables(disabled).is_empty());
}

#[test]
fn switch_styles_follow_tokens() {
    let theme = Theme::light();
    let c = theme.colors;
    let on = switch::style(&theme, Status::Active, true);
    assert_eq!(
        (on.track, on.handle, on.outline.a),
        (c.primary, c.on_primary, 0.0)
    );
    assert_eq!(
        switch::style(&theme, Status::Pressed, true).handle,
        c.primary_container
    );
    let off = switch::style(&theme, Status::Active, false);
    assert_eq!(
        (off.track, off.outline, off.handle),
        (c.surface_container_highest, c.outline, c.outline)
    );
    assert_eq!(
        switch::style(&theme, Status::Hovered, false).handle,
        c.on_surface_variant
    );
    let disabled = switch::style(&theme, Status::Disabled, true);
    assert_eq!(disabled.handle, c.surface);
}

#[test]
fn switch_handle_moves_to_the_end_of_the_track() {
    let theme = Theme::light();
    let view = |selected| place(switch(&theme, selected).on_toggle(Message::Toggle).into());
    let mut h = Harness::new(Size::new(200.0, 120.0));
    let off = h.screenshot(view(false), &theme, 1.0);
    let handle_off = off.at(Point::new(20.0 + 16.0, 20.0 + 24.0));
    assert!(distance(handle_off, rgb(theme.colors.outline)) <= 1);
    assert_eq!(
        h.click(|| view(false), Point::new(46.0, 36.0)),
        vec![Message::Toggle(true)]
    );
    let mut h = Harness::new(Size::new(200.0, 120.0));
    h.frame(view(true), Duration::ZERO);
    h.frame(view(true), Duration::from_millis(16));
    h.frame(view(true), Duration::from_millis(2000));
    let on = h.screenshot(view(true), &theme, 1.0);
    assert!(
        distance(
            on.at(Point::new(20.0 + 36.0, 20.0 + 24.0)),
            rgb(theme.colors.on_primary)
        ) <= 1
    );
    assert!(
        distance(
            on.at(Point::new(20.0 + 4.0, 20.0 + 24.0)),
            rgb(theme.colors.primary)
        ) <= 1
    );
}

#[test]
fn radio_group_selects_by_click_and_arrows() {
    let theme = Theme::light();
    let view = |selected| {
        place(
            radio_group(&theme, ["One", "Two", "Three"], Some(selected))
                .disable(1)
                .on_select(Message::Select)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(300.0, 240.0));
    assert_eq!(
        h.click(|| view(0), Point::new(60.0, 44.0 + 48.0 * 2.0)),
        vec![Message::Select(2)]
    );
    assert!(
        h.click(|| view(0), Point::new(60.0, 44.0 + 48.0))
            .is_empty()
    );
    h.update(view(0), &key(Named::Tab, false));
    assert_eq!(
        h.update(view(0), &key(Named::ArrowDown, false)),
        vec![Message::Select(2)]
    );
    assert_eq!(
        h.update(view(2), &key(Named::ArrowDown, false)),
        vec![Message::Select(0)]
    );
    assert_eq!(
        h.update(view(0), &key(Named::ArrowUp, false)),
        vec![Message::Select(2)]
    );
}

#[test]
fn radio_styles_follow_tokens() {
    let theme = Theme::light();
    let c = theme.colors;
    assert_eq!(radio::style(&theme, Status::Active, true).icon, c.primary);
    assert_eq!(
        radio::style(&theme, Status::Active, false).icon,
        c.on_surface_variant
    );
    assert_eq!(
        radio::style(&theme, Status::Hovered, false).icon,
        c.on_surface
    );
    let selected = radio::style(&theme, Status::Hovered, true);
    assert_eq!(
        (selected.hover_layer, selected.pressed_layer),
        (c.primary, c.on_surface)
    );
    let hover = overlay(c.surface, c.primary, theme.state.hover);
    assert_ne!(hover, c.surface);
}
