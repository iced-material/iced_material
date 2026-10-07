// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, ink, key, rgb, run, type_text};
use iced::keyboard::key::Named;
use iced::widget::{Space, container, row};
use iced::{Length, Point, Rectangle, Size};
use iced_material::icon::symbol;
use iced_material::state::overlay;
use iced_material::widget::focus_scope;
use iced_material::widget::navigation::{
    Badge, Destination, Labels, navigation_bar, navigation_rail,
};
use iced_material::widget::navigation_drawer::{
    DrawerItem, modal_drawer, navigation_drawer, section,
};
use iced_material::widget::search::search;
use iced_material::widget::side_sheet::side_sheet;
use iced_material::widget::tabs::{Tab, primary, secondary};
use iced_material::widget::top_app_bar::{self, action_button, menu_button};
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Select(usize),
    Menu,
    Dismiss,
    Input(String),
}

fn destinations() -> Vec<Destination> {
    vec![
        Destination::symbol("Search", symbol::search),
        Destination::symbol("Date", symbol::calendar_today),
        Destination::symbol("Time", symbol::schedule).badge(Badge::Count("12".into())),
        Destination::symbol("Edit", symbol::edit).badge(Badge::Dot),
    ]
}

fn settle(h: &mut Harness, view: impl Fn() -> Element<'static, Message>) {
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(16));
    h.frame(view(), Duration::from_millis(1500));
}

#[test]
fn navigation_bar_geometry_colors_and_selection() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || navigation_bar(&theme, destinations(), 1, Message::Select).into();
    let mut h = Harness::new(Size::new(360.0, 120.0));
    settle(&mut h, view);
    let image = h.screenshot(view(), &theme, 1.0);
    assert_eq!(image.at(Point::new(2.0, 75.0)), rgb(c.surface_container));
    assert_eq!(run(&image, Point::new(2.0, 75.0), 0, 1), 4);
    assert_eq!(
        image.at(Point::new(90.0 + 15.0, 28.0)),
        rgb(c.secondary_container)
    );
    assert_eq!(image.at(Point::new(15.0, 28.0)), rgb(c.surface_container));
    let pill = Point::new(90.0 + 28.0, 28.0);
    assert_eq!(run(&image, pill, -1, 0), 15);
    assert_eq!(run(&image, pill, 0, -1), 16);
    assert_eq!(run(&image, pill, 0, 1), 15);
    let label = ink(
        &image,
        Rectangle::new(Point::new(95.0, 48.0), Size::new(80.0, 18.0)),
        rgb(c.surface_container),
    );
    assert!(label.is_some());
    assert_eq!(h.focusables(view()).len(), 4);
    assert_eq!(
        h.click(view, Point::new(250.0, 40.0)),
        vec![Message::Select(2)]
    );
}

#[test]
fn navigation_rail_layout_and_labels() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        navigation_rail(&theme, destinations(), 0, Message::Select)
            .labels(Labels::Selected)
            .into()
    };
    let mut h = Harness::new(Size::new(120.0, 400.0));
    settle(&mut h, view);
    let image = h.screenshot(view(), &theme, 1.0);
    assert_eq!(image.at(Point::new(5.0, 300.0)), rgb(c.surface));
    assert_eq!(
        image.at(Point::new(14.0, 44.0 + 4.0 + 16.0)),
        rgb(c.secondary_container)
    );
    assert_eq!(
        image.at(Point::new(14.0, 44.0 + 60.0 + 28.0)),
        rgb(c.surface)
    );
    let selected_label = ink(
        &image,
        Rectangle::new(Point::new(5.0, 44.0 + 4.0 + 32.0), Size::new(70.0, 20.0)),
        rgb(c.surface),
    );
    assert!(selected_label.is_some());
    let hidden_label = ink(
        &image,
        Rectangle::new(
            Point::new(5.0, 44.0 + 60.0 + 4.0 + 32.0 + 8.0),
            Size::new(70.0, 12.0),
        ),
        rgb(c.surface),
    );
    assert!(hidden_label.is_none(), "{hidden_label:?}");
    assert_eq!(
        h.click(view, Point::new(40.0, 44.0 + 60.0 + 28.0)),
        vec![Message::Select(1)]
    );
}

fn drawer(theme: &Theme) -> iced_material::widget::navigation_drawer::Drawer<Message> {
    let _ = theme;
    navigation_drawer(
        vec![
            section(vec![
                DrawerItem::new("Inbox").symbol(symbol::search).badge("24"),
                DrawerItem::new("Sent").symbol(symbol::edit),
            ])
            .headline("Mail"),
            section(vec![DrawerItem::new("Trash").symbol(symbol::close)]),
        ],
        1,
        Message::Select,
    )
}

#[test]
fn standard_drawer_items_and_selection() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        row![
            drawer(&theme).build(&theme),
            Space::new().width(Length::Fill)
        ]
        .into()
    };
    let mut h = Harness::new(Size::new(600.0, 400.0));
    settle(&mut h, view);
    let image = h.screenshot(view(), &theme, 1.0);
    let first = 12.0 + 56.0;
    assert_eq!(
        image.at(Point::new(14.0, first + 56.0 + 28.0)),
        rgb(c.secondary_container)
    );
    assert_eq!(image.at(Point::new(14.0, first + 28.0)), rgb(c.surface));
    assert_eq!(image.at(Point::new(5.0, 300.0)), rgb(c.surface));
    assert_eq!(
        h.click(view, Point::new(100.0, first + 28.0)),
        vec![Message::Select(0)]
    );
}

#[test]
fn modal_drawer_slides_in_over_a_scrim() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = |open| {
        modal_drawer(drawer(&theme), open)
            .on_dismiss(Message::Dismiss)
            .build(
                &theme,
                Space::new().width(Length::Fill).height(Length::Fill),
            )
    };
    let mut h = Harness::new(Size::new(600.0, 400.0));
    h.frame(view(true), Duration::ZERO);
    h.frame(view(true), Duration::from_millis(16));
    h.frame(view(true), Duration::from_millis(1000));
    let image = h.screenshot(view(true), &theme, 1.0);
    assert_eq!(
        image.at(Point::new(5.0, 300.0)),
        rgb(c.surface_container_low)
    );
    assert!(
        distance(
            image.at(Point::new(500.0, 300.0)),
            rgb(overlay(c.surface, c.scrim, 0.32))
        ) <= 1
    );
    assert_eq!(
        h.click(|| view(true), Point::new(500.0, 300.0)),
        vec![Message::Dismiss]
    );
    h.frame(view(false), Duration::from_millis(1100));
    h.frame(view(false), Duration::from_millis(1500));
    let closed = h.screenshot(view(false), &theme, 1.0);
    assert_eq!(closed.at(Point::new(5.0, 300.0)), rgb(c.surface));
}

fn tab_set() -> Vec<Tab> {
    vec![Tab::new("Flights"), Tab::new("Trips"), Tab::new("Explore")]
}

#[test]
fn primary_tabs_indicator_follows_the_selection() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = |selected| {
        container(primary(&theme, tab_set(), selected, Message::Select))
            .width(Length::Fixed(300.0))
            .into()
    };
    let mut h = Harness::new(Size::new(300.0, 100.0));
    settle(&mut h, || view(0));
    let first = h.screenshot(view(0), &theme, 1.0);
    assert!(distance(first.at(Point::new(50.0, 46.0)), rgb(c.primary)) <= 1);
    assert_eq!(first.at(Point::new(150.0, 46.0)), rgb(c.surface));
    assert_eq!(first.at(Point::new(10.0, 48.0)), rgb(c.outline_variant));
    h.frame(view(1), Duration::from_millis(2000));
    h.frame(view(1), Duration::from_millis(2010));
    h.frame(view(1), Duration::from_millis(4000));
    let second = h.screenshot(view(1), &theme, 1.0);
    assert!(distance(second.at(Point::new(150.0, 46.0)), rgb(c.primary)) <= 1);
    assert_eq!(second.at(Point::new(50.0, 46.0)), rgb(c.surface));
    assert_eq!(
        h.click(|| view(1), Point::new(250.0, 20.0)),
        vec![Message::Select(2)]
    );
}

#[test]
fn secondary_tabs_indicator_spans_the_tab_and_arrows_select() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        focus_scope(
            container(secondary(&theme, tab_set(), 1, Message::Select)).width(Length::Fixed(300.0)),
        )
        .into()
    };
    let mut h = Harness::new(Size::new(300.0, 100.0));
    settle(&mut h, view);
    let image = h.screenshot(view(), &theme, 1.0);
    assert!(distance(image.at(Point::new(105.0, 47.0)), rgb(c.primary)) <= 1);
    assert!(distance(image.at(Point::new(105.0, 46.0)), rgb(c.primary)) <= 1);
    assert_eq!(image.at(Point::new(105.0, 45.0)), rgb(c.surface));
    h.update(view(), &key(Named::Tab, false));
    assert_eq!(
        h.update(view(), &key(Named::ArrowRight, false)),
        vec![Message::Select(2)]
    );
    assert_eq!(
        h.update(view(), &key(Named::Home, false)),
        vec![Message::Select(0)]
    );
}

#[test]
fn top_app_bars_have_the_specified_heights_and_title_places() {
    let theme = Theme::light();
    let c = theme.colors;
    let width = 400.0;
    let build = |kind: &str, collapse: f32| -> Element<'static, Message> {
        let nav = menu_button(&theme, Message::Menu);
        let act = action_button(&theme, symbol::more_vert(false), Message::Dismiss);
        let bar = match kind {
            "small" => top_app_bar::small("Title"),
            "center" => top_app_bar::center_aligned("Title"),
            "medium" => top_app_bar::medium("Title"),
            _ => top_app_bar::large("Title"),
        }
        .navigation(nav)
        .action(act)
        .scrolled(true)
        .collapse(collapse);
        container(bar.build(&theme))
            .width(Length::Fixed(width))
            .into()
    };
    let mut h = Harness::new(Size::new(400.0, 300.0));
    for (kind, collapse, height) in [
        ("small", 0.0, 64),
        ("center", 0.0, 64),
        ("medium", 0.0, 112),
        ("large", 0.0, 152),
        ("large", 1.0, 64),
    ] {
        let image = h.screenshot(build(kind, collapse), &theme, 1.0);
        let probe = Point::new(300.0, 2.0);
        assert_eq!(image.at(probe), rgb(c.surface_container), "{kind}");
        assert_eq!(run(&image, probe, 0, 1) + 3, height, "{kind} {collapse}");
    }
    let small = h.screenshot(build("small", 0.0), &theme, 1.0);
    let title = ink(
        &small,
        Rectangle::new(Point::new(50.0, 10.0), Size::new(200.0, 44.0)),
        rgb(c.surface_container),
    )
    .unwrap();
    assert!(title.x >= 56.0 && title.x <= 60.0, "{title:?}");
    let centered = h.screenshot(build("center", 0.0), &theme, 1.0);
    let title = ink(
        &centered,
        Rectangle::new(Point::new(60.0, 10.0), Size::new(280.0, 44.0)),
        rgb(c.surface_container),
    )
    .unwrap();
    assert!((title.center_x() - 200.0).abs() <= 2.0, "{title:?}");
    assert_eq!(h.focusables(build("small", 0.0)).len(), 2);
}

#[test]
fn search_bar_is_a_56_dp_pill_and_edits_text() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || {
        search("")
            .placeholder("Search")
            .on_input(Message::Input)
            .build(&theme)
    };
    let mut h = Harness::new(Size::new(800.0, 300.0));
    let image = h.screenshot(view(), &theme, 1.0);
    let probe = Point::new(300.0, 28.0);
    assert_eq!(image.at(probe), rgb(c.surface_container_high));
    assert_eq!(run(&image, Point::new(300.0, 10.0), 0, 1) + 11, 56);
    assert_eq!(run(&image, Point::new(300.0, 28.0), 1, 0) + 301, 720);
    h.click(view, Point::new(120.0, 28.0));
    assert_eq!(
        h.update(view(), &type_text("a")),
        vec![Message::Input("a".to_string())]
    );
    let expanded = || {
        search("a")
            .on_input(Message::Input)
            .results(Space::new().height(Length::Fixed(100.0)))
            .build(&theme)
    };
    let image = h.screenshot(expanded(), &theme, 1.0);
    assert_eq!(image.at(Point::new(300.0, 56.0)), rgb(c.outline));
    assert_eq!(
        run(&image, Point::new(300.0, 60.0), 0, 1) + 61,
        56 + 1 + 100
    );
}

#[test]
fn side_sheets_standard_and_modal() {
    let theme = Theme::light();
    let c = theme.colors;
    let standard = || {
        row![
            Space::new().width(Length::Fill),
            side_sheet("Title", Space::new())
                .on_close(Message::Dismiss)
                .action("Save", Message::Select(1))
                .build(&theme)
        ]
        .into()
    };
    let mut h = Harness::new(Size::new(600.0, 400.0));
    let image = h.screenshot(standard(), &theme, 1.0);
    assert_eq!(
        image.at(Point::new(600.0 - 256.0 - 1.0, 200.0)),
        rgb(c.outline)
    );
    assert_eq!(image.at(Point::new(600.0 - 100.0, 300.0)), rgb(c.surface));
    assert_eq!(h.focusables(standard()).len(), 2);
    let modal = |open| {
        side_sheet("Title", Space::new())
            .on_close(Message::Dismiss)
            .width(320.0)
            .modal(
                &theme,
                Space::new().width(Length::Fill).height(Length::Fill),
                open,
                Some(Message::Dismiss),
            )
    };
    h.frame(modal(true), Duration::ZERO);
    h.frame(modal(true), Duration::from_millis(16));
    h.frame(modal(true), Duration::from_millis(1000));
    let image = h.screenshot(modal(true), &theme, 1.0);
    assert_eq!(
        image.at(Point::new(600.0 - 100.0, 300.0)),
        rgb(c.surface_container_low)
    );
    assert!(
        distance(
            image.at(Point::new(100.0, 300.0)),
            rgb(overlay(c.surface, c.scrim, 0.32))
        ) <= 1
    );
    assert_eq!(
        h.click(|| modal(true), Point::new(100.0, 300.0)),
        vec![Message::Dismiss]
    );
}
