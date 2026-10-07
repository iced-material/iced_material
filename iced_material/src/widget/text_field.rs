// SPDX-License-Identifier: LGPL-3.0-only

//! Filled and outlined text fields, single line and multi line.
//!
//! The text itself is edited by the stock Iced `TextInput` and `TextEditor`,
//! which provide the caret, selection, clipboard and input methods. Everything
//! around it is drawn by this crate.

use iced::widget::text_editor::Action;
use iced::widget::{text_editor as editor, text_input};
use iced::{Color, Length, Pixels, Renderer, widget};

use crate::Element;
use crate::state::alpha;
use crate::theme::Theme;
use crate::widget::field::{Field, Parts};
use crate::widget::pressable::Status;

/// Text field dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of the container of a single line field.
    pub height: f32,
    /// Start and end padding without icons.
    pub padding: f32,
    /// Start padding with a leading icon and end padding with a trailing icon.
    pub icon_padding: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Space between an icon and the text.
    pub icon_text_space: f32,
    /// Top and bottom padding of the text when there is no label.
    pub vertical_padding: f32,
    /// Top and bottom padding of the text of a filled field that has a label.
    pub label_vertical_padding: f32,
    /// Line height of the text and of the resting label.
    pub line_height: f32,
    /// Line height of the floating label.
    pub floating_label_line_height: f32,
    /// Space between a prefix or suffix and the text.
    pub affix_space: f32,
    /// Space between the container and the supporting text.
    pub supporting_text_top_space: f32,
    /// Start and end padding of the supporting text.
    pub supporting_text_padding: f32,
    /// Space on each side of the label in the outline gap.
    pub outline_label_padding: f32,
    /// Width used when the field is not given one and has no wider content.
    pub min_width: f32,
    /// Rows shown by a multi line field.
    pub rows: usize,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            height: 56.0,
            padding: 16.0,
            icon_padding: 12.0,
            icon_size: 24.0,
            icon_text_space: 16.0,
            vertical_padding: 16.0,
            label_vertical_padding: 8.0,
            line_height: 24.0,
            floating_label_line_height: 16.0,
            affix_space: 2.0,
            supporting_text_top_space: 4.0,
            supporting_text_padding: 16.0,
            outline_label_padding: 4.0,
            min_width: 210.0,
            rows: 2,
        }
    }
}

/// Text field type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Filled container with an active indicator.
    Filled,
    /// Outline without a container color.
    Outlined,
}

/// What a text field style depends on besides the status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variant {
    /// Field type.
    pub kind: Kind,
    /// Whether the field shows an error.
    pub error: bool,
}

/// The appearance of a text field in one status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Container fill of a filled field.
    pub container: Color,
    /// Hover state layer over the container of a filled field.
    pub state_layer: Color,
    /// Active indicator color of a filled field.
    pub indicator: Color,
    /// Active indicator height of a filled field.
    pub indicator_height: f32,
    /// Outline color of an outlined field.
    pub outline: Color,
    /// Outline width of an outlined field.
    pub outline_width: f32,
    /// Label color.
    pub label: Color,
    /// Color of the placeholder.
    pub placeholder: Color,
    /// Leading icon color.
    pub leading_icon: Color,
    /// Trailing icon color.
    pub trailing_icon: Color,
    /// Supporting text and counter color.
    pub supporting_text: Color,
    /// Prefix and suffix color.
    pub affix: Color,
}

/// The appearance catalog of text fields.
pub trait Catalog {
    /// Style class.
    type Class<'a>;

    /// The default class.
    fn default<'a>() -> Self::Class<'a>;

    /// The style of a class in a status. Only `Active`, `Hovered`, `Focused`
    /// and `Disabled` are passed.
    fn style(&self, class: &Self::Class<'_>, status: Status, variant: Variant) -> Style;
}

/// A style function.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, Variant) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, variant: Variant) -> Style {
        class(self, status, variant)
    }
}

/// The baseline text field style.
pub fn style(theme: &Theme, status: Status, variant: Variant) -> Style {
    let c = &theme.colors;
    let Variant { kind, error } = variant;
    if status == Status::Disabled {
        let content = alpha(c.on_surface, theme.disabled.content);
        return Style {
            container: alpha(c.on_surface, 0.04),
            state_layer: c.on_surface,
            indicator: content,
            indicator_height: 1.0,
            outline: alpha(c.on_surface, theme.disabled.outline),
            outline_width: 1.0,
            label: content,
            placeholder: content,
            leading_icon: content,
            trailing_icon: content,
            supporting_text: content,
            affix: content,
        };
    }
    let hovered = status == Status::Hovered;
    let focused = status == Status::Focused;
    let accent = match (error, focused, hovered) {
        (true, false, true) => c.on_error_container,
        (true, _, _) => c.error,
        (false, true, _) => c.primary,
        (false, false, true) => c.on_surface,
        (false, false, false) => match kind {
            Kind::Filled => c.on_surface_variant,
            Kind::Outlined => c.outline,
        },
    };
    let label = match (error, focused, hovered) {
        (true, false, true) => c.on_error_container,
        (true, _, _) => c.error,
        (false, true, _) => c.primary,
        (false, false, true) if kind == Kind::Outlined => c.on_surface,
        _ => c.on_surface_variant,
    };
    let trailing_icon = match (error, focused, hovered) {
        (true, false, true) => c.on_error_container,
        (true, _, _) => c.error,
        _ => c.on_surface_variant,
    };
    Style {
        container: c.surface_container_highest,
        state_layer: c.on_surface,
        indicator: accent,
        indicator_height: if focused { 2.0 } else { 1.0 },
        outline: accent,
        outline_width: if focused { 2.0 } else { 1.0 },
        label,
        placeholder: c.on_surface_variant,
        leading_icon: c.on_surface_variant,
        trailing_icon,
        supporting_text: if error { c.error } else { c.on_surface_variant },
        affix: c.on_surface_variant,
    }
}

enum Input<'a, Message> {
    Line {
        value: String,
        on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
        on_submit: Option<Message>,
        secure: bool,
    },
    Area {
        content: &'a editor::Content<Renderer>,
        on_action: Option<Box<dyn Fn(Action) -> Message + 'a>>,
    },
    Custom {
        element: Element<'a, Message>,
        populated: bool,
        enabled: bool,
    },
}

/// A Material text field.
pub struct TextField<'a, Message> {
    input: Input<'a, Message>,
    kind: Kind,
    label: Option<String>,
    placeholder: String,
    supporting_text: Option<String>,
    error_text: Option<String>,
    error: bool,
    leading_icon: Option<widget::svg::Handle>,
    trailing_icon: Option<widget::svg::Handle>,
    on_trailing_icon: Option<Message>,
    prefix: Option<String>,
    suffix: Option<String>,
    max_length: Option<usize>,
    width: Length,
    id: Option<widget::Id>,
    active: bool,
    theme: Theme,
    class: StyleFn<'a>,
}

fn new<'a, Message>(
    theme: &Theme,
    kind: Kind,
    input: Input<'a, Message>,
) -> TextField<'a, Message> {
    TextField {
        input,
        kind,
        label: None,
        placeholder: String::new(),
        supporting_text: None,
        error_text: None,
        error: false,
        leading_icon: None,
        trailing_icon: None,
        on_trailing_icon: None,
        prefix: None,
        suffix: None,
        max_length: None,
        width: Length::Fill,
        id: None,
        active: false,
        theme: theme.clone(),
        class: Box::new(style),
    }
}

fn line<'a, Message>(value: &str) -> Input<'a, Message> {
    Input::Line {
        value: value.to_string(),
        on_input: None,
        on_submit: None,
        secure: false,
    }
}

/// A single line filled text field. Without `on_input` it is disabled.
pub fn filled<'a, Message>(theme: &Theme, value: &str) -> TextField<'a, Message> {
    new(theme, Kind::Filled, line(value))
}

/// A single line outlined text field. Without `on_input` it is disabled.
pub fn outlined<'a, Message>(theme: &Theme, value: &str) -> TextField<'a, Message> {
    new(theme, Kind::Outlined, line(value))
}

/// A multi line filled text field. Without `on_action` it is disabled.
pub fn filled_area<'a, Message>(
    theme: &Theme,
    content: &'a editor::Content<Renderer>,
) -> TextField<'a, Message> {
    new(
        theme,
        Kind::Filled,
        Input::Area {
            content,
            on_action: None,
        },
    )
}

/// A multi line outlined text field. Without `on_action` it is disabled.
pub fn outlined_area<'a, Message>(
    theme: &Theme,
    content: &'a editor::Content<Renderer>,
) -> TextField<'a, Message> {
    new(
        theme,
        Kind::Outlined,
        Input::Area {
            content,
            on_action: None,
        },
    )
}

pub(crate) fn custom<'a, Message>(
    theme: &Theme,
    kind: Kind,
    element: Element<'a, Message>,
    populated: bool,
    enabled: bool,
) -> TextField<'a, Message> {
    new(
        theme,
        kind,
        Input::Custom {
            element,
            populated,
            enabled,
        },
    )
}

impl<'a, Message> TextField<'a, Message> {
    pub(crate) fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets the message produced with the new text. Single line fields only.
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        if let Input::Line { on_input: slot, .. } = &mut self.input {
            *slot = Some(Box::new(on_input));
        }
        self
    }

    /// Sets the message produced on Enter. Single line fields only.
    pub fn on_submit(mut self, message: Message) -> Self {
        if let Input::Line { on_submit, .. } = &mut self.input {
            *on_submit = Some(message);
        }
        self
    }

    /// Hides the text behind bullets. Single line fields only.
    pub fn secure(mut self, secure: bool) -> Self {
        if let Input::Line { secure: slot, .. } = &mut self.input {
            *slot = secure;
        }
        self
    }

    /// Sets the message produced with every editor action. Multi line fields only.
    pub fn on_action(mut self, on_action: impl Fn(Action) -> Message + 'a) -> Self {
        if let Input::Area {
            on_action: slot, ..
        } = &mut self.input
        {
            *slot = Some(Box::new(on_action));
        }
        self
    }

    /// Sets the label that floats above the text.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the hint shown while the field is empty and the label floats.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the text below the field.
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// Sets the text that replaces the supporting text while the field has an error.
    pub fn error_text(mut self, text: impl Into<String>) -> Self {
        self.error_text = Some(text.into());
        self
    }

    /// Shows the error colors.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Adds an icon before the text.
    pub fn leading_icon(mut self, icon: widget::svg::Handle) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Adds an icon after the text.
    pub fn trailing_icon(mut self, icon: widget::svg::Handle) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Makes the trailing icon pressable.
    pub fn on_trailing_icon_press(mut self, message: Message) -> Self {
        self.on_trailing_icon = Some(message);
        self
    }

    /// Adds fixed text before the input.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Adds fixed text after the input.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Shows a counter. A single line field also limits its text to `max` characters.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the widget id used to focus the field.
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Replaces the style function.
    pub fn style(mut self, style: impl Fn(&Theme, Status, Variant) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<'a, Message: Clone + 'a> From<TextField<'a, Message>> for Element<'a, Message> {
    fn from(field: TextField<'a, Message>) -> Self {
        let TextField {
            input,
            kind,
            label,
            placeholder,
            supporting_text,
            error_text,
            error,
            leading_icon,
            trailing_icon,
            on_trailing_icon,
            prefix,
            suffix,
            max_length,
            width,
            id,
            active,
            theme,
            class,
        } = field;
        let metrics = theme.components.text_field;
        let mut body = theme.typography.body_large;
        body.tracking = 0.0;
        let line_height = Pixels(metrics.line_height);
        let (element, populated, count, enabled, rows): (
            Element<'a, Message>,
            bool,
            usize,
            bool,
            usize,
        ) = match input {
            Input::Line {
                value,
                on_input,
                on_submit,
                secure,
            } => {
                let enabled = on_input.is_some();
                let count = value.chars().count();
                let mut input = text_input("", &value)
                    .font(body.font)
                    .size(body.size)
                    .line_height(line_height)
                    .padding(0)
                    .width(Length::Fill)
                    .secure(secure)
                    .on_submit_maybe(on_submit);
                if let Some(id) = id {
                    input = input.id(id);
                }
                if let Some(on_input) = on_input {
                    input = input.on_input(move |text: String| {
                        on_input(match max_length {
                            Some(max) => text.chars().take(max).collect(),
                            None => text,
                        })
                    });
                }
                (input.into(), !value.is_empty(), count, enabled, 1)
            }
            Input::Area { content, on_action } => {
                let enabled = on_action.is_some();
                let rows = metrics.rows;
                let mut area = editor(content)
                    .font(body.font)
                    .size(body.size)
                    .line_height(line_height)
                    .padding(0)
                    .height(Length::Fixed(metrics.line_height * rows as f32));
                if let Some(id) = id {
                    area = area.id(id);
                }
                if let Some(on_action) = on_action {
                    area = area.on_action(on_action);
                }
                let count = content.text().trim_end_matches('\n').chars().count();
                (area.into(), !content.is_empty(), count, enabled, rows)
            }
            Input::Custom {
                element,
                populated,
                enabled,
            } => (element, populated, 0, enabled, 1),
        };
        let supporting = match (error, error_text) {
            (true, Some(text)) => Some(text),
            _ => supporting_text,
        };
        Field::new(
            &theme,
            element,
            Parts {
                kind,
                label,
                placeholder,
                supporting,
                counter: max_length.map(|max| (count, max)),
                error,
                enabled,
                populated,
                active,
                rows,
                leading_icon,
                trailing_icon,
                prefix,
                suffix,
                width,
                body,
            },
            on_trailing_icon,
            class,
        )
        .into()
    }
}
