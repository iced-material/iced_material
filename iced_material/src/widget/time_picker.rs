// SPDX-License-Identifier: LGPL-3.0-only

//! Time pickers with a clock dial and a text input, for 12 and 24 hour clocks.
//!
//! The state lives in the application: keep a [`State`], update it with every
//! [`Event`], and build the picker from it in `view`.

use std::rc::Rc;

use iced::border::Border;
use iced::widget::{Space, column, container, row, text_input};
use iced::{Alignment, Background, Color, Length, Padding, Size};

use crate::Element;
use crate::shape::{Corner, Shape};
use crate::theme::Theme;
use crate::widget::dial::{Dial, Value};
use crate::widget::layer::{Layered, Mode, Placement};
use crate::widget::panel::{Panel, Surface, text};
use crate::widget::tile::{Colors, Tile};
use crate::widget::{button, icon_button};

/// Time picker dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Padding around the content.
    pub padding: f32,
    /// Size of the hour and minute selectors.
    pub selector: Size,
    /// Width of the separator between the selectors.
    pub separator: f32,
    /// Size of the period selector.
    pub period: Size,
    /// Space between the minute selector and the period selector.
    pub period_gap: f32,
    /// Size of the text fields in text input mode.
    pub field: Size,
    /// Space between the headline and the selectors, and between the selectors and the dial.
    pub section_gap: f32,
    /// Space between the dial and the actions.
    pub dial_actions_gap: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            padding: 24.0,
            selector: Size::new(96.0, 80.0),
            separator: 24.0,
            period: Size::new(52.0, 80.0),
            period_gap: 12.0,
            field: Size::new(96.0, 72.0),
            section_gap: 36.0,
            dial_actions_gap: 24.0,
        }
    }
}

/// Which part of the time the dial edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Select {
    /// The hour.
    Hour,
    /// The minute.
    Minute,
}

/// The state of a time picker.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    hour: u8,
    minute: u8,
    select: Select,
    input: bool,
    twenty_four: bool,
    hour_text: String,
    minute_text: String,
}

/// A change of the state of a time picker.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The hour changed, on a 24 hour scale.
    Hour(u8),
    /// The hour was chosen and the dial moves on to the minutes.
    HourDone(u8),
    /// The minute changed.
    Minute(u8),
    /// The dial edits another part of the time.
    Select(Select),
    /// The period changed: `true` for the afternoon.
    Period(bool),
    /// The text fields open or close.
    ToggleInput,
    /// The text of the hour field changed.
    HourText(String),
    /// The text of the minute field changed.
    MinuteText(String),
}

impl State {
    /// A picker at `hour` (0 to 23) and `minute` (0 to 59).
    pub fn new(hour: u8, minute: u8, twenty_four: bool) -> State {
        let mut state = State {
            hour: hour.min(23),
            minute: minute.min(59),
            select: Select::Hour,
            input: false,
            twenty_four,
            hour_text: String::new(),
            minute_text: String::new(),
        };
        state.sync();
        state
    }

    /// The hour on a 24 hour scale.
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// The minute.
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Which part the dial edits.
    pub fn select(&self) -> Select {
        self.select
    }

    /// Whether the text fields are shown instead of the dial.
    pub fn is_input(&self) -> bool {
        self.input
    }

    fn pm(&self) -> bool {
        self.hour >= 12
    }

    fn shown_hour(&self) -> u8 {
        if self.twenty_four {
            self.hour
        } else if self.hour.is_multiple_of(12) {
            12
        } else {
            self.hour % 12
        }
    }

    fn sync(&mut self) {
        self.hour_text = format!("{:02}", self.shown_hour());
        self.minute_text = format!("{:02}", self.minute);
    }

    fn to_hour24(&self, shown: u8) -> u8 {
        if self.twenty_four {
            shown
        } else {
            shown % 12 + if self.pm() { 12 } else { 0 }
        }
    }

    /// Applies an event.
    pub fn update(&mut self, event: Event) {
        match event {
            Event::Hour(hour) => {
                self.hour = hour.min(23);
                self.sync();
            }
            Event::HourDone(hour) => {
                self.hour = hour.min(23);
                self.select = Select::Minute;
                self.sync();
            }
            Event::Minute(minute) => {
                self.minute = minute.min(59);
                self.sync();
            }
            Event::Select(select) => self.select = select,
            Event::Period(pm) => {
                self.hour = self.hour % 12 + if pm { 12 } else { 0 };
                self.sync();
            }
            Event::ToggleInput => {
                self.input = !self.input;
                self.sync();
            }
            Event::HourText(text) => {
                let digits = typed_digits(&text);
                if let Ok(value) = digits.parse::<u8>() {
                    let valid = if self.twenty_four {
                        value <= 23
                    } else {
                        (1..=12).contains(&value)
                    };
                    if valid {
                        self.hour = self.to_hour24(value);
                    }
                }
                self.hour_text = digits;
            }
            Event::MinuteText(text) => {
                let digits = typed_digits(&text);
                if let Ok(value) = digits.parse::<u8>()
                    && value <= 59
                {
                    self.minute = value;
                }
                self.minute_text = digits;
            }
        }
    }
}

/// Texts of the time picker. The defaults are English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labels {
    /// Headline of the dial view.
    pub select_time: &'static str,
    /// Headline of the text view.
    pub enter_time: &'static str,
    /// Caption of the hour field.
    pub hour: &'static str,
    /// Caption of the minute field.
    pub minute: &'static str,
    /// Morning.
    pub am: &'static str,
    /// Afternoon.
    pub pm: &'static str,
    /// Cancel button.
    pub cancel: &'static str,
    /// Confirm button.
    pub ok: &'static str,
}

impl Default for Labels {
    fn default() -> Self {
        Labels {
            select_time: "Select time",
            enter_time: "Enter time",
            hour: "Hour",
            minute: "Minute",
            am: "AM",
            pm: "PM",
            cancel: "Cancel",
            ok: "OK",
        }
    }
}

/// A time picker.
pub struct TimePicker<'a, Message> {
    state: &'a State,
    labels: Labels,
    on_event: Rc<dyn Fn(Event) -> Message + 'a>,
    on_cancel: Option<Message>,
    on_confirm: Option<Message>,
}

/// A time picker.
pub fn time_picker<'a, Message>(
    state: &'a State,
    on_event: impl Fn(Event) -> Message + 'a,
) -> TimePicker<'a, Message> {
    TimePicker {
        state,
        labels: Labels::default(),
        on_event: Rc::new(on_event),
        on_cancel: None,
        on_confirm: None,
    }
}

impl<'a, Message> TimePicker<'a, Message> {
    /// Sets the texts.
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Sets the message produced by the cancel button.
    pub fn on_cancel(mut self, message: Message) -> Self {
        self.on_cancel = Some(message);
        self
    }

    /// Sets the message produced by the confirm button.
    pub fn on_confirm(mut self, message: Message) -> Self {
        self.on_confirm = Some(message);
        self
    }
}

fn typed_digits(text: &str) -> String {
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    if digits.len() > 2 {
        digits.chars().last().map(String::from).unwrap_or_default()
    } else {
        digits
    }
}

fn period_shape(theme: &Theme, top: bool) -> Shape {
    let corner = theme.shape.small.top_left;
    let square = Corner::Fixed(0.0);
    if top {
        Shape {
            top_left: corner,
            top_right: corner,
            bottom_right: square,
            bottom_left: square,
        }
    } else {
        Shape {
            top_left: square,
            top_right: square,
            bottom_right: corner,
            bottom_left: corner,
        }
    }
}

impl<'a, Message: Clone + 'a> TimePicker<'a, Message> {
    fn field(
        &self,
        theme: &Theme,
        value: &str,
        caption: &'static str,
        on_input: impl Fn(String) -> Message + 'a,
    ) -> Element<'a, Message> {
        let m = theme.components.time_picker;
        let c = &theme.colors;
        let style = theme.typography.display_medium;
        let radius = theme.shape.small.radius(m.field);
        let input = text_input("", value)
            .font(style.font)
            .size(style.size)
            .line_height(iced::widget::text::LineHeight::Absolute(
                style.line_height.into(),
            ))
            .padding(0)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fill)
            .on_input(on_input)
            .style(move |theme: &Theme, status| {
                let c = &theme.colors;
                let focused = matches!(status, text_input::Status::Focused { .. });
                text_input::Style {
                    background: Background::Color(if focused {
                        c.primary_container
                    } else {
                        c.surface_container_highest
                    }),
                    border: Border {
                        color: if focused {
                            c.primary
                        } else {
                            Color::TRANSPARENT
                        },
                        width: if focused { 2.0 } else { 0.0 },
                        radius,
                    },
                    icon: c.on_surface_variant,
                    placeholder: c.on_surface_variant,
                    value: if focused {
                        c.on_primary_container
                    } else {
                        c.on_surface
                    },
                    selection: crate::state::alpha(c.primary, 0.4),
                }
            });
        column![
            container(input)
                .width(Length::Fixed(m.field.width))
                .height(Length::Fixed(m.field.height))
                .center_y(Length::Fixed(m.field.height)),
            text(caption, theme.typography.body_small, c.on_surface_variant),
        ]
        .spacing(7)
        .into()
    }

    /// Builds the picker.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        let m = theme.components.time_picker;
        let c = &theme.colors;
        let t = &theme.typography;
        let state = self.state;
        let send = {
            let on_event = self.on_event.clone();
            move |event: Event| on_event(event)
        };
        let title = if state.input {
            self.labels.enter_time
        } else {
            self.labels.select_time
        };
        let mut body = column![text(title, t.label_medium, c.on_surface_variant)];
        body = body.push(Space::new().height(Length::Fixed(m.section_gap)));

        let mut display = row![].align_y(Alignment::Start);
        if state.input {
            let (hour, minute) = (send.clone(), send.clone());
            display = display
                .push(
                    self.field(theme, &state.hour_text, self.labels.hour, move |s| {
                        hour(Event::HourText(s))
                    }),
                )
                .push(
                    container(text(":", t.display_large, c.on_surface))
                        .width(Length::Fixed(m.separator))
                        .height(Length::Fixed(m.field.height))
                        .center(Length::Fixed(m.separator)),
                )
                .push(
                    self.field(theme, &state.minute_text, self.labels.minute, move |s| {
                        minute(Event::MinuteText(s))
                    }),
                );
        } else {
            let selector = |label: String, selected: bool, event: Event| -> Element<'a, Message> {
                Tile::new(
                    theme,
                    label,
                    t.display_large,
                    Colors {
                        container: if selected {
                            c.primary_container
                        } else {
                            c.surface_container_highest
                        },
                        label: if selected {
                            c.on_primary_container
                        } else {
                            c.on_surface
                        },
                        layer: if selected {
                            c.on_primary_container
                        } else {
                            c.on_surface
                        },
                        outline: Color::TRANSPARENT,
                        outline_width: 0.0,
                    },
                    theme.shape.small,
                    m.selector,
                    send(event),
                )
                .into()
            };
            display = display
                .push(selector(
                    format!("{:02}", state.shown_hour()),
                    state.select == Select::Hour,
                    Event::Select(Select::Hour),
                ))
                .push(
                    container(text(":", t.display_large, c.on_surface))
                        .width(Length::Fixed(m.separator))
                        .height(Length::Fixed(m.selector.height))
                        .center(Length::Fixed(m.separator)),
                )
                .push(selector(
                    format!("{:02}", state.minute),
                    state.select == Select::Minute,
                    Event::Select(Select::Minute),
                ));
        }
        if !state.twenty_four {
            let half = |label: &'static str, pm: bool, top: bool| -> Element<'a, Message> {
                let selected = state.pm() == pm;
                Tile::new(
                    theme,
                    label,
                    t.title_medium,
                    Colors {
                        container: if selected {
                            c.tertiary_container
                        } else {
                            Color::TRANSPARENT
                        },
                        label: if selected {
                            c.on_tertiary_container
                        } else {
                            c.on_surface_variant
                        },
                        layer: if selected {
                            c.on_tertiary_container
                        } else {
                            c.on_surface_variant
                        },
                        outline: c.outline,
                        outline_width: 1.0,
                    },
                    period_shape(theme, top),
                    Size::new(
                        m.period.width,
                        m.period.height / 2.0 + if top { 0.0 } else { 1.0 },
                    ),
                    send(Event::Period(pm)),
                )
                .into()
            };
            display = display
                .push(Space::new().width(Length::Fixed(m.period_gap)))
                .push(
                    column![
                        half(self.labels.am, false, true),
                        half(self.labels.pm, true, false)
                    ]
                    .spacing(-1.0),
                );
        }
        let boxes = if state.input {
            m.field.width * 2.0
        } else {
            m.selector.width * 2.0
        };
        let period = if state.twenty_four {
            0.0
        } else {
            m.period_gap + m.period.width
        };
        let display_width = boxes + m.separator + period;
        let content_width = display_width.max(256.0);
        body = body.push(container(display).center_x(Length::Fill));
        body = body.push(Space::new().height(Length::Fixed(m.section_gap)));

        if !state.input {
            let hours = state.select == Select::Hour;
            let value = if hours {
                if state.twenty_four {
                    Value::Hour24(state.hour)
                } else {
                    Value::Hour12(state.shown_hour())
                }
            } else {
                Value::Minute(state.minute)
            };
            let send = send.clone();
            let pm = state.pm();
            let twenty_four = state.twenty_four;
            let dial = Dial::new(theme, value, move |value, done| {
                let to24 = |shown: u8| {
                    if twenty_four {
                        shown
                    } else {
                        shown % 12 + if pm { 12 } else { 0 }
                    }
                };
                match value {
                    Value::Hour12(h) | Value::Hour24(h) if done => send(Event::HourDone(to24(h))),
                    Value::Hour12(h) | Value::Hour24(h) => send(Event::Hour(to24(h))),
                    Value::Minute(minute) => send(Event::Minute(minute)),
                }
            });
            body = body.push(container(dial).center_x(Length::Fill));
            body = body.push(Space::new().height(Length::Fixed(m.dial_actions_gap)));
        }

        let toggle = {
            let send = send.clone();
            icon_button::standard(
                theme,
                if state.input {
                    crate::icon::symbol::schedule(false)
                } else {
                    crate::icon::symbol::keyboard(false)
                },
            )
            .on_press(send(Event::ToggleInput))
        };
        body = body.push(
            row![
                toggle,
                Space::new().width(Length::Fill),
                button::text(theme, self.labels.cancel).on_press_maybe(self.on_cancel.clone()),
                button::text(theme, self.labels.ok).on_press_maybe(self.on_confirm.clone()),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );

        Panel::new(
            body.width(Length::Fixed(content_width + m.padding * 2.0))
                .padding(m.padding)
                .into(),
            Surface {
                color: c.surface_container_high,
                text: c.on_surface,
                shape: theme.shape.extra_large,
                elevation: 3.0,
                padding: Padding::ZERO,
                min_width: 0.0,
                max_width: f32::INFINITY,
            },
        )
        .into()
    }

    /// Builds the picker as a modal dialog over `base` while `open` is true.
    pub fn over(
        self,
        theme: &Theme,
        base: impl Into<Element<'a, Message>>,
        open: bool,
        on_dismiss: Option<Message>,
    ) -> Element<'a, Message> {
        Layered::new(
            theme,
            base.into(),
            self.build(theme),
            Mode::Modal {
                open,
                on_dismiss,
                dismiss_on_scrim: false,
            },
            Placement::Center,
        )
        .into()
    }

    /// Builds the picker as a popup below `anchor` while `open` is true.
    pub fn below(
        self,
        theme: &Theme,
        anchor: impl Into<Element<'a, Message>>,
        open: bool,
        on_dismiss: Option<Message>,
    ) -> Element<'a, Message> {
        Layered::new(
            theme,
            anchor.into(),
            self.build(theme),
            Mode::Popup { open, on_dismiss },
            Placement::BelowStart(4.0),
        )
        .into()
    }
}
