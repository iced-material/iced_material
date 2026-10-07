// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, ink, key, rgb, run, solid};
use iced::keyboard::key::Named;
use iced::widget::{container, space};
use iced::{Length, Point, Rectangle, Size};
use iced_material::state::overlay;
use iced_material::widget::dialog::{dialog, modal};
use iced_material::widget::menu::{self, item, menu as menu_widget};
use iced_material::widget::select;
use iced_material::widget::snackbar::snackbar;
use iced_material::widget::tooltip;
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Select(usize),
    Dismiss,
    Toggle,
    Action,
}

fn anchor() -> Element<'static, Message> {
    space().width(100).height(40).into()
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content).padding(40).into()
}

fn size() -> Size {
    Size::new(400.0, 400.0)
}

#[test]
fn plain_tooltip_opens_after_the_delay_above_the_anchor() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || place(tooltip::plain(&theme, anchor(), "Tooltip"));
    let mut h = Harness::new(size());
    h.move_cursor(view(), Point::new(90.0, 60.0));
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(300));
    let area = Rectangle::new(Point::new(40.0, 0.0), Size::new(160.0, 40.0));
    let early = h.screenshot(view(), &theme, 1.0);
    assert!(solid(&early, area, rgb(c.inverse_surface)).is_none());
    h.frame(view(), Duration::from_millis(700));
    h.frame(view(), Duration::from_millis(716));
    let shown = h.screenshot(view(), &theme, 1.0);
    let bubble = solid(&shown, area, rgb(c.inverse_surface)).unwrap();
    assert_eq!(bubble.height, 24.0 - 2.0 * 0.0);
    assert_eq!(bubble.y + bubble.height, 40.0 - 4.0);
    h.move_cursor(view(), Point::new(300.0, 300.0));
    h.frame(view(), Duration::from_millis(800));
    let gone = h.screenshot(view(), &theme, 1.0);
    assert!(solid(&gone, area, rgb(c.inverse_surface)).is_none());
}

fn menu_view(theme: &Theme, open: bool) -> Element<'static, Message> {
    place(
        menu_widget(
            anchor(),
            vec![
                item("Cut").on_select(Message::Select(0)),
                item("Copy").on_select(Message::Select(1)),
                menu::divider(),
                item("More").submenu(vec![
                    item("First").on_select(Message::Select(10)),
                    item("Second").on_select(Message::Select(11)),
                ]),
            ],
            open,
        )
        .on_dismiss(Message::Dismiss)
        .build(theme),
    )
}

#[test]
fn menu_draws_below_the_anchor_and_publishes_choices() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(size());
    let closed = h.screenshot(menu_view(&theme, false), &theme, 1.0);
    assert!(
        solid(
            &closed,
            Rectangle::new(Point::ORIGIN, size()),
            rgb(c.surface_container)
        )
        .is_none()
    );
    let open = h.screenshot(menu_view(&theme, true), &theme, 1.0);
    let inside = Point::new(44.0, 84.0);
    assert_eq!(open.at(inside), rgb(c.surface_container));
    assert_eq!(44 - run(&open, inside, -1, 0), 40);
    assert_eq!(run(&open, inside, 1, 0) + 44 + 1, 40 + 112);
    assert_eq!(84 - run(&open, inside, 0, -1), 80);
    assert_eq!(
        run(&open, Point::new(44.0, 250.0), 0, 1) + 250 + 1,
        80 + 177
    );
    assert_eq!(
        h.click(
            || menu_view(&theme, true),
            Point::new(70.0, 80.0 + 8.0 + 48.0 + 24.0)
        ),
        vec![Message::Select(1), Message::Dismiss]
    );
}

#[test]
fn menu_dismisses_on_escape_and_outside_press_without_reaching_the_anchor() {
    let theme = Theme::light();
    let mut h = Harness::new(size());
    h.frame(menu_view(&theme, true), Duration::ZERO);
    assert_eq!(
        h.update(menu_view(&theme, true), &key(Named::Escape, false)),
        vec![Message::Dismiss]
    );
    assert_eq!(
        h.click(|| menu_view(&theme, true), Point::new(350.0, 350.0)),
        vec![Message::Dismiss]
    );
}

#[test]
fn menu_keyboard_navigation_and_submenu() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || menu_view(&theme, true);
    let mut h = Harness::new(size());
    h.frame(view(), Duration::ZERO);
    h.update(view(), &key(Named::ArrowDown, false));
    assert_eq!(
        h.update(view(), &key(Named::Enter, false)),
        vec![Message::Select(1), Message::Dismiss]
    );

    let mut h = Harness::new(size());
    h.frame(view(), Duration::ZERO);
    h.update(view(), &key(Named::ArrowDown, false));
    h.update(view(), &key(Named::ArrowDown, false));
    h.update(view(), &key(Named::ArrowRight, false));
    h.frame(view(), Duration::from_millis(16));
    let image = h.screenshot(view(), &theme, 1.0);
    let probe = Point::new(156.0, 197.0);
    assert_eq!(image.at(probe), rgb(c.surface_container));
    assert_eq!(156 - run(&image, probe, -1, 0), 152);
    assert_eq!(197 - run(&image, probe, 0, -1), 193);
    h.update(view(), &key(Named::ArrowDown, false));
    assert_eq!(
        h.update(view(), &key(Named::Enter, false)),
        vec![Message::Select(11), Message::Dismiss]
    );
}

fn dialog_view(theme: &Theme, open: bool) -> Element<'static, Message> {
    modal(
        space().width(Length::Fill).height(Length::Fill),
        dialog()
            .headline("Headline")
            .supporting_text("Supporting text")
            .action("Cancel", Message::Dismiss)
            .action("Accept", Message::Action),
        open,
    )
    .on_dismiss(Message::Dismiss)
    .build(theme)
}

#[test]
fn dialog_animates_in_and_out_with_a_scrim() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(size());
    let before = h.screenshot(dialog_view(&theme, false), &theme, 1.0);
    assert!(distance(before.at(Point::new(5.0, 5.0)), rgb(c.surface)) <= 1);
    h.frame(dialog_view(&theme, true), Duration::ZERO);
    h.frame(dialog_view(&theme, true), Duration::from_millis(16));
    h.frame(dialog_view(&theme, true), Duration::from_millis(1000));
    let open = h.screenshot(dialog_view(&theme, true), &theme, 1.0);
    let scrim = rgb(overlay(c.surface, c.scrim, 0.32));
    assert!(distance(open.at(Point::new(5.0, 5.0)), scrim) <= 1);
    let panel = solid(
        &open,
        Rectangle::new(Point::ORIGIN, size()),
        rgb(c.surface_container_high),
    )
    .unwrap();
    assert!(panel.width >= 280.0 && panel.width <= 560.0);
    assert!((panel.center_x() - 200.0).abs() <= 1.0);
    assert!((panel.center_y() - 200.0).abs() <= 1.0);

    h.frame(dialog_view(&theme, false), Duration::from_millis(1100));
    h.frame(dialog_view(&theme, false), Duration::from_millis(1400));
    let closed = h.screenshot(dialog_view(&theme, false), &theme, 1.0);
    assert!(distance(closed.at(Point::new(5.0, 5.0)), rgb(c.surface)) <= 1);
}

#[test]
fn dialog_traps_focus_dismisses_on_escape_and_blocks_the_base() {
    let theme = Theme::light();
    let view = || dialog_view(&theme, true);
    let mut h = Harness::new(size());
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(1000));
    let stops = h.focusables(view());
    assert_eq!(stops.len(), 2, "{stops:?}");
    assert!(stops[0].x < stops[1].x, "{stops:?}");
    assert_eq!(h.click(view, stops[1].center()), vec![Message::Action]);
    assert_eq!(
        h.update(view(), &key(Named::Escape, false)),
        vec![Message::Dismiss]
    );
    let mut h = Harness::new(size());
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(1000));
    h.update(view(), &key(Named::Tab, false));
    h.update(view(), &key(Named::Tab, false));
    h.update(view(), &key(Named::Tab, false));
    assert_eq!(
        h.update(view(), &key(Named::Enter, false)),
        vec![Message::Action]
    );
}

#[test]
fn snackbar_is_48_dp_for_one_line_and_68_dp_for_two() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(size());
    let area = Rectangle::new(Point::ORIGIN, size());
    let one = place(
        snackbar::<Message>("Message sent")
            .action("Undo", Message::Action)
            .on_dismiss(Message::Dismiss)
            .build(&theme),
    );
    let image = h.screenshot(one, &theme, 1.0);
    let bar = solid(&image, area, rgb(c.inverse_surface)).unwrap();
    assert_eq!(bar.height, 48.0);
    let two = place(
        container(
            snackbar::<Message>("This snackbar message needs two lines of text").build(&theme),
        )
        .max_width(220)
        .into(),
    );
    let image = h.screenshot(two, &theme, 1.0);
    let bar = solid(&image, area, rgb(c.inverse_surface)).unwrap();
    assert_eq!(bar.height, 68.0);
}

#[test]
fn snackbar_action_publishes() {
    let theme = Theme::light();
    let view = || {
        place(
            snackbar::<Message>("Message sent")
                .action("Undo", Message::Action)
                .on_dismiss(Message::Dismiss)
                .build(&theme),
        )
    };
    let mut h = Harness::new(size());
    let stops = h.focusables(view());
    assert_eq!(stops.len(), 2);
    assert_eq!(h.click(view, stops[0].center()), vec![Message::Action]);
    assert_eq!(h.click(view, stops[1].center()), vec![Message::Dismiss]);
}

fn select_view(theme: &Theme, selected: Option<usize>, open: bool) -> Element<'static, Message> {
    place(
        select::filled(["Apple", "Banana", "Cherry"], selected, open)
            .label("Fruit")
            .width(Length::Fixed(240.0))
            .on_toggle(Message::Toggle)
            .on_select(Message::Select)
            .on_dismiss(Message::Dismiss)
            .build(theme),
    )
}

#[test]
fn select_opens_a_menu_as_wide_as_the_field_and_shows_the_choice() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(size());
    assert_eq!(
        h.click(|| select_view(&theme, None, false), Point::new(100.0, 70.0)),
        vec![Message::Toggle]
    );
    let image = h.screenshot(select_view(&theme, Some(1), true), &theme, 1.0);
    let probe = Point::new(44.0, 100.0);
    assert_eq!(image.at(probe), rgb(c.surface_container));
    assert_eq!(44 - run(&image, probe, -1, 0), 40);
    assert_eq!(run(&image, probe, 1, 0) + 44 + 1, 40 + 240);
    assert_eq!(100 - run(&image, probe, 0, -1), 96);
    assert_eq!(
        run(&image, Point::new(44.0, 252.0), 0, 1) + 252 + 1,
        96 + 160
    );
    assert!(
        distance(
            image.at(Point::new(60.0, 96.0 + 8.0 + 48.0 + 4.0)),
            rgb(c.secondary_container)
        ) <= 1
    );
    let value = ink(
        &image,
        Rectangle::new(Point::new(50.0, 62.0), Size::new(150.0, 24.0)),
        rgb(c.surface_container_highest),
    );
    assert!(value.is_some());
    assert_eq!(
        h.click(
            || select_view(&theme, Some(1), true),
            Point::new(60.0, 96.0 + 8.0 + 48.0 * 2.0 + 24.0)
        ),
        vec![Message::Select(2), Message::Dismiss]
    );
}
