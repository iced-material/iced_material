// SPDX-License-Identifier: LGPL-3.0-only

mod common;

use std::time::Duration;

use common::{Harness, Image, distance, rgb};
use iced::widget::{column, container, row};
use iced::{Length, Point, Size};
use material_iced::icon::symbol;
use material_iced::widget::{button, card, fab, slider::slider, text_field};
use material_iced::{Element, Theme};

#[derive(Debug, Clone)]
enum Message {
    Nothing,
}

fn scene(theme: &Theme) -> Element<'static, Message> {
    container(
        column![
            row![
                button::filled(theme, "Filled").on_press(Message::Nothing),
                button::elevated(theme, "Elevated").on_press(Message::Nothing),
                button::outlined(theme, "Outlined").on_press(Message::Nothing),
                fab::fab(theme, symbol::add(false)).on_press(Message::Nothing),
            ]
            .spacing(16),
            card::elevated(theme, iced::widget::text("Card"))
                .padding(16)
                .width(Length::Fixed(200.0)),
            text_field::outlined(theme, "text")
                .label("Label")
                .width(Length::Fixed(240.0))
                .on_input(|_| Message::Nothing),
            slider(theme, 0.0..=100.0, 40.0)
                .width(Length::Fixed(240.0))
                .on_change(|_| Message::Nothing),
        ]
        .spacing(20),
    )
    .padding(24)
    .into()
}

fn difference(a: &Image, b: &Image) -> (f64, f64) {
    let (mut sum, mut large, mut total) = (0u64, 0u64, 0u64);
    for y in 0..a.height {
        for x in 0..a.width {
            let d = distance(a.pixel(x, y), b.pixel(x, y));
            sum += u64::from(d);
            large += u64::from(d > 48);
            total += 1;
        }
    }
    (sum as f64 / total as f64, large as f64 / total as f64)
}

#[test]
fn wgpu_and_tiny_skia_draw_the_same_scene() {
    let theme = Theme::light();
    let size = Size::new(520.0, 400.0);
    for scale in [1.0, 2.0] {
        let mut images = Vec::new();
        for backend in ["tiny-skia", "wgpu"] {
            let mut h = Harness::with_backend(size, backend);
            h.move_cursor(scene(&theme), Point::new(60.0, 40.0));
            h.frame(scene(&theme), Duration::ZERO);
            h.frame(scene(&theme), Duration::from_millis(200));
            images.push(h.screenshot(scene(&theme), &theme, scale));
        }
        let (mean, large) = difference(&images[0], &images[1]);
        assert!(mean < 3.0, "scale {scale}: mean difference {mean}");
        assert!(
            large < 0.01,
            "scale {scale}: {large} of the pixels differ strongly"
        );
    }
}

fn outline_rows(
    image: &Image,
    x: f32,
    from: f32,
    to: f32,
    outline: [u8; 4],
    bg: [u8; 4],
) -> (usize, usize) {
    let scale = image.scale;
    let mut solid = 0;
    let mut blended = 0;
    let px = (x * scale).round() as u32;
    for py in (from * scale) as u32..(to * scale) as u32 {
        let p = image.pixel(px, py);
        if distance(p, outline) <= 6 {
            solid += 1;
        } else if distance(p, bg) > 6 {
            blended += 1;
        }
    }
    (solid, blended)
}

#[test]
fn one_dp_outlines_are_crisp_at_whole_scales_and_one_row_soft_at_fractional_ones() {
    let theme = Theme::light();
    let c = theme.colors;
    let view = || -> Element<'static, Message> {
        container(
            column![
                button::outlined(&theme, "Outlined").on_press(Message::Nothing),
                card::outlined(&theme, iced::widget::text("Card"))
                    .padding(16)
                    .width(Length::Fixed(200.0)),
            ]
            .spacing(24),
        )
        .padding(24)
        .into()
    };
    for backend in ["tiny-skia", "wgpu"] {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let mut h = Harness::with_backend(Size::new(300.0, 200.0), backend);
            let image = h.screenshot(view(), &theme, scale);
            let whole = scale.fract() == 0.0;
            let edges = [(21.0, 27.0, c.outline), (84.0, 92.0, c.outline_variant)];
            for (from, to, color) in edges {
                let (solid, blended) =
                    outline_rows(&image, 60.0, from, to, rgb(color), rgb(c.surface));
                let allowed = if whole { 0 } else { 1 };
                assert!(
                    (1..=2).contains(&solid) && blended <= allowed,
                    "{backend} {scale}: edge at {from}: {solid} solid {blended} blended"
                );
            }
        }
    }
}
