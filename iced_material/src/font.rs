// SPDX-License-Identifier: LGPL-3.0-only

//! Bundled Roboto fonts.
//!
//! The files are the static Roboto 3.016 instances from
//! `googlefonts/roboto-classic`, licensed under the SIL Open Font License 1.1.

/// Roboto Regular, Medium and Bold. Pass each to `iced::application(..).font(..)`.
pub const ROBOTO: [&[u8]; 3] = [
    include_bytes!("../fonts/Roboto-Regular.ttf"),
    include_bytes!("../fonts/Roboto-Medium.ttf"),
    include_bytes!("../fonts/Roboto-Bold.ttf"),
];
