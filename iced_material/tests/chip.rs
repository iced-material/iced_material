// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use common::{Harness, key};
use iced::Size;
use iced::keyboard::key::Named;
use iced::widget::container;
use iced_material::draw::text::Label;
use iced_material::icon::symbol;
use iced_material::widget::chip::{self, Kind, Variant};
use iced_material::widget::chip_set::chip_set;
use iced_material::widget::focus_scope;
use iced_material::widget::pressable::Status;
use iced_material::{Element, Theme};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Press(usize),
    Remove(usize),
}

fn label_width(theme: &Theme, text: &str) -> f32 {
    Label::default()
        .update(text, theme.typography.label_large)
        .width
}

fn place(content: Element<'static, Message>) -> Element<'static, Message> {
    focus_scope(container(content).padding(20)).into()
}

#[test]
fn geometry_matches_tokens() {
    let theme = Theme::light();
    let mut h = Harness::new(Size::new(400.0, 120.0));
    let w = label_width(&theme, "Chip");
    let cases: [(chip::Chip<'static, Message>, f32); 4] = [
        (chip::assist(&theme, "Chip"), 16.0 + w + 16.0),
        (
            chip::assist(&theme, "Chip").icon(symbol::add(false)),
            8.0 + 18.0 + 8.0 + w + 16.0,
        ),
        (
            chip::filter(&theme, "Chip").selected(true),
            8.0 + 18.0 + 8.0 + w + 16.0,
        ),
        (
            chip::filter(&theme, "Chip").trailing_icon(symbol::arrow_drop_down(false)),
            16.0 + w + 8.0 + 18.0 + 8.0,
        ),
    ];
    for (chip, width) in cases {
        let bounds = h.focusables(place(chip.on_press(Message::Press(0)).into()));
        assert_eq!(bounds[0].height, 32.0);
        assert!(
            (bounds[0].width - width).abs() < 0.01,
            "{} vs {width}",
            bounds[0].width
        );
    }
}

#[test]
fn filter_chip_tokens() {
    let theme = Theme::light();
    let c = theme.colors;
    let v = |selected, elevated| Variant {
        kind: Kind::Filter,
        selected,
        elevated,
    };
    let unselected = chip::style(&theme, Status::Active, v(false, false));
    assert_eq!(unselected.base.outline_width, 1.0);
    assert_eq!(unselected.base.label, c.on_surface_variant);
    assert_eq!(unselected.base.pressed_layer, c.on_secondary_container);
    let selected = chip::style(&theme, Status::Active, v(true, false));
    assert_eq!(selected.base.container, c.secondary_container);
    assert_eq!(selected.base.outline_width, 0.0);
    assert_eq!(
        chip::style(&theme, Status::Hovered, v(true, false))
            .base
            .elevation,
        1.0
    );
    assert_eq!(
        chip::style(&theme, Status::Hovered, v(false, false))
            .base
            .elevation,
        0.0
    );
    assert_eq!(
        chip::style(&theme, Status::Hovered, v(false, true))
            .base
            .elevation,
        2.0
    );
    assert_eq!(
        chip::style(&theme, Status::Active, v(false, true))
            .base
            .container,
        c.surface_container_low
    );
}

#[test]
fn input_chip_remove_action_is_separate() {
    let theme = Theme::light();
    let view = || {
        place(
            chip::input(&theme, "Chip")
                .on_press(Message::Press(0))
                .on_remove(Message::Remove(0))
                .into(),
        )
    };
    let mut h = Harness::new(Size::new(400.0, 120.0));
    let bounds = h.focusables(view());
    assert_eq!(bounds.len(), 2);
    assert_eq!(bounds[1].size(), Size::new(24.0, 24.0));
    assert_eq!(h.click(view, bounds[1].center()), vec![Message::Remove(0)]);
    assert_eq!(
        h.click(
            view,
            iced::Point::new(bounds[0].x + 10.0, bounds[0].center_y())
        ),
        vec![Message::Press(0)]
    );
    h.update(view(), &key(Named::Tab, false));
    h.update(view(), &key(Named::ArrowRight, false));
    assert_eq!(
        h.update(view(), &key(Named::Enter, false)),
        vec![Message::Remove(0)]
    );
}

#[test]
fn chip_set_arrow_keys_skip_disabled_and_wrap() {
    let theme = Theme::light();
    let view = || {
        place(
            chip_set(
                &theme,
                (0..4).map(|i| {
                    chip::assist(&theme, format!("Chip {i}"))
                        .on_press_maybe((i != 2).then_some(Message::Press(i)))
                        .into()
                }),
            )
            .into(),
        )
    };
    let mut h = Harness::new(Size::new(600.0, 120.0));
    let enter = |h: &mut Harness| h.update(view(), &key(Named::Enter, false));
    h.update(view(), &key(Named::Tab, false));
    h.update(view(), &key(Named::ArrowRight, false));
    assert_eq!(enter(&mut h), vec![Message::Press(1)]);
    h.update(view(), &key(Named::ArrowRight, false));
    assert_eq!(enter(&mut h), vec![Message::Press(3)]);
    h.update(view(), &key(Named::ArrowRight, false));
    assert_eq!(enter(&mut h), vec![Message::Press(0)]);
    h.update(view(), &key(Named::End, false));
    assert_eq!(enter(&mut h), vec![Message::Press(3)]);
}
