// SPDX-License-Identifier: LGPL-3.0-only

//! Material Symbols icons rendered as SVG.
//!
//! Iced can only drive the weight axis of a variable font, so Material
//! Symbols are rendered from the SVG files Google publishes for each
//! combination of fill, weight, grade and optical size. The bundled set
//! covers the icons used by this crate in the outlined style at weight 400,
//! grade 0 and optical size 24. Other icons can be loaded with
//! [`svg::Handle::from_path`] or [`svg::Handle::from_memory`].

use std::sync::LazyLock;

use iced::widget::svg::{self, Svg};
use iced::{Color, Length};

use crate::theme::Theme;

/// An icon of `size` logical pixels tinted with `color`.
pub fn icon<'a>(handle: svg::Handle, size: f32, color: Color) -> Svg<'a, Theme> {
    Svg::new(handle)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .opacity(color.a)
        .style(move |_, _| svg::Style {
            color: Some(Color { a: 1.0, ..color }),
        })
}

/// An SVG tinted with `color`. The alpha of `color` becomes the opacity, because Iced drops it from tints.
pub(crate) fn tinted(handle: svg::Handle, color: Color, opacity: f32) -> iced::advanced::svg::Svg {
    iced::advanced::svg::Svg {
        color: Some(Color { a: 1.0, ..color }),
        opacity: opacity * color.a,
        ..iced::advanced::svg::Svg::new(handle)
    }
}

macro_rules! symbols {
    ($($name:ident => $file:literal,)*) => {
        /// Bundled Material Symbols, outlined style.
        pub mod symbol {
            use super::*;

            $(
                #[allow(missing_docs)]
                pub fn $name(filled: bool) -> svg::Handle {
                    static OUTLINED: LazyLock<svg::Handle> = LazyLock::new(|| {
                        svg::Handle::from_memory(include_bytes!(concat!("../icons/", $file, ".svg")).as_slice())
                    });
                    static FILLED: LazyLock<svg::Handle> = LazyLock::new(|| {
                        svg::Handle::from_memory(include_bytes!(concat!("../icons/", $file, "_fill1.svg")).as_slice())
                    });
                    if filled { FILLED.clone() } else { OUTLINED.clone() }
                }
            )*
        }
    };
}

symbols! {
    add => "add",
    arrow_back => "arrow_back",
    arrow_drop_down => "arrow_drop_down",
    arrow_drop_up => "arrow_drop_up",
    calendar_today => "calendar_today",
    check => "check",
    chevron_left => "chevron_left",
    chevron_right => "chevron_right",
    close => "close",
    edit => "edit",
    error => "error",
    keyboard => "keyboard",
    keyboard_arrow_down => "keyboard_arrow_down",
    keyboard_arrow_up => "keyboard_arrow_up",
    menu => "menu",
    more_vert => "more_vert",
    schedule => "schedule",
    search => "search",
}
