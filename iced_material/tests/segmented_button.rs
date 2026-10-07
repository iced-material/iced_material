// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, distance, key, rgb};
use iced::keyboard::key::Named;
use iced::widget::container;
use iced::{Point, Size};
use iced_material::draw::text::Label;
use iced_material::widget::focus_scope;
use iced_material::widget::segmented_button::{Segment, segmented_button};
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Pick(usize),
}

fn view(theme: &Theme, selected: usize) -> Element<'static, Message> {
    let segments = ["Day", "Week", "Month"]
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            Segment::new(label)
                .selected(i == selected)
                .on_press(Message::Pick(i))
        });
    focus_scope(container(segmented_button(theme, segments)).padding(20)).into()
}

#[test]
fn segments_share_the_widest_width_and_overlap_outlines() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(600.0, 120.0));
    let bounds = h.focusables(view(&theme, 0));
    assert_eq!(bounds.len(), 3);
    let mut label = Label::default();
    let widest = label.update("Month", theme.typography.label_large).width;
    let width = 12.0 + 18.0 + 8.0 + widest + 12.0;
    for (i, b) in bounds.iter().enumerate() {
        assert_eq!(b.height, 40.0);
        assert!((b.width - width).abs() < 0.01);
        assert!((b.x - (20.0 + i as f32 * (width - 1.0))).abs() < 0.01);
    }
}

#[test]
fn press_publishes_the_segment() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(600.0, 120.0));
    let bounds = h.focusables(view(&theme, 0));
    let center = bounds[2].center();
    assert_eq!(h.click(|| view(&theme, 0), center), vec![Message::Pick(2)]);
}

#[test]
fn selected_segment_uses_secondary_container() {
    let theme = Theme::light();
    let c = theme.colors;
    let mut h = Harness::new(Size::new(600.0, 120.0));
    let bounds = h.focusables(view(&theme, 1));
    h.frame(view(&theme, 1), Duration::ZERO);
    let image = h.screenshot(view(&theme, 1), &theme, 1.0);
    let inside = Point::new(bounds[1].x + 4.0, bounds[1].y + 4.0);
    assert!(distance(image.at(inside), rgb(c.secondary_container)) <= 1);
    let other = Point::new(bounds[0].x + 30.0, bounds[0].y + 4.0);
    assert!(distance(image.at(other), rgb(c.surface)) <= 1);
}

#[test]
fn arrow_keys_move_focus_and_wrap() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(600.0, 120.0));
    h.update(view(&theme, 0), &key(Named::Tab, false));
    h.update(view(&theme, 0), &key(Named::ArrowLeft, false));
    assert_eq!(
        h.update(view(&theme, 0), &key(Named::Enter, false)),
        vec![Message::Pick(2)]
    );
    h.update(view(&theme, 0), &key(Named::ArrowRight, false));
    assert_eq!(
        h.update(view(&theme, 0), &key(Named::Enter, false)),
        vec![Message::Pick(0)]
    );
}
