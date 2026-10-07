// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, key, rgb, run, type_text};
use iced::keyboard::key::Named;
use iced::widget::Space;
use iced::{Length, Point, Size};
use iced_material::calendar::Date;
use iced_material::widget::date_picker::{self, Event as DateEvent, Format, ViewMode};
use iced_material::widget::time_picker::{self, Event as TimeEvent, Select, time_picker};
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Date(DateEvent),
    Time(TimeEvent),
    Cancel,
    Confirm,
}

fn today() -> Date {
    Date::new(2025, 8, 12).unwrap()
}

fn base() -> Element<'static, Message> {
    Space::new().width(Length::Fill).height(Length::Fill).into()
}

fn size() -> Size {
    Size::new(600.0, 700.0)
}

fn date_view<'a>(theme: &Theme, state: &'a date_picker::State) -> Element<'a, Message> {
    date_picker::modal(state, today(), Message::Date)
        .on_cancel(Message::Cancel)
        .on_confirm(Message::Confirm)
        .over(theme, base(), true, Some(Message::Cancel))
}

fn open<'a>(h: &mut Harness, view: impl Fn() -> Element<'a, Message>) {
    h.frame(view(), Duration::ZERO);
    h.frame(view(), Duration::from_millis(16));
    h.frame(view(), Duration::from_millis(1500));
}

#[test]
fn date_format_parses_and_prints_in_each_order() {
    let date = Date::new(2025, 8, 17).unwrap();
    for (format, text) in [
        (Format::MonthDayYear, "08/17/2025"),
        (Format::DayMonthYear, "17/08/2025"),
        (Format::YearMonthDay, "2025/08/17"),
    ] {
        assert_eq!(format.format(date), text);
        assert_eq!(format.parse(text), Some(date));
    }
    assert_eq!(Format::DayMonthYear.parse("17.8.2025"), Some(date));
    assert_eq!(Format::MonthDayYear.parse("8-17-2025"), Some(date));
    assert_eq!(Format::MonthDayYear.parse("02/30/2025"), None);
    assert_eq!(Format::MonthDayYear.parse("08/17"), None);
    assert_eq!(Format::MonthDayYear.parse("aa/bb/cccc"), None);
}

#[test]
fn date_state_follows_events() {
    let mut state = date_picker::State::new(None, today()).years(2000..=2030);
    assert_eq!((state.shown(), state.selected()), ((2025, 8), None));
    state.update(DateEvent::Month(5));
    assert_eq!(state.shown(), (2026, 1));
    state.update(DateEvent::Month(-1));
    assert_eq!(state.shown(), (2025, 12));
    state.update(DateEvent::Month(-312));
    assert_eq!(state.shown(), (2025, 12));
    let pick = Date::new(2024, 2, 29).unwrap();
    state.update(DateEvent::Pick(pick));
    assert_eq!((state.selected(), state.shown()), (Some(pick), (2024, 2)));
    state.update(DateEvent::ToggleYears);
    assert_eq!(state.mode(), ViewMode::Years);
    state.update(DateEvent::Year(2027));
    assert_eq!(
        (state.mode(), state.shown()),
        (ViewMode::Calendar, (2027, 2))
    );
    state.update(DateEvent::ToggleInput);
    assert_eq!(state.mode(), ViewMode::Input);
    state.update(DateEvent::Text("03/05/2026".into()));
    assert_eq!(state.selected(), Date::new(2026, 3, 5));
    assert_eq!(state.shown(), (2026, 3));
    state.update(DateEvent::Text("03/05/1999".into()));
    assert_eq!(state.selected(), None);
    state.update(DateEvent::Text("13/05/2026".into()));
    assert_eq!(state.selected(), None);
}

#[test]
fn modal_date_picker_has_the_specified_size_and_colors() {
    let theme = Theme::light();
    let c = theme.colors;
    let state = date_picker::State::new(Date::new(2025, 8, 17), today());
    let view = || date_view(&theme, &state);
    let mut h = Harness::new(size());
    open(&mut h, view);
    let image = h.screenshot(view(), &theme, 1.0);
    let inside = Point::new(300.0, 200.0);
    assert_eq!(image.at(inside), rgb(c.surface_container_high));
    assert_eq!(inside.x as i32 - run(&image, inside, -1, 0), 120);
    assert_eq!(run(&image, inside, 1, 0) + 301, 120 + 360);
    assert_eq!(run(&image, Point::new(300.0, 100.0), 0, -1), 34);
    assert_eq!(run(&image, Point::new(300.0, 600.0), 0, 1), 33);
}

fn day_center(day: u8) -> Point {
    let offset = 5 + usize::from(day) - 1;
    Point::new(
        132.0 + (offset % 7) as f32 * 48.0 + 24.0,
        66.0 + 120.0 + 1.0 + 56.0 + 48.0 + (offset / 7) as f32 * 48.0 + 24.0,
    )
}

#[test]
fn clicking_a_day_picks_it_and_the_grid_has_the_right_layout() {
    let theme = Theme::light();
    let c = theme.colors;
    let state = date_picker::State::new(Date::new(2025, 8, 17), today());
    let view = || date_view(&theme, &state);
    let mut h = Harness::new(size());
    open(&mut h, view);
    let image = h.screenshot(view(), &theme, 1.0);
    let selected = day_center(17);
    assert_eq!(
        image.at(Point::new(selected.x, selected.y - 10.0)),
        rgb(c.primary)
    );
    assert_eq!(
        run(&image, Point::new(selected.x, selected.y - 10.0), 0, -1),
        10
    );
    let other = day_center(18);
    assert_eq!(
        image.at(Point::new(other.x, other.y - 17.0)),
        rgb(c.surface_container_high)
    );
    let messages = h.click(view, day_center(21));
    assert_eq!(
        messages,
        vec![Message::Date(DateEvent::Pick(
            Date::new(2025, 8, 21).unwrap()
        ))]
    );
}

#[test]
fn today_is_outlined_and_other_months_are_blank() {
    let theme = Theme::light();
    let c = theme.colors;
    let state = date_picker::State::new(Date::new(2025, 8, 17), today());
    let view = || date_view(&theme, &state);
    let mut h = Harness::new(size());
    open(&mut h, view);
    let image = h.screenshot(view(), &theme, 1.0);
    let t = day_center(12);
    assert!(distance(image.at(Point::new(t.x, t.y - 20.0)), rgb(c.primary)) < 40);
    assert_eq!(
        image.at(Point::new(t.x, t.y - 12.0)),
        rgb(c.surface_container_high)
    );
    assert_eq!(
        image.at(Point::new(132.0 + 24.0, day_center(1).y)),
        rgb(c.surface_container_high)
    );
}

#[test]
fn keyboard_moves_the_focused_day_across_months() {
    let theme = Theme::light();
    let mut state = date_picker::State::new(Date::new(2025, 8, 31), today());
    let mut h = Harness::new(size());
    open(&mut h, || date_view(&theme, &state));
    let first = h.click(|| date_view(&theme, &state), day_center(31));
    assert_eq!(first.len(), 1);
    let moved = h.update(date_view(&theme, &state), &key(Named::ArrowRight, false));
    assert_eq!(moved, vec![Message::Date(DateEvent::Month(1))]);
    state.update(DateEvent::Month(1));
    let enter = h.update(date_view(&theme, &state), &key(Named::Enter, false));
    assert_eq!(
        enter,
        vec![Message::Date(DateEvent::Pick(
            Date::new(2025, 9, 1).unwrap()
        ))]
    );
}

#[test]
fn month_arrows_and_ok_follow_the_selection() {
    let theme = Theme::light();
    let empty = date_picker::State::new(None, today());
    let chosen = date_picker::State::new(Date::new(2025, 8, 17), today());
    let mut h = Harness::new(size());
    open(&mut h, || date_view(&theme, &empty));
    let stops = h.focusables(date_view(&theme, &empty));
    let with = h.focusables(date_view(&theme, &chosen));
    assert_eq!(with.len(), stops.len() + 1);
    let prev = Point::new(
        120.0 + 360.0 - 12.0 - 40.0 - 40.0 + 20.0,
        66.0 + 121.0 + 28.0,
    );
    let next = Point::new(prev.x + 40.0, prev.y);
    assert_eq!(
        h.click(|| date_view(&theme, &chosen), prev),
        vec![Message::Date(DateEvent::Month(-1))]
    );
    assert_eq!(
        h.click(|| date_view(&theme, &chosen), next),
        vec![Message::Date(DateEvent::Month(1))]
    );
}

#[test]
fn year_list_and_text_input_modes() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut state = date_picker::State::new(Date::new(2025, 8, 17), today());
    state.update(DateEvent::ToggleYears);
    let mut h = Harness::new(size());
    open(&mut h, || date_view(&theme, &state));
    let image = h.screenshot(date_view(&theme, &state), &theme, 1.0);
    let list_top = 66.0 + 121.0 + 56.0;
    let found = (0..336)
        .map(|y| Point::new(132.0 + 224.0 + 56.0, list_top + y as f32))
        .find(|p| image.at(*p) == rgb(c.primary));
    assert!(found.is_some());
    let click = found.unwrap();
    let messages = h.click(
        || date_view(&theme, &state),
        Point::new(click.x, click.y + 6.0),
    );
    assert_eq!(messages, vec![Message::Date(DateEvent::Year(2025))]);

    state.update(DateEvent::Year(2025));
    state.update(DateEvent::ToggleInput);
    let view = || date_view(&theme, &state);
    open(&mut h, view);
    let stops = h.focusables(view());
    assert!(stops.iter().any(|r| r.height == 24.0));
    let field = stops.iter().find(|r| r.height == 24.0).unwrap();
    h.click(
        view,
        Point::new(field.x + field.width - 2.0, field.center_y()),
    );
    let typed = h.update(view(), &type_text("1"));
    assert_eq!(
        typed,
        vec![Message::Date(DateEvent::Text("08/17/20251".into()))]
    );
}

#[test]
fn docked_picker_is_456_dp_high_without_a_header() {
    let theme = Theme::light();
    let c = theme.colors;
    let state = date_picker::State::new(Date::new(2025, 8, 17), today());
    let view = || {
        date_picker::docked(&state, today(), Message::Date)
            .on_cancel(Message::Cancel)
            .on_confirm(Message::Confirm)
            .build(&theme)
    };
    let mut h = Harness::new(Size::new(400.0, 500.0));
    let image = h.screenshot(view(), &theme, 1.0);
    let inside = Point::new(200.0, 20.0);
    assert_eq!(image.at(inside), rgb(c.surface_container_high));
    assert_eq!(run(&image, Point::new(200.0, 30.0), 0, 1) + 31, 456);
    assert_eq!(run(&image, Point::new(200.0, 58.0), 1, 0) + 201, 360);
}

fn time_view<'a>(theme: &Theme, state: &'a time_picker::State) -> Element<'a, Message> {
    time_picker(state, Message::Time)
        .on_cancel(Message::Cancel)
        .on_confirm(Message::Confirm)
        .over(theme, base(), true, Some(Message::Cancel))
}

#[test]
fn time_state_follows_events() {
    let mut s = time_picker::State::new(9, 5, false);
    assert_eq!((s.hour(), s.minute(), s.select()), (9, 5, Select::Hour));
    s.update(TimeEvent::Period(true));
    assert_eq!(s.hour(), 21);
    s.update(TimeEvent::Period(false));
    assert_eq!(s.hour(), 9);
    s.update(TimeEvent::HourDone(11));
    assert_eq!((s.hour(), s.select()), (11, Select::Minute));
    s.update(TimeEvent::Minute(47));
    assert_eq!(s.minute(), 47);
    s.update(TimeEvent::Period(true));
    s.update(TimeEvent::HourText("12".into()));
    assert_eq!(s.hour(), 12);
    s.update(TimeEvent::HourText("1".into()));
    assert_eq!(s.hour(), 13);
    s.update(TimeEvent::HourText("13".into()));
    assert_eq!(s.hour(), 13);
    s.update(TimeEvent::MinuteText("61".into()));
    assert_eq!(s.minute(), 47);
    s.update(TimeEvent::MinuteText("8".into()));
    assert_eq!(s.minute(), 8);
    s.update(TimeEvent::HourText("123".into()));
    assert_eq!(s.hour(), 15);
    let mut s = time_picker::State::new(0, 0, true);
    s.update(TimeEvent::HourText("23".into()));
    assert_eq!(s.hour(), 23);
    s.update(TimeEvent::HourText("24".into()));
    assert_eq!(s.hour(), 23);
}

#[test]
fn dial_clicks_pick_hours_then_minutes() {
    let theme = Theme::light();
    let mut state = time_picker::State::new(0, 0, false);
    let mut h = Harness::new(size());
    open(&mut h, || time_view(&theme, &state));
    let stops = h.focusables(time_view(&theme, &state));
    let handle = stops
        .iter()
        .find(|r| r.width == 48.0 && r.height == 48.0)
        .unwrap();
    let (cx, cy) = (handle.center_x(), handle.center_y() + 101.0);
    let messages = h.click(|| time_view(&theme, &state), Point::new(cx + 101.0, cy));
    assert_eq!(
        messages,
        vec![
            Message::Time(TimeEvent::Hour(3)),
            Message::Time(TimeEvent::HourDone(3))
        ]
    );
    state.update(TimeEvent::HourDone(3));
    assert_eq!(state.select(), Select::Minute);
    let stops = h.focusables(time_view(&theme, &state));
    let handle = stops
        .iter()
        .find(|r| r.width == 48.0 && r.height == 48.0)
        .unwrap();
    let (cx, cy) = (handle.center_x(), handle.center_y() + 101.0);
    let messages = h.click(|| time_view(&theme, &state), Point::new(cx, cy + 101.0));
    assert_eq!(
        messages,
        vec![
            Message::Time(TimeEvent::Minute(30)),
            Message::Time(TimeEvent::Minute(30))
        ]
    );
}

#[test]
fn twenty_four_hour_dial_has_an_inner_ring() {
    let theme = Theme::light();
    let state = time_picker::State::new(0, 0, true);
    let mut h = Harness::new(size());
    open(&mut h, || time_view(&theme, &state));
    let stops = h.focusables(time_view(&theme, &state));
    let handle = stops
        .iter()
        .find(|r| r.width == 48.0 && r.height == 48.0)
        .unwrap();
    let (cx, cy) = (handle.center_x(), handle.center_y() + 69.0);
    let inner = h.click(|| time_view(&theme, &state), Point::new(cx + 69.0, cy));
    assert_eq!(inner[0], Message::Time(TimeEvent::Hour(15)));
    let outer = h.click(|| time_view(&theme, &state), Point::new(cx + 101.0, cy));
    assert_eq!(outer[0], Message::Time(TimeEvent::Hour(3)));
}

#[test]
fn period_and_selector_buttons_publish() {
    let theme = Theme::light();
    let state = time_picker::State::new(9, 5, false);
    let mut h = Harness::new(size());
    open(&mut h, || time_view(&theme, &state));
    let stops = h.focusables(time_view(&theme, &state));
    let tall: Vec<_> = stops.iter().filter(|r| r.height == 80.0).collect();
    assert_eq!(tall.len(), 2, "{stops:?}");
    let minute = tall[1].center();
    assert_eq!(
        h.click(|| time_view(&theme, &state), minute),
        vec![Message::Time(TimeEvent::Select(Select::Minute))]
    );
    let pm = stops
        .iter()
        .find(|r| r.width == 52.0 && r.y > 100.0 && r.height == 41.0)
        .unwrap();
    assert_eq!(
        h.click(|| time_view(&theme, &state), pm.center()),
        vec![Message::Time(TimeEvent::Period(true))]
    );
}

#[test]
fn time_picker_contains_the_period_selector_in_every_mode() {
    let theme = Theme::light();
    let c = theme.colors;
    for input in [false, true] {
        let mut state = time_picker::State::new(9, 41, false);
        if input {
            state.update(TimeEvent::ToggleInput);
        }
        let mut h = Harness::new(size());
        open(&mut h, || time_view(&theme, &state));
        let image = h.screenshot(time_view(&theme, &state), &theme, 1.0);
        let stops = h.focusables(time_view(&theme, &state));
        let pm = stops
            .iter()
            .filter(|r| r.width == 52.0)
            .max_by(|a, b| a.y.total_cmp(&b.y))
            .unwrap();
        let probe = Point::new(pm.x - 8.0, pm.y - 60.0);
        assert_eq!(image.at(probe), rgb(c.surface_container_high));
        let right = probe.x + run(&image, probe, 1, 0) as f32 + 1.0;
        assert!(
            pm.x + pm.width + 24.0 <= right + 1.0,
            "input {input}: {} vs {right}",
            pm.x + pm.width
        );
    }
}

#[test]
fn dial_handle_hides_the_labels_below_it() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut state = time_picker::State::new(9, 21, false);
    state.update(TimeEvent::Select(Select::Minute));
    let mut h = Harness::new(size());
    open(&mut h, || time_view(&theme, &state));
    let stops = h.focusables(time_view(&theme, &state));
    let handle = stops
        .iter()
        .find(|r| r.width == 48.0 && r.height == 48.0)
        .unwrap();
    let image = h.screenshot(time_view(&theme, &state), &theme, 1.0);
    let center = handle.center();
    for dy in -17i32..=17 {
        for dx in -17i32..=17 {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if (6.0..=17.0).contains(&d) {
                let p = Point::new(center.x + dx as f32, center.y + dy as f32);
                assert_eq!(image.at(p), rgb(c.primary), "{dx} {dy}");
            }
        }
    }
}
