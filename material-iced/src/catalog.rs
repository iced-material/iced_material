// SPDX-License-Identifier: LGPL-3.0-only

use iced::border::{self, Border};
use iced::widget::{container, scrollable, svg, text, text_editor, text_input};
use iced::{Background, Color, Shadow};

use crate::state::alpha;
use crate::theme::Theme;

impl text::Catalog for Theme {
    type Class<'a> = text::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| text::Style { color: None })
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        class(self)
    }
}

impl container::Catalog for Theme {
    type Class<'a> = container::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| container::Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        class(self)
    }
}

impl svg::Catalog for Theme {
    type Class<'a> = svg::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_, _| svg::Style { color: None })
    }

    fn style(&self, class: &Self::Class<'_>, status: svg::Status) -> svg::Style {
        class(self, status)
    }
}

impl scrollable::Catalog for Theme {
    type Class<'a> = scrollable::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(scrollbar)
    }

    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        class(self, status)
    }
}

fn scrollbar(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let c = &theme.colors;
    let active = matches!(
        status,
        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
    );
    let scroller_color = if active {
        c.on_surface_variant
    } else {
        c.outline
    };
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_color),
            border: border::rounded(u32::MAX),
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(c.surface_container_high),
            border: border::rounded(u32::MAX).width(1).color(c.outline_variant),
            shadow: Shadow::default(),
            icon: c.on_surface_variant,
        },
    }
}

impl text_input::Catalog for Theme {
    type Class<'a> = text_input::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(bare_input)
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        class(self, status)
    }
}

fn bare_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let c = &theme.colors;
    let value = if matches!(status, text_input::Status::Disabled) {
        alpha(c.on_surface, theme.disabled.content)
    } else {
        c.on_surface
    };
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        icon: c.on_surface_variant,
        placeholder: c.on_surface_variant,
        value,
        selection: alpha(c.primary, 0.4),
    }
}

impl text_editor::Catalog for Theme {
    type Class<'a> = text_editor::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(bare_editor)
    }

    fn style(&self, class: &Self::Class<'_>, status: text_editor::Status) -> text_editor::Style {
        class(self, status)
    }
}

fn bare_editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let c = &theme.colors;
    let value = if matches!(status, text_editor::Status::Disabled) {
        alpha(c.on_surface, theme.disabled.content)
    } else {
        c.on_surface
    };
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        placeholder: c.on_surface_variant,
        value,
        selection: alpha(c.primary, 0.4),
    }
}
