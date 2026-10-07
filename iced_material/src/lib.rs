// SPDX-License-Identifier: LGPL-3.0-only

//! Material Design 3 widgets and theming for Iced.

mod catalog;
mod interaction;

pub mod calendar;
pub mod color;
pub mod draw;
pub mod elevation;
#[cfg(feature = "roboto")]
pub mod font;
pub mod icon;
pub mod layout;
pub mod motion;
pub mod shape;
pub mod state;
pub mod theme;
pub mod typography;
pub mod widget;

pub use theme::Theme;

/// An [`iced::Element`] using the Material [`Theme`].
pub type Element<'a, Message> = iced::Element<'a, Message, Theme, iced::Renderer>;
