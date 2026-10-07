// SPDX-License-Identifier: LGPL-3.0-only

//! The Material theme and its interop with [`iced::Theme`].

use iced::theme::{Base, Mode, Palette, Style};
use iced::time::Instant;

use crate::color::{ColorScheme, SchemeOptions};
use crate::elevation::Elevation;
use crate::motion::{Motion, Transition};
use crate::shape::ShapeScale;
use crate::state::{Disabled, FocusRing, Ripple, StateLayers};
use crate::typography::TypeScale;
use crate::widget::{
    badge, button, checkbox, chip, chip_set, date_picker, dialog, divider, fab, icon_button, list,
    menu, navigation, navigation_drawer, progress, radio, search, segmented_button, side_sheet,
    slider, snackbar, switch, tabs, text_field, time_picker, tooltip, top_app_bar, transition,
};

/// All design tokens of the Material system.
///
/// Every widget of this crate reads its colors, type, shape, elevation,
/// motion and state values from here.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Resolved color roles.
    pub colors: ColorScheme,
    /// Whether `colors` is a dark scheme.
    pub dark: bool,
    /// Type scale.
    pub typography: TypeScale,
    /// Shape scale.
    pub shape: ShapeScale,
    /// Elevation levels and shadows.
    pub elevation: Elevation,
    /// Easing curves and durations.
    pub motion: Motion,
    /// State layer opacities.
    pub state: StateLayers,
    /// Disabled opacities.
    pub disabled: Disabled,
    /// Focus indicator tokens.
    pub focus_ring: FocusRing,
    /// Press ripple tokens.
    pub ripple: Ripple,
    /// Component dimensions.
    pub components: Components,
}

/// Dimensions of the components in this crate.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Components {
    pub button: button::Metrics,
    pub icon_button: icon_button::Metrics,
    pub fab: fab::Metrics,
    pub segmented_button: segmented_button::Metrics,
    pub chip: chip::Metrics,
    pub chip_set: chip_set::Metrics,
    pub divider: divider::Metrics,
    pub badge: badge::Metrics,
    pub checkbox: checkbox::Metrics,
    pub radio: radio::Metrics,
    pub switch: switch::Metrics,
    pub slider: slider::Metrics,
    pub text_field: text_field::Metrics,
    pub progress: progress::Metrics,
    pub list: list::Metrics,
    pub tooltip: tooltip::Metrics,
    pub dialog: dialog::Metrics,
    pub menu: menu::Metrics,
    pub snackbar: snackbar::Metrics,
    pub navigation: navigation::Metrics,
    pub navigation_drawer: navigation_drawer::Metrics,
    pub tabs: tabs::Metrics,
    pub top_app_bar: top_app_bar::Metrics,
    pub search: search::Metrics,
    pub side_sheet: side_sheet::Metrics,
    pub date_picker: date_picker::Metrics,
    pub time_picker: time_picker::Metrics,
    pub transition: transition::Metrics,
}

impl Theme {
    /// Builds a theme from a generated color scheme and the baseline tokens.
    pub fn new(options: SchemeOptions) -> Theme {
        Theme::with_colors(ColorScheme::new(options), options.dark)
    }

    /// Builds a theme from explicit colors and the baseline tokens.
    pub fn with_colors(colors: ColorScheme, dark: bool) -> Theme {
        Theme {
            colors,
            dark,
            typography: TypeScale::default(),
            shape: ShapeScale::default(),
            elevation: Elevation::default(),
            motion: Motion::default(),
            state: StateLayers::default(),
            disabled: Disabled::default(),
            focus_ring: FocusRing::default(),
            ripple: Ripple::default(),
            components: Components::default(),
        }
    }

    /// The baseline light theme.
    pub fn light() -> Theme {
        Theme::new(SchemeOptions::default())
    }

    /// The baseline dark theme.
    pub fn dark() -> Theme {
        Theme::new(SchemeOptions {
            dark: true,
            ..SchemeOptions::default()
        })
    }

    /// A palette for stock Iced widgets.
    ///
    /// Iced has six palette slots. `success` and `warning` use the success
    /// and warning custom colors of the scheme. Stock widgets styled this way
    /// do not follow Material component specs.
    pub fn iced_palette(&self) -> Palette {
        Palette {
            background: self.colors.surface,
            text: self.colors.on_surface,
            primary: self.colors.primary,
            success: self.colors.success.color,
            warning: self.colors.warning.color,
            danger: self.colors.error,
        }
    }

    /// An [`iced::Theme`] built from [`Theme::iced_palette`].
    pub fn iced_theme(&self) -> iced::Theme {
        iced::Theme::custom(self.name().to_string(), self.iced_palette())
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::light()
    }
}

impl Base for Theme {
    fn default(preference: Mode) -> Self {
        match preference {
            Mode::Dark => Theme::dark(),
            Mode::Light | Mode::None => Theme::light(),
        }
    }

    fn mode(&self) -> Mode {
        if self.dark { Mode::Dark } else { Mode::Light }
    }

    fn base(&self) -> Style {
        Style {
            background_color: self.colors.surface,
            text_color: self.colors.on_surface,
        }
    }

    fn palette(&self) -> Option<Palette> {
        Some(self.iced_palette())
    }

    fn name(&self) -> &str {
        if self.dark {
            "Material Dark"
        } else {
            "Material Light"
        }
    }
}

/// An animated change from one color scheme to another.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorTransition {
    from: ColorScheme,
    to: ColorScheme,
    start: Instant,
    transition: Transition,
}

impl ColorTransition {
    /// Starts a transition at `start`.
    pub fn new(from: ColorScheme, to: ColorScheme, start: Instant, transition: Transition) -> Self {
        ColorTransition {
            from,
            to,
            start,
            transition,
        }
    }

    /// Colors at `now`.
    pub fn colors(&self, now: Instant) -> ColorScheme {
        let t = self
            .transition
            .progress(now.saturating_duration_since(self.start));
        self.from.mix(&self.to, t)
    }

    /// Whether the transition is still running at `now`.
    pub fn is_running(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.start) < self.transition.duration
    }
}
