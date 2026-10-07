// SPDX-License-Identifier: LGPL-3.0-only

use iced::widget::{column, container, mouse_area, row, text, text_input};
use iced::{Color, Element, Length, mouse};
use material_iced::Theme;
use material_iced::color::{SpecVersion, Variant};
use material_iced::typography::TypeStyle;

use crate::{Gallery, Message, Mode, Page};

pub fn styled<'a>(content: impl text::IntoFragment<'a>, style: TypeStyle) -> text::Text<'a, Theme> {
    text(content)
        .size(style.size)
        .line_height(text::LineHeight::Absolute(style.line_height.into()))
        .font(style.font)
}

fn choice<'a>(
    label: &'a str,
    selected: bool,
    message: Message,
    theme: &Theme,
) -> Element<'a, Message, Theme> {
    let label_style = theme.typography.label_large;
    let chip = container(styled(label, label_style))
        .padding([6, 12])
        .style(move |t: &Theme| {
            let c = &t.colors;
            container::Style {
                background: selected.then(|| c.secondary_container.into()),
                text_color: Some(if selected {
                    c.on_secondary_container
                } else {
                    c.on_surface_variant
                }),
                border: iced::border::rounded(8)
                    .width(if selected { 0 } else { 1 })
                    .color(c.outline),
                ..container::Style::default()
            }
        });
    mouse_area(chip)
        .on_press(message)
        .interaction(mouse::Interaction::Pointer)
        .into()
}

fn section<'a>(
    title: &'a str,
    items: Vec<Element<'a, Message, Theme>>,
    theme: &Theme,
) -> Element<'a, Message, Theme> {
    column![
        styled(title, theme.typography.title_small),
        row(items).spacing(8).wrap()
    ]
    .spacing(8)
    .into()
}

pub fn view<'a>(gallery: &'a Gallery, theme: &Theme) -> Element<'a, Message, Theme> {
    let seeds = [
        ("Baseline", Color::from_rgb8(0x67, 0x50, 0xA4)),
        ("Blue", Color::from_rgb8(0x0B, 0x57, 0xD0)),
        ("Green", Color::from_rgb8(0x1B, 0x6D, 0x00)),
        ("Teal", Color::from_rgb8(0x00, 0x79, 0x6B)),
        ("Red", Color::from_rgb8(0xB3, 0x26, 0x1E)),
        ("Yellow", Color::from_rgb8(0xFF, 0xDE, 0x3F)),
    ];
    let variants = [
        ("Tonal spot", Variant::TonalSpot),
        ("Neutral", Variant::Neutral),
        ("Vibrant", Variant::Vibrant),
        ("Expressive", Variant::Expressive),
        ("Fidelity", Variant::Fidelity),
        ("Content", Variant::Content),
        ("Rainbow", Variant::Rainbow),
        ("Fruit salad", Variant::FruitSalad),
        ("Monochrome", Variant::Monochrome),
    ];
    let contrasts = [
        ("Reduced", -1.0),
        ("Standard", 0.0),
        ("Medium", 0.5),
        ("High", 1.0),
    ];
    let modes = [
        ("System", Mode::System),
        ("Light", Mode::Light),
        ("Dark", Mode::Dark),
    ];
    let pages = [
        ("Color", Page::Color),
        ("Typography", Page::Typography),
        ("Shape", Page::Shape),
        ("Elevation", Page::Elevation),
        ("Actions", Page::Actions),
        ("Chips", Page::Chips),
        ("Containment", Page::Containment),
        ("Selection", Page::Selection),
        ("Text fields", Page::TextFields),
        ("Progress", Page::Progress),
        ("Lists", Page::Lists),
        ("Overlays", Page::Overlays),
        ("Navigation", Page::Navigation),
        ("Pickers", Page::Pickers),
        ("Adaptive", Page::Adaptive),
        ("Motion", Page::Motion),
    ];
    let o = &gallery.options;
    column![
        section(
            "Pages",
            pages
                .iter()
                .map(|(l, p)| choice(l, gallery.page == *p, Message::Page(*p), theme))
                .collect(),
            theme
        ),
        section(
            "Seed",
            seeds
                .iter()
                .map(|(l, c)| choice(l, o.source == *c, Message::Seed(*c), theme))
                .collect(),
            theme
        ),
        text_input("#RRGGBB", &gallery.seed_input)
            .on_input(Message::SeedInput)
            .padding(8),
        section(
            "Variant",
            variants
                .iter()
                .map(|(l, v)| choice(l, o.variant == *v, Message::Variant(*v), theme))
                .collect(),
            theme
        ),
        section(
            "Contrast",
            contrasts
                .iter()
                .map(|(l, c)| choice(l, o.contrast == *c, Message::Contrast(*c), theme))
                .collect(),
            theme
        ),
        section(
            "Spec",
            vec![
                choice(
                    "2021",
                    o.spec == SpecVersion::Spec2021,
                    Message::Spec(SpecVersion::Spec2021),
                    theme
                ),
                choice(
                    "2025",
                    o.spec == SpecVersion::Spec2025,
                    Message::Spec(SpecVersion::Spec2025),
                    theme
                ),
            ],
            theme
        ),
        section(
            "Mode",
            modes
                .iter()
                .map(|(l, m)| choice(l, gallery.mode == *m, Message::Mode(*m), theme))
                .collect(),
            theme
        ),
        styled("Matugen JSON", theme.typography.title_small),
        text_input("path/to/colors.json", &gallery.matugen_path)
            .on_input(Message::MatugenPath)
            .on_submit(Message::LoadMatugen)
            .padding(8),
        row![
            choice("Load", false, Message::LoadMatugen, theme),
            choice(
                "Clear",
                gallery.matugen.is_none(),
                Message::ClearMatugen,
                theme
            ),
        ]
        .spacing(8),
        styled(gallery.status.as_str(), theme.typography.body_small),
    ]
    .spacing(16)
    .width(Length::Fill)
    .into()
}
