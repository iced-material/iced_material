// SPDX-License-Identifier: LGPL-3.0-only

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::widget::{column, container, row};
use iced::{Element, Length, Rectangle, Renderer, Size, mouse};
use iced_material::Theme;
use iced_material::color::Role;
use iced_material::draw::{shadow, surface};
use iced_material::shape::Shape;
use iced_material::typography::TypeStyle;

use crate::Message;
use crate::controls::styled;

fn heading<'a>(title: &'a str, theme: &Theme) -> Element<'a, Message, Theme> {
    styled(title, theme.typography.headline_small).into()
}

fn swatch<'a>(background: Role, foreground: Role, theme: &Theme) -> Element<'a, Message, Theme> {
    let label = styled(background.name(), theme.typography.label_medium);
    container(label)
        .width(Length::Fixed(200.0))
        .height(Length::Fixed(64.0))
        .padding(8)
        .style(move |t: &Theme| container::Style {
            background: Some(t.colors.role(background).into()),
            text_color: Some(t.colors.role(foreground)),
            ..container::Style::default()
        })
        .into()
}

pub fn color(theme: &Theme) -> Element<'static, Message, Theme> {
    let pairs = [
        (Role::Primary, Role::OnPrimary),
        (Role::OnPrimary, Role::Primary),
        (Role::PrimaryContainer, Role::OnPrimaryContainer),
        (Role::OnPrimaryContainer, Role::PrimaryContainer),
        (Role::Secondary, Role::OnSecondary),
        (Role::OnSecondary, Role::Secondary),
        (Role::SecondaryContainer, Role::OnSecondaryContainer),
        (Role::OnSecondaryContainer, Role::SecondaryContainer),
        (Role::Tertiary, Role::OnTertiary),
        (Role::OnTertiary, Role::Tertiary),
        (Role::TertiaryContainer, Role::OnTertiaryContainer),
        (Role::OnTertiaryContainer, Role::TertiaryContainer),
        (Role::Error, Role::OnError),
        (Role::OnError, Role::Error),
        (Role::ErrorContainer, Role::OnErrorContainer),
        (Role::OnErrorContainer, Role::ErrorContainer),
        (Role::PrimaryFixed, Role::OnPrimaryFixed),
        (Role::PrimaryFixedDim, Role::OnPrimaryFixedVariant),
        (Role::SecondaryFixed, Role::OnSecondaryFixed),
        (Role::SecondaryFixedDim, Role::OnSecondaryFixedVariant),
        (Role::TertiaryFixed, Role::OnTertiaryFixed),
        (Role::TertiaryFixedDim, Role::OnTertiaryFixedVariant),
        (Role::SurfaceDim, Role::OnSurface),
        (Role::Surface, Role::OnSurface),
        (Role::SurfaceBright, Role::OnSurface),
        (Role::SurfaceContainerLowest, Role::OnSurface),
        (Role::SurfaceContainerLow, Role::OnSurface),
        (Role::SurfaceContainer, Role::OnSurface),
        (Role::SurfaceContainerHigh, Role::OnSurface),
        (Role::SurfaceContainerHighest, Role::OnSurface),
        (Role::OnSurface, Role::Surface),
        (Role::OnSurfaceVariant, Role::Surface),
        (Role::Outline, Role::Surface),
        (Role::OutlineVariant, Role::OnSurface),
        (Role::InverseSurface, Role::InverseOnSurface),
        (Role::InverseOnSurface, Role::InverseSurface),
        (Role::InversePrimary, Role::OnPrimaryContainer),
        (Role::Scrim, Role::InverseOnSurface),
    ];
    column![
        heading("Color roles", theme),
        iced::widget::row(pairs.iter().map(|(b, f)| swatch(*b, *f, theme))).wrap(),
    ]
    .spacing(16)
    .into()
}

pub fn typography(theme: &Theme) -> Element<'static, Message, Theme> {
    let t = &theme.typography;
    let roles: [(&'static str, TypeStyle); 17] = [
        ("Display large", t.display_large),
        ("Display medium", t.display_medium),
        ("Display small", t.display_small),
        ("Headline large", t.headline_large),
        ("Headline medium", t.headline_medium),
        ("Headline small", t.headline_small),
        ("Title large", t.title_large),
        ("Title medium", t.title_medium),
        ("Title small", t.title_small),
        ("Body large", t.body_large),
        ("Body medium", t.body_medium),
        ("Body small", t.body_small),
        ("Label large", t.label_large),
        ("Label medium", t.label_medium),
        ("Label small", t.label_small),
        ("Label large prominent", t.label_large_prominent),
        ("Label medium prominent", t.label_medium_prominent),
    ];
    let mut list = column![heading("Type scale", theme)].spacing(12);
    for (name, style) in roles {
        let spec = format!(
            "{} / {} / {:?} / {}",
            style.size, style.line_height, style.font.weight, style.tracking
        );
        list = list.push(row![
            container(styled(name, style)).width(Length::Fill),
            styled(spec, theme.typography.body_small),
        ]);
    }
    list.into()
}

struct Tile {
    shape: Shape,
    level: f32,
}

impl Widget<Message, Theme, Renderer> for Tile {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(160.0), Length::Fixed(112.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(160.0, 112.0))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds().shrink(16);
        let radius = self.shape.radius(bounds.size());
        let c = &theme.colors;
        shadow::draw(
            renderer,
            bounds,
            radius,
            self.level,
            c.shadow,
            &theme.elevation,
        );
        let fill = if self.level > 0.0 {
            c.surface_container_low
        } else {
            c.primary_container
        };
        surface::fill(renderer, bounds, radius, fill);
    }
}

fn tile(
    shape: Shape,
    level: f32,
    label: &'static str,
    theme: &Theme,
) -> Element<'static, Message, Theme> {
    column![
        Element::new(Tile { shape, level }),
        styled(label, theme.typography.label_medium),
    ]
    .spacing(4)
    .into()
}

pub fn shape(theme: &Theme) -> Element<'static, Message, Theme> {
    let s = theme.shape;
    let shapes = [
        (s.none, "None 0"),
        (s.extra_small, "Extra small 4"),
        (s.extra_small_top, "Extra small top"),
        (s.small, "Small 8"),
        (s.medium, "Medium 12"),
        (s.large, "Large 16"),
        (s.large_top, "Large top"),
        (s.large_start, "Large start"),
        (s.large_end, "Large end"),
        (s.extra_large, "Extra large 28"),
        (s.extra_large_top, "Extra large top"),
        (s.full, "Full"),
    ];
    column![
        heading("Shape scale", theme),
        iced::widget::row(
            shapes
                .iter()
                .map(|(shape, label)| tile(*shape, 0.0, label, theme))
        )
        .wrap(),
    ]
    .spacing(16)
    .into()
}

pub fn elevation(theme: &Theme) -> Element<'static, Message, Theme> {
    let labels = [
        "Level 0", "Level 1", "Level 2", "Level 3", "Level 4", "Level 5",
    ];
    column![
        heading("Elevation", theme),
        iced::widget::row(labels.iter().enumerate().map(|(level, label)| tile(
            theme.shape.medium,
            level as f32,
            label,
            theme
        )))
        .wrap(),
    ]
    .spacing(16)
    .into()
}
