// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, key, rgb};
use iced::keyboard::key::Named;
use iced::widget::container;
use iced::{Length, Point, Size};
use iced_material::state::overlay;
use iced_material::widget::focus_scope;
use iced_material::widget::slider::{range_slider, slider};
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Value(f32),
    Range(f32, f32),
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    focus_scope(container(content).padding(20)).into()
}

fn single(theme: &Theme, value: f32) -> Element<'static, Message> {
    place(
        slider(theme, 0.0..=100.0, value)
            .step(10.0)
            .width(Length::Fixed(300.0))
            .on_change(Message::Value)
            .into(),
    )
}

fn x_of(value: f32) -> f32 {
    30.0 + value / 100.0 * 280.0
}

const Y: f32 = 44.0;

#[test]
fn handles_are_focus_targets_with_the_state_layer_size() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 120.0));
    let one = h.focusables(single(&theme, 50.0));
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].size(), Size::new(40.0, 40.0));
    assert_eq!(one[0].center(), Point::new(x_of(50.0), Y));
    let two = h.focusables(place(
        range_slider(&theme, 0.0..=100.0, (30.0, 60.0))
            .width(Length::Fixed(300.0))
            .on_range_change(Message::Range)
            .into(),
    ));
    assert_eq!(two.len(), 2);
    let off = h.focusables(place(
        slider(&theme, 0.0..=100.0, 50.0)
            .width(Length::Fixed(300.0))
            .into(),
    ));
    assert!(off.is_empty());
}

#[test]
fn track_press_snaps_to_the_step_and_drag_follows_the_pointer() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 120.0));
    let messages = h.press(|| single(&theme, 0.0), Point::new(x_of(50.0) + 3.0, Y));
    assert_eq!(messages, vec![Message::Value(50.0)]);
    let messages = h.move_cursor(single(&theme, 50.0), Point::new(x_of(80.0), Y + 10.0));
    assert_eq!(messages, vec![Message::Value(80.0)]);
    assert!(h.release(single(&theme, 80.0)).is_empty());
    assert!(
        h.move_cursor(single(&theme, 80.0), Point::new(x_of(10.0), Y))
            .is_empty()
    );
}

#[test]
fn grabbing_the_handle_keeps_the_offset() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 120.0));
    assert!(
        h.press(|| single(&theme, 50.0), Point::new(x_of(50.0) + 6.0, Y))
            .is_empty()
    );
    let messages = h.move_cursor(single(&theme, 50.0), Point::new(x_of(70.0) + 6.0, Y));
    assert_eq!(messages, vec![Message::Value(70.0)]);
}

#[test]
fn keyboard_wheel_and_clamping() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 120.0));
    h.update(single(&theme, 50.0), &key(Named::Tab, false));
    let send = |h: &mut Harness, named, value| h.update(single(&theme, value), &key(named, false));
    assert_eq!(
        send(&mut h, Named::ArrowRight, 50.0),
        vec![Message::Value(60.0)]
    );
    assert_eq!(
        send(&mut h, Named::ArrowLeft, 50.0),
        vec![Message::Value(40.0)]
    );
    assert_eq!(
        send(&mut h, Named::PageUp, 50.0),
        vec![Message::Value(60.0)]
    );
    assert_eq!(send(&mut h, Named::Home, 50.0), vec![Message::Value(0.0)]);
    assert_eq!(send(&mut h, Named::End, 50.0), vec![Message::Value(100.0)]);
    assert!(send(&mut h, Named::ArrowRight, 100.0).is_empty());
    let up = h.scroll(single(&theme, 50.0), Point::new(100.0, Y), 1.0);
    assert_eq!(up, vec![Message::Value(60.0)]);
    let down = h.scroll(single(&theme, 50.0), Point::new(100.0, Y), -1.0);
    assert_eq!(down, vec![Message::Value(40.0)]);
}

#[test]
fn range_handles_do_not_cross() {
    let theme = Theme::light();
    let view = || {
        place(
            range_slider(&theme, 0.0..=100.0, (30.0, 60.0))
                .step(10.0)
                .width(Length::Fixed(300.0))
                .on_range_change(Message::Range)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(400.0, 120.0));
    h.update(view(), &key(Named::Tab, false));
    assert_eq!(
        h.update(view(), &key(Named::End, false)),
        vec![Message::Range(60.0, 60.0)]
    );
    h.update(view(), &key(Named::Tab, false));
    assert_eq!(
        h.update(view(), &key(Named::Home, false)),
        vec![Message::Range(30.0, 30.0)]
    );
}

#[test]
fn tracks_ticks_and_handle_use_token_colors() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        place(
            slider(&theme, 0.0..=100.0, 50.0)
                .step(10.0)
                .ticks(true)
                .width(Length::Fixed(300.0))
                .on_change(Message::Value)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(400.0, 120.0));
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(60.0, Y)), rgb(c.primary)) <= 1);
    assert!(distance(image.at(Point::new(60.0, Y + 6.0)), rgb(c.surface)) <= 1);
    assert!(
        distance(
            image.at(Point::new(260.0, Y)),
            rgb(c.surface_container_highest)
        ) <= 1
    );
    assert!(distance(image.at(Point::new(x_of(50.0), Y)), rgb(c.primary)) <= 1);
    assert!(
        distance(
            image.at(Point::new(x_of(20.0) - 0.5, Y - 0.5)),
            rgb(c.on_primary)
        ) <= 40
    );
    assert!(
        distance(
            image.at(Point::new(x_of(80.0) - 0.5, Y - 0.5)),
            rgb(c.on_surface_variant)
        ) <= 40
    );
}

#[test]
fn disabled_slider_uses_on_surface_opacities() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(400.0, 120.0));
    let view = place(
        slider(&theme, 0.0..=100.0, 50.0)
            .width(Length::Fixed(300.0))
            .into(),
    );
    let image = h.screenshot(view, &theme, 1.0);
    let active = rgb(overlay(c.surface, c.on_surface, theme.disabled.content));
    let inactive = rgb(overlay(c.surface, c.on_surface, theme.disabled.container));
    assert!(distance(image.at(Point::new(60.0, Y)), active) <= 1);
    assert!(distance(image.at(Point::new(260.0, Y)), inactive) <= 1);
}

#[test]
fn value_label_appears_above_a_hovered_handle() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        place(
            slider(&theme, 0.0..=100.0, 50.0)
                .step(10.0)
                .labeled(true)
                .width(Length::Fixed(300.0))
                .on_change(Message::Value)
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(400.0, 120.0));
    let before = h.screenshot(view(), &theme, 1.0);
    assert!(
        distance(
            before.at(Point::new(x_of(50.0) - 11.0, 15.0)),
            rgb(c.surface)
        ) <= 1
    );
    h.move_cursor(view(), Point::new(x_of(50.0), Y));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(16));
    h.frame(view(), Duration::from_millis(1000));
    let after = h.screenshot(view(), &theme, 1.0);
    assert!(
        distance(
            after.at(Point::new(x_of(50.0) - 11.0, 15.0)),
            rgb(c.primary)
        ) <= 1
    );
}

#[test]
fn the_handle_reaches_both_ends_of_the_track() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(400.0, 120.0));
    for (value, probe, outside) in [(100.0, 318.0, 328.0), (0.0, 22.0, 12.0)] {
        let image = h.screenshot(single(&theme, value), &theme, 1.0);
        assert_eq!(image.at(Point::new(probe, Y)), rgb(c.primary), "{value}");
        assert_eq!(image.at(Point::new(outside, Y)), rgb(c.surface), "{value}");
    }
}
