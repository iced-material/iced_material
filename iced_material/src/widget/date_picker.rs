// SPDX-License-Identifier: LGPL-3.0-only

//! Docked and modal date pickers with a calendar, a year list and text input.
//!
//! The state lives in the application: keep a [`State`], update it with every
//! [`Event`], and build the picker from it in `view`. The current date is
//! passed in, because the library does not read the clock.

use std::ops::RangeInclusive;
use std::rc::Rc;

use iced::widget::{Space, column, container, row};
use iced::{Alignment, Length, Padding};

use crate::Element;
use crate::calendar::{Date, Names, days_in_month};
use crate::icon::symbol;
use crate::theme::Theme;
use crate::widget::day_grid::{COLUMN, DayGrid};
use crate::widget::layer::{Layered, Mode, Placement};
use crate::widget::panel::{Panel, Surface, text};
use crate::widget::year_grid::YearGrid;
use crate::widget::{button, divider, icon_button, text_field};

/// Date picker dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Width of a modal picker with the calendar.
    pub modal_width: f32,
    /// Width of a modal picker with text input.
    pub input_width: f32,
    /// Width of a docked picker.
    pub docked_width: f32,
    /// Height of the header of a modal picker.
    pub header_height: f32,
    /// Height of the month and year row of a modal picker.
    pub modal_month_height: f32,
    /// Height of the month and year row of a docked picker.
    pub docked_month_height: f32,
    /// Height of the weekday row, of a calendar row and of a year grid.
    pub body_height: f32,
    /// Height of the actions row of a modal picker.
    pub modal_actions_height: f32,
    /// Height of the actions row of a docked picker.
    pub docked_actions_height: f32,
    /// Diameter of the selected day and the today outline in a modal picker.
    pub modal_day: f32,
    /// Diameter of the selected day and the today outline in a docked picker.
    pub docked_day: f32,
    /// Space before and after the month and year row, the grid and the year list.
    pub side_padding: f32,
    /// Distance between the picker and its anchor.
    pub gap: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            modal_width: 360.0,
            input_width: 328.0,
            docked_width: 360.0,
            header_height: 120.0,
            modal_month_height: 56.0,
            docked_month_height: 64.0,
            body_height: 336.0,
            modal_actions_height: 55.0,
            docked_actions_height: 56.0,
            modal_day: 40.0,
            docked_day: 48.0,
            side_padding: 12.0,
            gap: 4.0,
        }
    }
}

/// Which view the picker shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// The month grid.
    Calendar,
    /// The list of years.
    Years,
    /// The text field.
    Input,
}

/// Order of the parts of a typed date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 08/17/2025.
    MonthDayYear,
    /// 17/08/2025.
    DayMonthYear,
    /// 2025/08/17.
    YearMonthDay,
}

impl Format {
    /// The hint shown in the text field.
    pub fn hint(self) -> &'static str {
        match self {
            Format::MonthDayYear => "mm/dd/yyyy",
            Format::DayMonthYear => "dd/mm/yyyy",
            Format::YearMonthDay => "yyyy/mm/dd",
        }
    }

    /// Writes a date in this order with `/` between the parts.
    pub fn format(self, date: Date) -> String {
        match self {
            Format::MonthDayYear => format!("{:02}/{:02}/{:04}", date.month, date.day, date.year),
            Format::DayMonthYear => format!("{:02}/{:02}/{:04}", date.day, date.month, date.year),
            Format::YearMonthDay => format!("{:04}/{:02}/{:02}", date.year, date.month, date.day),
        }
    }

    /// Reads three numbers separated by `/`, `.` or `-` in this order.
    pub fn parse(self, text: &str) -> Option<Date> {
        let parts: Vec<&str> = text.trim().split(['/', '.', '-']).collect();
        let [a, b, c] = parts.as_slice() else {
            return None;
        };
        let number = |s: &str| s.trim().parse::<i32>().ok();
        let (a, b, c) = (number(a)?, number(b)?, number(c)?);
        let (year, month, day) = match self {
            Format::MonthDayYear => (c, a, b),
            Format::DayMonthYear => (c, b, a),
            Format::YearMonthDay => (a, b, c),
        };
        Date::new(year, u8::try_from(month).ok()?, u8::try_from(day).ok()?)
    }
}

/// The state of a date picker.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    selected: Option<Date>,
    shown: (i32, u8),
    mode: ViewMode,
    text: String,
    years: RangeInclusive<i32>,
    format: Format,
}

/// A change of the state of a date picker.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A date was chosen.
    Pick(Date),
    /// The shown month moves by this many months.
    Month(i32),
    /// The year list opens or closes.
    ToggleYears,
    /// A year was chosen.
    Year(i32),
    /// The text field opens or closes.
    ToggleInput,
    /// The text in the text field changed.
    Text(String),
}

impl State {
    /// A picker showing the month of `selected`, or of `today` without a selection.
    pub fn new(selected: Option<Date>, today: Date) -> State {
        let shown = selected.unwrap_or(today);
        let format = Format::MonthDayYear;
        State {
            selected,
            shown: (shown.year, shown.month),
            mode: ViewMode::Calendar,
            text: selected.map(|d| format.format(d)).unwrap_or_default(),
            years: 1900..=2100,
            format,
        }
    }

    /// Limits the years that can be chosen.
    pub fn years(mut self, years: RangeInclusive<i32>) -> State {
        self.years = years;
        self
    }

    /// Sets the order of the typed date.
    pub fn format(mut self, format: Format) -> State {
        self.format = format;
        self.text = self.selected.map(|d| format.format(d)).unwrap_or_default();
        self
    }

    /// The chosen date.
    pub fn selected(&self) -> Option<Date> {
        self.selected
    }

    /// The view that is shown.
    pub fn mode(&self) -> ViewMode {
        self.mode
    }

    /// The month shown by the calendar as year and month.
    pub fn shown(&self) -> (i32, u8) {
        self.shown
    }

    fn range(&self) -> (Date, Date) {
        (
            Date {
                year: *self.years.start(),
                month: 1,
                day: 1,
            },
            Date {
                year: *self.years.end(),
                month: 12,
                day: 31,
            },
        )
    }

    /// Applies an event.
    pub fn update(&mut self, event: Event) {
        match event {
            Event::Pick(date) => {
                self.selected = Some(date);
                self.shown = (date.year, date.month);
                self.text = self.format.format(date);
            }
            Event::Month(delta) => {
                let first = Date {
                    year: self.shown.0,
                    month: self.shown.1,
                    day: 1,
                }
                .add_months(delta);
                if self.years.contains(&first.year) {
                    self.shown = (first.year, first.month);
                }
            }
            Event::ToggleYears => {
                self.mode = if self.mode == ViewMode::Years {
                    ViewMode::Calendar
                } else {
                    ViewMode::Years
                };
            }
            Event::Year(year) => {
                self.shown.0 = year;
                self.mode = ViewMode::Calendar;
            }
            Event::ToggleInput => {
                if self.mode == ViewMode::Input {
                    self.mode = ViewMode::Calendar;
                } else {
                    self.text = self
                        .selected
                        .map(|d| self.format.format(d))
                        .unwrap_or_default();
                    self.mode = ViewMode::Input;
                }
            }
            Event::Text(text) => {
                let (min, max) = self.range();
                self.selected = self.format.parse(&text).filter(|d| *d >= min && *d <= max);
                if let Some(date) = self.selected {
                    self.shown = (date.year, date.month);
                }
                self.text = text;
            }
        }
    }
}

/// Texts of the date picker. The defaults are English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labels {
    /// Title of the calendar header.
    pub select_date: &'static str,
    /// Title of the text input header.
    pub enter_date: &'static str,
    /// Headline while no date is chosen.
    pub no_date: &'static str,
    /// Label of the text field.
    pub date: &'static str,
    /// Error text of the text field.
    pub invalid: &'static str,
    /// Cancel button.
    pub cancel: &'static str,
    /// Confirm button.
    pub ok: &'static str,
}

impl Default for Labels {
    fn default() -> Self {
        Labels {
            select_date: "Select date",
            enter_date: "Enter date",
            no_date: "Date",
            date: "Date",
            invalid: "Invalid date",
            cancel: "Cancel",
            ok: "OK",
        }
    }
}

/// A date picker.
pub struct DatePicker<'a, Message> {
    state: &'a State,
    today: Date,
    names: Names,
    labels: Labels,
    on_event: Rc<dyn Fn(Event) -> Message + 'a>,
    on_cancel: Option<Message>,
    on_confirm: Option<Message>,
    docked: bool,
}

fn new<'a, Message>(
    state: &'a State,
    today: Date,
    on_event: impl Fn(Event) -> Message + 'a,
    docked: bool,
) -> DatePicker<'a, Message> {
    DatePicker {
        state,
        today,
        names: Names::default(),
        labels: Labels::default(),
        on_event: Rc::new(on_event),
        on_cancel: None,
        on_confirm: None,
        docked,
    }
}

/// A modal date picker with a header.
pub fn modal<'a, Message>(
    state: &'a State,
    today: Date,
    on_event: impl Fn(Event) -> Message + 'a,
) -> DatePicker<'a, Message> {
    new(state, today, on_event, false)
}

/// A docked date picker without a header, for use below a text field.
pub fn docked<'a, Message>(
    state: &'a State,
    today: Date,
    on_event: impl Fn(Event) -> Message + 'a,
) -> DatePicker<'a, Message> {
    new(state, today, on_event, true)
}

impl<'a, Message> DatePicker<'a, Message> {
    /// Sets the month, weekday names and the first day of the week.
    pub fn names(mut self, names: Names) -> Self {
        self.names = names;
        self
    }

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

    /// Sets the message produced by the confirm button, which is disabled without a date.
    pub fn on_confirm(mut self, message: Message) -> Self {
        self.on_confirm = Some(message);
        self
    }
}

impl<'a, Message: Clone + 'a> DatePicker<'a, Message> {
    /// Builds the picker.
    pub fn build(self, theme: &Theme) -> Element<'a, Message> {
        let m = theme.components.date_picker;
        let c = &theme.colors;
        let t = &theme.typography;
        let state = self.state;
        let send = {
            let on_event = self.on_event.clone();
            move |event: Event| on_event(event)
        };
        let input = state.mode == ViewMode::Input;
        let width = if self.docked {
            m.docked_width
        } else if input {
            m.input_width
        } else {
            m.modal_width
        };
        let mut body = column![].width(Length::Fixed(width));

        if !self.docked {
            let title = if input {
                self.labels.enter_date
            } else {
                self.labels.select_date
            };
            let headline = state
                .selected
                .map(|d| {
                    let weekday = self.names.weekdays_short[usize::from(d.weekday())];
                    let month = self.names.months_short[usize::from(d.month) - 1];
                    format!("{weekday}, {month} {}", d.day)
                })
                .unwrap_or_else(|| self.labels.no_date.to_string());
            let toggle = {
                let send = send.clone();
                icon_button::standard(
                    theme,
                    if input {
                        symbol::calendar_today(false)
                    } else {
                        symbol::edit(false)
                    },
                )
                .on_press(send(Event::ToggleInput))
            };
            body = body.push(
                container(
                    column![
                        text(title, t.label_large, c.on_surface_variant),
                        Space::new().height(Length::Fill),
                        row![
                            text(headline, t.headline_large, c.on_surface_variant),
                            Space::new().width(Length::Fill),
                            toggle
                        ]
                        .align_y(Alignment::Center),
                    ]
                    .height(Length::Fill),
                )
                .height(Length::Fixed(m.header_height))
                .padding(Padding {
                    top: 16.0,
                    right: 12.0,
                    bottom: 12.0,
                    left: 24.0,
                }),
            );
            body = body.push(divider::horizontal(theme).color(c.outline));
        }

        match state.mode {
            ViewMode::Input => {
                let format = state.format;
                let invalid = !state.text.is_empty() && state.selected.is_none();
                let send = send.clone();
                let field = text_field::outlined(theme, &state.text)
                    .label(self.labels.date)
                    .placeholder(format.hint())
                    .supporting_text(format.hint())
                    .error_text(self.labels.invalid)
                    .error(invalid)
                    .on_input(move |t| send(Event::Text(t)));
                body = body.push(container(field).padding(Padding {
                    top: 24.0,
                    right: 24.0,
                    bottom: 24.0,
                    left: 24.0,
                }));
            }
            ViewMode::Calendar | ViewMode::Years => {
                let years = state.mode == ViewMode::Years;
                let height = if self.docked {
                    m.docked_month_height
                } else {
                    m.modal_month_height
                };
                let label = format!(
                    "{} {}",
                    self.names.months[usize::from(state.shown.1) - 1],
                    state.shown.0
                );
                let menu = {
                    let send = send.clone();
                    button::text(theme, label)
                        .trailing_icon(if years {
                            symbol::arrow_drop_up(false)
                        } else {
                            symbol::arrow_drop_down(false)
                        })
                        .on_press(send(Event::ToggleYears))
                };
                let mut header =
                    row![menu, Space::new().width(Length::Fill)].align_y(Alignment::Center);
                if !years {
                    let (min, max) = state.range();
                    let first = Date {
                        year: state.shown.0,
                        month: state.shown.1,
                        day: 1,
                    };
                    let last = Date {
                        day: days_in_month(state.shown.0, state.shown.1),
                        ..first
                    };
                    let (previous, next) = (send.clone(), send.clone());
                    header = header
                        .push(
                            icon_button::standard(theme, symbol::chevron_left(false))
                                .on_press_maybe((first > min).then(|| previous(Event::Month(-1)))),
                        )
                        .push(
                            icon_button::standard(theme, symbol::chevron_right(false))
                                .on_press_maybe((last < max).then(|| next(Event::Month(1)))),
                        );
                }
                body = body.push(
                    container(header)
                        .height(Length::Fixed(height))
                        .center_y(Length::Fixed(height))
                        .padding(Padding::from([0.0, m.side_padding])),
                );
                let content: Element<'a, Message> = if years {
                    let send = send.clone();
                    YearGrid::new(
                        theme,
                        state.years.clone(),
                        state.shown.0,
                        self.today.year,
                        m.body_height,
                        move |year| send(Event::Year(year)),
                    )
                    .into()
                } else {
                    let (pick, month) = (send.clone(), send.clone());
                    DayGrid::new(
                        theme,
                        state.shown,
                        state.selected,
                        self.today,
                        state.range(),
                        self.names.clone(),
                        if self.docked {
                            m.docked_day
                        } else {
                            m.modal_day
                        },
                        move |date| pick(Event::Pick(date)),
                        move |delta| month(Event::Month(delta)),
                    )
                    .into()
                };
                body = body.push(
                    container(content)
                        .width(Length::Fixed(width))
                        .height(Length::Fixed(m.body_height))
                        .padding(Padding::from([0.0, (width - COLUMN * 7.0) / 2.0])),
                );
            }
        }

        if self.on_cancel.is_some() || self.on_confirm.is_some() || !self.docked {
            let actions_height = if self.docked {
                m.docked_actions_height
            } else {
                m.modal_actions_height
            };
            let mut actions = row![Space::new().width(Length::Fill)]
                .spacing(8)
                .align_y(Alignment::Center);
            actions = actions.push(
                button::text(theme, self.labels.cancel).on_press_maybe(self.on_cancel.clone()),
            );
            actions = actions.push(
                button::text(theme, self.labels.ok)
                    .on_press_maybe(state.selected.and(self.on_confirm.clone())),
            );
            body = body.push(
                container(actions)
                    .height(Length::Fixed(actions_height))
                    .center_y(Length::Fixed(actions_height))
                    .padding(Padding::from([0.0, 8.0])),
            );
        }

        Panel::new(
            body.into(),
            Surface {
                color: c.surface_container_high,
                text: c.on_surface,
                shape: if self.docked {
                    theme.shape.large
                } else {
                    theme.shape.extra_large
                },
                elevation: 3.0,
                padding: Padding::ZERO,
                min_width: width,
                max_width: width,
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
        let gap = theme.components.date_picker.gap;
        Layered::new(
            theme,
            anchor.into(),
            self.build(theme),
            Mode::Popup { open, on_dismiss },
            Placement::BelowStart(gap),
        )
        .into()
    }
}
