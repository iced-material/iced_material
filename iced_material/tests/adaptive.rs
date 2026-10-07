// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use common::{Harness, rgb, run};
use iced::widget::{Space, container};
use iced::{Color, Length, Point, Size};
use iced_material::icon::symbol;
use iced_material::layout::{WidthClass, adaptive, feed, list_detail, scaffold, supporting_pane};
use iced_material::widget::navigation::Destination;
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Select(usize),
}

fn filled(color: Color) -> Element<'static, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(color.into()),
            ..container::Style::default()
        })
        .into()
}

#[test]
fn adaptive_view_receives_the_class_of_its_space() {
    let theme = Theme::light();
    let c = theme.colors;
    let colors = [
        (500.0, c.primary),
        (700.0, c.secondary),
        (900.0, c.tertiary),
        (1300.0, c.error),
        (1700.0, c.inverse_primary),
    ];
    for (width, expected) in colors {
        let mut h = Harness::new(Size::new(width, 200.0));
        let view = adaptive(|class, size| {
            assert_eq!(size.width, width);
            filled(match class.width {
                WidthClass::Compact => c.primary,
                WidthClass::Medium => c.secondary,
                WidthClass::Expanded => c.tertiary,
                WidthClass::Large => c.error,
                WidthClass::ExtraLarge => c.inverse_primary,
            })
        });
        let image = h.screenshot(view, &theme, 1.0);
        assert_eq!(image.at(Point::new(5.0, 5.0)), rgb(expected), "{width}");
    }
}

fn destinations() -> Vec<Destination> {
    vec![
        Destination::symbol("Search", symbol::search),
        Destination::symbol("Date", symbol::calendar_today),
    ]
}

fn page(theme: &Theme, width: f32, height: f32) -> (Harness, Element<'static, Message>) {
    let h = Harness::new(Size::new(width, height));
    let theme = theme.clone();
    let view = adaptive(move |class, _| {
        scaffold(
            &theme,
            class.width,
            destinations(),
            0,
            Message::Select,
            filled(theme.colors.surface_container_low),
        )
    });
    (h, view)
}

#[test]
fn scaffold_picks_bar_rail_or_drawer() {
    let theme = Theme::light();
    let c = theme.colors;
    let (mut h, view) = page(&theme, 500.0, 400.0);
    h.frame(view, std::time::Duration::ZERO);
    let (mut h, view) = {
        let _ = &mut h;
        page(&theme, 500.0, 400.0)
    };
    let image = h.screenshot(view, &theme, 1.0);
    assert_eq!(
        image.at(Point::new(250.0, 100.0)),
        rgb(c.surface_container_low)
    );
    assert_eq!(image.at(Point::new(5.0, 395.0)), rgb(c.surface_container));
    assert_eq!(
        image.at(Point::new(105.0, 320.0 + 28.0)),
        rgb(c.secondary_container)
    );

    let (mut h, view) = page(&theme, 700.0, 400.0);
    let image = h.screenshot(view, &theme, 1.0);
    assert_eq!(image.at(Point::new(20.0, 64.0)), rgb(c.secondary_container));
    assert_eq!(
        image.at(Point::new(300.0, 100.0)),
        rgb(c.surface_container_low)
    );
    assert_eq!(run(&image, Point::new(300.0, 100.0), -1, 0), 300 - 80);

    let (mut h, view) = page(&theme, 1000.0, 400.0);
    let image = h.screenshot(view, &theme, 1.0);
    assert_eq!(
        image.at(Point::new(100.0, 12.0 + 28.0)),
        rgb(c.secondary_container)
    );
    assert_eq!(
        image.at(Point::new(600.0, 100.0)),
        rgb(c.surface_container_low)
    );
    assert_eq!(run(&image, Point::new(600.0, 100.0), -1, 0), 600 - 360);
}

#[test]
fn list_detail_and_supporting_pane_follow_the_class() {
    let theme = Theme::light();
    let c = theme.colors;
    let (list, detail) = (c.primary, c.tertiary);
    let at = |class, detail_open: bool| {
        let mut h = Harness::new(Size::new(1000.0, 200.0));
        let element = list_detail(
            class,
            false,
            filled(list),
            detail_open.then(|| filled(detail)),
        );
        let image = h.screenshot(element, &theme, 1.0);
        (
            image.at(Point::new(10.0, 100.0)),
            image.at(Point::new(900.0, 100.0)),
        )
    };
    let (left, right) = at(WidthClass::Expanded, true);
    assert_eq!((left, right), (rgb(list), rgb(detail)));
    let mut h = Harness::new(Size::new(1000.0, 200.0));
    let image = h.screenshot(
        list_detail(
            WidthClass::Expanded,
            false,
            filled(list),
            Some(filled(detail)),
        ),
        &theme,
        1.0,
    );
    assert_eq!(run(&image, Point::new(10.0, 100.0), 1, 0), 349);
    assert_eq!(image.at(Point::new(370.0, 100.0)), rgb(c.surface));
    assert_eq!(image.at(Point::new(384.0, 100.0)), rgb(detail));
    assert_eq!(at(WidthClass::Compact, false).1, rgb(list));
    assert_eq!(at(WidthClass::Compact, true).0, rgb(detail));
    assert_eq!(at(WidthClass::Medium, true).0, rgb(detail));
    let medium = list_detail(WidthClass::Medium, true, filled(list), Some(filled(detail)));
    let image = Harness::new(Size::new(1000.0, 200.0)).screenshot(medium, &theme, 1.0);
    assert_eq!(image.at(Point::new(10.0, 100.0)), rgb(list));
    assert_eq!(image.at(Point::new(900.0, 100.0)), rgb(detail));

    let supporting = |class, show| {
        let mut h = Harness::new(Size::new(1000.0, 200.0));
        let element = supporting_pane(class, false, show, filled(list), filled(detail));
        let image = h.screenshot(element, &theme, 1.0);
        (
            image.at(Point::new(10.0, 100.0)),
            image.at(Point::new(990.0, 100.0)),
        )
    };
    assert_eq!(
        supporting(WidthClass::Expanded, false),
        (rgb(list), rgb(detail))
    );
    assert_eq!(supporting(WidthClass::Compact, false).0, rgb(list));
    assert_eq!(supporting(WidthClass::Compact, true).0, rgb(detail));
    let mut h = Harness::new(Size::new(1000.0, 200.0));
    let element = supporting_pane(
        WidthClass::ExtraLarge,
        false,
        false,
        filled(list),
        filled(detail),
    );
    let image = h.screenshot(element, &theme, 1.0);
    assert_eq!(run(&image, Point::new(990.0, 100.0), -1, 0), 402);
}

#[test]
fn feed_arranges_items_in_columns_by_class() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(900.0, 300.0));
    let items = |n: usize| {
        (0..n)
            .map(|_| container(filled(c.primary)).height(100).into())
            .collect::<Vec<Element<'static, Message>>>()
    };
    let image = h.screenshot(feed(WidthClass::Expanded, items(4)), &theme, 1.0);
    let row = Point::new(2.0, 50.0);
    assert_eq!(image.at(row), rgb(c.primary));
    let first = run(&image, row, 1, 0);
    assert_eq!(first + 3, (900 - 48) / 3);
    assert_eq!(
        image.at(Point::new(first as f32 + 10.0, 50.0)),
        rgb(c.surface)
    );
    assert_eq!(image.at(Point::new(2.0, 110.0)), rgb(c.surface));
    assert_eq!(image.at(Point::new(2.0, 130.0)), rgb(c.primary));
    let image = h.screenshot(feed(WidthClass::Compact, items(2)), &theme, 1.0);
    assert_eq!(run(&image, row, 1, 0), 897);
}
