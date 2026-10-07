// SPDX-License-Identifier: LGPL-3.0-only

use iced::widget::{column, container, row, space, text, text_editor};
use iced::{Alignment, Element, Length};
use material_iced::Theme;
use material_iced::calendar::Date;
use material_iced::icon::{icon, symbol};
use material_iced::layout::{adaptive, feed, list_detail, margins, scaffold};
use material_iced::widget::badge::{self, badged};
use material_iced::widget::checkbox::checkbox;
use material_iced::widget::chip_set::chip_set;
use material_iced::widget::dialog::{dialog, modal};
use material_iced::widget::list::{Leading, Trailing, list_item};
use material_iced::widget::menu::{self, item, menu as menu_widget};
use material_iced::widget::navigation::{
    Badge, Destination, Labels, navigation_bar, navigation_rail,
};
use material_iced::widget::navigation_drawer::{
    DrawerItem, modal_drawer, navigation_drawer, section,
};
use material_iced::widget::progress::{circular, linear};
use material_iced::widget::radio::radio_group;
use material_iced::widget::search::search;
use material_iced::widget::segmented_button::{Segment, segmented_button};
use material_iced::widget::side_sheet::side_sheet;
use material_iced::widget::slider::{range_slider, slider};
use material_iced::widget::snackbar::snackbar;
use material_iced::widget::switch::switch;
use material_iced::widget::tabs::{self, Tab};
use material_iced::widget::top_app_bar::{self, action_button, menu_button};
use material_iced::widget::transition::{
    Axis, container_transform, fade, fade_through, shared_axis,
};
use material_iced::widget::{button, card, chip, divider, fab, icon_button};
use material_iced::widget::{date_picker, time_picker};
use material_iced::widget::{select, text_field, tooltip};

use crate::Message;
use crate::controls::styled;

#[derive(Debug, Clone)]
pub enum Event {
    Press,
    Toggle(usize),
    Single(usize),
    Multi(usize),
    Filter(usize),
    Remove(usize),
    Check(usize, bool),
    Switch(usize, bool),
    Radio(usize),
    Slider(f32),
    Range(f32, f32),
    Text(usize, String),
    Area(text_editor::Action),
    ToggleSecret,
    Menu(bool),
    SelectOpen(bool),
    SelectChoice(usize),
    Dialog(bool),
    Snackbar(bool),
    Progress(f32),
    List(usize),
    Destination(usize),
    Drawer(bool),
    Sheet(bool),
    Tab(usize),
    Search(String),
    Date(date_picker::Event),
    Time(time_picker::Event),
    DateDialog(bool),
    TimeDialog(bool),
    DateDocked(bool),
    Pm(bool),
    Preview(usize),
    Mail(Option<usize>),
    Swatch(usize, bool),
    SwatchDone,
    Expand(bool),
    FadeToggle(bool),
}

pub struct Demo {
    toggles: [bool; 4],
    single: usize,
    multi: [bool; 3],
    filters: [bool; 4],
    inputs: Vec<String>,
    checks: [bool; 3],
    switches: [bool; 3],
    radio: usize,
    slider: f32,
    range: (f32, f32),
    texts: [String; 6],
    secret: bool,
    area: text_editor::Content,
    menu: bool,
    select_open: bool,
    select: Option<usize>,
    dialog: bool,
    snackbar: bool,
    progress: f32,
    list: usize,
    destination: usize,
    drawer: bool,
    sheet: bool,
    tab: usize,
    search: String,
    date: date_picker::State,
    time: time_picker::State,
    date_dialog: bool,
    time_dialog: bool,
    date_docked: bool,
    twenty_four: bool,
    preview: usize,
    mail: Option<usize>,
    swatch: usize,
    swatch_previous: Option<usize>,
    pattern: usize,
    forward: bool,
    expanded: bool,
    fade: bool,
}

impl Default for Demo {
    fn default() -> Self {
        Demo {
            toggles: [true, false, true, false],
            single: 0,
            multi: [true, false, true],
            filters: [true, false, false, true],
            inputs: ["Rust", "Iced", "Material", "Wayland"]
                .map(String::from)
                .to_vec(),
            checks: [true, false, true],
            switches: [true, false, true],
            radio: 0,
            slider: 40.0,
            range: (20.0, 70.0),
            texts: [
                String::new(),
                "Filled".to_string(),
                String::new(),
                "name@example.com".to_string(),
                "hunter2".to_string(),
                "12".to_string(),
            ],
            secret: true,
            menu: false,
            select_open: false,
            select: Some(1),
            dialog: false,
            snackbar: true,
            progress: 0.6,
            list: 1,
            destination: 0,
            drawer: false,
            sheet: false,
            tab: 0,
            search: String::new(),
            date: date_picker::State::new(Date::new(2025, 8, 17), Date::new(2025, 8, 12).unwrap()),
            time: time_picker::State::new(9, 41, false),
            date_dialog: false,
            time_dialog: false,
            date_docked: false,
            twenty_four: false,
            preview: 2,
            mail: None,
            swatch: 0,
            swatch_previous: None,
            pattern: 0,
            forward: true,
            expanded: false,
            fade: true,
            area: text_editor::Content::with_text("First line\nSecond line\nThird line"),
        }
    }
}

impl Demo {
    pub fn update(&mut self, event: Event) {
        match event {
            Event::Press => {}
            Event::Toggle(i) => self.toggles[i] = !self.toggles[i],
            Event::Single(i) => self.single = i,
            Event::Multi(i) => self.multi[i] = !self.multi[i],
            Event::Filter(i) => self.filters[i] = !self.filters[i],
            Event::Remove(i) => {
                self.inputs.remove(i);
            }
            Event::Check(i, value) => self.checks[i] = value,
            Event::Switch(i, value) => self.switches[i] = value,
            Event::Radio(i) => self.radio = i,
            Event::Slider(value) => self.slider = value,
            Event::Range(start, end) => self.range = (start, end),
            Event::Text(i, value) => self.texts[i] = value,
            Event::Area(action) => self.area.perform(action),
            Event::ToggleSecret => self.secret = !self.secret,
            Event::Menu(open) => self.menu = open,
            Event::SelectOpen(open) => self.select_open = open,
            Event::SelectChoice(i) => self.select = Some(i),
            Event::Dialog(open) => self.dialog = open,
            Event::Snackbar(show) => self.snackbar = show,
            Event::Progress(value) => self.progress = value,
            Event::List(i) => self.list = i,
            Event::Destination(i) => self.destination = i,
            Event::Drawer(open) => self.drawer = open,
            Event::Sheet(open) => self.sheet = open,
            Event::Tab(i) => self.tab = i,
            Event::Search(text) => self.search = text,
            Event::Date(event) => self.date.update(event),
            Event::Time(event) => self.time.update(event),
            Event::DateDialog(open) => self.date_dialog = open,
            Event::TimeDialog(open) => self.time_dialog = open,
            Event::DateDocked(open) => self.date_docked = open,
            Event::Preview(i) => self.preview = i,
            Event::Mail(i) => self.mail = i,
            Event::Swatch(pattern, forward) => {
                if self.swatch_previous.is_none() {
                    self.swatch_previous = Some(self.swatch);
                    self.swatch = (self.swatch + if forward { 1 } else { 2 }) % 3;
                    self.pattern = pattern;
                    self.forward = forward;
                }
            }
            Event::SwatchDone => self.swatch_previous = None,
            Event::Expand(open) => self.expanded = open,
            Event::FadeToggle(visible) => self.fade = visible,
            Event::Pm(twenty_four) => {
                self.twenty_four = twenty_four;
                self.time =
                    time_picker::State::new(self.time.hour(), self.time.minute(), twenty_four);
            }
        }
    }
}

fn heading<'a>(title: &'a str, theme: &Theme) -> Element<'a, Message, Theme> {
    styled(title, theme.typography.title_medium).into()
}

fn press() -> Message {
    Message::Demo(Event::Press)
}

pub fn actions<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let buttons = |enabled: bool| {
        let on = enabled.then(press);
        row![
            button::elevated(theme, "Elevated").on_press_maybe(on.clone()),
            button::filled(theme, "Filled").on_press_maybe(on.clone()),
            button::filled_tonal(theme, "Tonal").on_press_maybe(on.clone()),
            button::outlined(theme, "Outlined").on_press_maybe(on.clone()),
            button::text(theme, "Text").on_press_maybe(on.clone()),
            button::filled(theme, "Icon")
                .leading_icon(symbol::add(false))
                .on_press_maybe(on),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
    };
    let icon_buttons = |enabled: bool| {
        let mut items = row![].spacing(8).align_y(Alignment::Center);
        let constructors: [fn(&Theme, _) -> icon_button::IconButton<'a, Message>; 4] = [
            icon_button::standard,
            icon_button::filled,
            icon_button::filled_tonal,
            icon_button::outlined,
        ];
        for (i, make) in constructors.into_iter().enumerate() {
            items =
                items.push(make(theme, symbol::menu(false)).on_press_maybe(enabled.then(press)));
            items = items.push(
                make(theme, symbol::edit(false))
                    .selected_icon(symbol::edit(true))
                    .selected(demo.toggles[i])
                    .on_press_maybe(enabled.then_some(Message::Demo(Event::Toggle(i)))),
            );
        }
        items
    };
    let fabs = row![
        fab::small(theme, symbol::add(false)).on_press(press()),
        fab::fab(theme, symbol::edit(false)).on_press(press()),
        fab::large(theme, symbol::add(false)).on_press(press()),
        fab::extended(theme, "Compose")
            .icon(symbol::edit(false))
            .on_press(press()),
        fab::fab(theme, symbol::add(false))
            .color(fab::Color::Secondary)
            .on_press(press()),
        fab::fab(theme, symbol::add(false))
            .color(fab::Color::Tertiary)
            .on_press(press()),
        fab::fab(theme, symbol::add(false))
            .color(fab::Color::Surface)
            .lowered(true)
            .on_press(press()),
    ]
    .spacing(16)
    .align_y(Alignment::Center);
    let single = segmented_button(
        theme,
        ["Day", "Week", "Month", "Year"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                Segment::new(label)
                    .selected(demo.single == i)
                    .on_press(Message::Demo(Event::Single(i)))
            }),
    );
    let multi = segmented_button(
        theme,
        [
            symbol::search(false),
            symbol::schedule(false),
            symbol::calendar_today(false),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, icon)| {
            Segment::new(["Search", "Time", "Date"][i])
                .icon(icon)
                .selected(demo.multi[i])
                .on_press(Message::Demo(Event::Multi(i)))
        }),
    );
    let disabled = segmented_button(
        theme,
        ["One", "Two"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| Segment::new(label).selected(i == 0)),
    );
    column![
        heading("Common buttons", theme),
        buttons(true),
        buttons(false),
        heading("Icon buttons", theme),
        icon_buttons(true),
        icon_buttons(false),
        heading("Floating action buttons", theme),
        fabs,
        heading("Segmented buttons", theme),
        single,
        multi,
        disabled,
    ]
    .spacing(16)
    .into()
}

pub fn chips<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let assist = chip_set(
        theme,
        [
            chip::assist(theme, "Assist").on_press(press()).into(),
            chip::assist(theme, "With icon")
                .icon(symbol::calendar_today(false))
                .on_press(press())
                .into(),
            chip::assist(theme, "Elevated")
                .elevated(true)
                .on_press(press())
                .into(),
            chip::assist(theme, "Disabled").into(),
            chip::assist(theme, "Disabled elevated")
                .elevated(true)
                .into(),
        ],
    );
    let filters = chip_set(
        theme,
        ["Flat", "Selected", "Elevated", "Dropdown"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                let mut c = chip::filter(theme, label)
                    .selected(demo.filters[i])
                    .elevated(i == 2)
                    .on_press(Message::Demo(Event::Filter(i)));
                if i == 3 {
                    c = c.trailing_icon(symbol::arrow_drop_down(false));
                }
                c.into()
            })
            .chain([chip::filter(theme, "Disabled").selected(true).into()]),
    );
    let inputs = chip_set(
        theme,
        demo.inputs.iter().enumerate().map(|(i, label)| {
            chip::input(theme, label.as_str())
                .selected(i == 0)
                .on_press(press())
                .on_remove(Message::Demo(Event::Remove(i)))
                .into()
        }),
    );
    let suggestions = chip_set(
        theme,
        [
            chip::suggestion(theme, "Suggestion")
                .on_press(press())
                .into(),
            chip::suggestion(theme, "Elevated")
                .elevated(true)
                .on_press(press())
                .into(),
            chip::suggestion(theme, "Disabled").into(),
        ],
    );
    column![
        heading("Assist", theme),
        assist,
        heading("Filter", theme),
        filters,
        heading("Input", theme),
        inputs,
        heading("Suggestion", theme),
        suggestions,
    ]
    .spacing(16)
    .into()
}

pub fn containment<'a>(theme: &Theme) -> Element<'a, Message, Theme> {
    let body = |title: &'a str, theme: &Theme| -> Element<'a, Message, Theme> {
        column![
            styled(title, theme.typography.title_medium),
            text("Supporting text"),
        ]
        .spacing(4)
        .into()
    };
    let cards = row![
        card::elevated(theme, body("Elevated", theme))
            .padding(16)
            .width(Length::Fill)
            .on_press(press()),
        card::filled(theme, body("Filled", theme))
            .padding(16)
            .width(Length::Fill)
            .on_press(press()),
        card::outlined(theme, body("Outlined", theme))
            .padding(16)
            .width(Length::Fill)
            .on_press(press()),
        card::outlined(theme, body("Static", theme))
            .padding(16)
            .width(Length::Fill),
        card::filled(theme, body("Disabled", theme))
            .padding(16)
            .width(Length::Fill)
            .disabled(true),
    ]
    .spacing(16);
    let anchor = || icon(symbol::menu(false), 24.0, theme.colors.on_surface_variant);
    let badges = row![
        badged(anchor(), badge::small(theme)),
        badged(anchor(), badge::large(theme, "3")),
        badged(anchor(), badge::large(theme, "999+")),
    ]
    .spacing(32);
    column![
        heading("Cards", theme),
        cards,
        heading("Dividers", theme),
        divider::horizontal(theme),
        divider::horizontal(theme).inset(16.0, 0.0),
        row![
            space().width(Length::Fixed(40.0)),
            divider::vertical(theme),
            space().width(Length::Fixed(40.0)),
        ]
        .height(Length::Fixed(48.0)),
        heading("Badges", theme),
        badges,
    ]
    .spacing(16)
    .into()
}

pub fn selection<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let checks = |enabled: bool| {
        let mut items = row![].spacing(8).align_y(Alignment::Center);
        for i in 0..3 {
            let mut c = checkbox(theme, demo.checks[i]).error(i == 2 && !enabled);
            if enabled {
                c = c.on_toggle(move |v| Message::Demo(Event::Check(i, v)));
            }
            items = items.push(c.indeterminate(i == 1));
        }
        items
    };
    let errors = row![
        checkbox(theme, false)
            .error(true)
            .on_toggle(|v| Message::Demo(Event::Check(0, v))),
        checkbox(theme, true)
            .error(true)
            .on_toggle(|v| Message::Demo(Event::Check(2, v))),
    ];
    let switches = |enabled: bool| {
        let mut items = row![].spacing(16).align_y(Alignment::Center);
        for i in 0..3 {
            let mut s = switch(theme, demo.switches[i]).icons(i == 2);
            if enabled {
                s = s.on_toggle(move |v| Message::Demo(Event::Switch(i, v)));
            }
            items = items.push(s);
        }
        items
    };
    let radios = radio_group(
        theme,
        ["Option A", "Option B", "Option C", "Disabled"],
        Some(demo.radio),
    )
    .disable(3)
    .on_select(|i| Message::Demo(Event::Radio(i)));
    let sliders = column![
        slider(theme, 0.0..=100.0, demo.slider)
            .labeled(true)
            .width(Length::Fixed(320.0))
            .on_change(|v| Message::Demo(Event::Slider(v))),
        slider(theme, 0.0..=100.0, demo.slider)
            .step(10.0)
            .ticks(true)
            .labeled(true)
            .width(Length::Fixed(320.0))
            .on_change(|v| Message::Demo(Event::Slider(v))),
        range_slider(theme, 0.0..=100.0, demo.range)
            .step(5.0)
            .labeled(true)
            .width(Length::Fixed(320.0))
            .on_range_change(|a, b| Message::Demo(Event::Range(a, b))),
        slider(theme, 0.0..=100.0, demo.slider).width(Length::Fixed(320.0)),
    ]
    .spacing(8);
    column![
        heading("Checkboxes", theme),
        checks(true),
        checks(false),
        errors,
        heading("Switches", theme),
        switches(true),
        switches(false),
        heading("Radio buttons", theme),
        radios,
        heading("Sliders", theme),
        sliders,
    ]
    .spacing(16)
    .into()
}

pub fn text_fields<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let input = |i: usize| move |v| Message::Demo(Event::Text(i, v));
    let width = Length::Fixed(280.0);
    let filled = column![
        text_field::filled(theme, &demo.texts[0])
            .label("Label")
            .supporting_text("Supporting text")
            .width(width)
            .on_input(input(0)),
        text_field::filled(theme, &demo.texts[1])
            .label("With icons")
            .leading_icon(symbol::search(false))
            .trailing_icon(symbol::close(false))
            .on_trailing_icon_press(Message::Demo(Event::Text(1, String::new())))
            .width(width)
            .on_input(input(1)),
        text_field::filled(theme, &demo.texts[2])
            .label("Amount")
            .prefix("$")
            .suffix("USD")
            .placeholder("0.00")
            .width(width)
            .on_input(input(2)),
        text_field::filled(theme, &demo.texts[3])
            .label("Email")
            .supporting_text("Supporting text")
            .error_text("Enter a valid address")
            .error(!demo.texts[3].contains("@example.org"))
            .width(width)
            .on_input(input(3)),
        text_field::filled(theme, &demo.texts[4])
            .label("Password")
            .secure(demo.secret)
            .trailing_icon(symbol::keyboard(false))
            .on_trailing_icon_press(Message::Demo(Event::ToggleSecret))
            .width(width)
            .on_input(input(4)),
        text_field::filled(theme, &demo.texts[5])
            .label("Counter")
            .max_length(20)
            .width(width)
            .on_input(input(5)),
        text_field::filled(theme, "Disabled")
            .label("Disabled")
            .supporting_text("Supporting text")
            .width(width),
        text_field::filled_area(theme, &demo.area)
            .label("Multi line")
            .max_length(200)
            .width(width)
            .on_action(|a| Message::Demo(Event::Area(a))),
    ]
    .spacing(16);
    let outlined = column![
        text_field::outlined(theme, &demo.texts[0])
            .label("Label")
            .supporting_text("Supporting text")
            .width(width)
            .on_input(input(0)),
        text_field::outlined(theme, &demo.texts[1])
            .label("With icons")
            .leading_icon(symbol::search(false))
            .trailing_icon(symbol::close(false))
            .on_trailing_icon_press(Message::Demo(Event::Text(1, String::new())))
            .width(width)
            .on_input(input(1)),
        text_field::outlined(theme, &demo.texts[2])
            .label("Amount")
            .prefix("$")
            .suffix("USD")
            .placeholder("0.00")
            .width(width)
            .on_input(input(2)),
        text_field::outlined(theme, &demo.texts[3])
            .label("Email")
            .supporting_text("Supporting text")
            .error_text("Enter a valid address")
            .error(!demo.texts[3].contains("@example.org"))
            .width(width)
            .on_input(input(3)),
        text_field::outlined(theme, &demo.texts[4])
            .label("Password")
            .secure(demo.secret)
            .trailing_icon(symbol::keyboard(false))
            .on_trailing_icon_press(Message::Demo(Event::ToggleSecret))
            .width(width)
            .on_input(input(4)),
        text_field::outlined(theme, &demo.texts[5])
            .label("Counter")
            .max_length(20)
            .width(width)
            .on_input(input(5)),
        text_field::outlined(theme, "Disabled")
            .label("Disabled")
            .supporting_text("Supporting text")
            .width(width),
        text_field::outlined_area(theme, &demo.area)
            .label("Multi line")
            .max_length(200)
            .width(width)
            .on_action(|a| Message::Demo(Event::Area(a))),
    ]
    .spacing(16);
    column![
        heading("Filled and outlined text fields", theme),
        row![filled, outlined].spacing(32),
    ]
    .spacing(16)
    .into()
}

pub fn progress<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    column![
        heading("Linear, determinate", theme),
        linear(theme, Some(demo.progress)).width(Length::Fixed(320.0)),
        slider(theme, 0.0..=1.0, demo.progress)
            .step(0.05)
            .labeled(true)
            .format(|v| format!("{:.0}%", v * 100.0))
            .width(Length::Fixed(320.0))
            .on_change(|v| Message::Demo(Event::Progress(v))),
        heading("Linear, indeterminate", theme),
        linear(theme, None).width(Length::Fixed(320.0)),
        heading("Circular", theme),
        row![
            circular(theme, Some(demo.progress)),
            circular(theme, Some(1.0)),
            circular(theme, None),
        ]
        .spacing(24),
    ]
    .spacing(16)
    .into()
}

pub fn lists<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let select = |i: usize| Message::Demo(Event::List(i));
    column![
        heading("Lists", theme),
        container(column![
            list_item(theme, "One line")
                .leading(Leading::Icon(symbol::search(false)))
                .selected(demo.list == 0)
                .on_press(select(0)),
            divider::horizontal(theme).inset(16.0, 0.0),
            list_item(theme, "Two lines")
                .supporting_text("Supporting text")
                .leading(Leading::Avatar("AB".into()))
                .trailing(Trailing::Text("12:30".into()))
                .selected(demo.list == 1)
                .on_press(select(1)),
            divider::horizontal(theme).inset(16.0, 0.0),
            list_item(theme, "Three lines")
                .overline("Overline")
                .supporting_text("Supporting text")
                .leading(Leading::Icon(symbol::calendar_today(false)))
                .trailing(Trailing::Icon(symbol::chevron_right(false)))
                .selected(demo.list == 2)
                .on_press(select(2)),
            divider::horizontal(theme).inset(16.0, 0.0),
            list_item(theme, "With a checkbox")
                .leading(Leading::Element(
                    checkbox(theme, demo.checks[0])
                        .on_toggle(|v| Message::Demo(Event::Check(0, v)))
                        .into(),
                ))
                .trailing(Trailing::Element(
                    switch(theme, demo.switches[0])
                        .on_toggle(|v| Message::Demo(Event::Switch(0, v)))
                        .into(),
                ))
                .on_press(select(3)),
            divider::horizontal(theme).inset(16.0, 0.0),
            list_item(theme, "Disabled")
                .supporting_text("Supporting text")
                .leading(Leading::Icon(symbol::edit(false)))
                .disabled(true)
                .on_press(select(4)),
        ])
        .width(Length::Fixed(400.0)),
    ]
    .spacing(16)
    .into()
}

pub fn overlays<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let menu_button = menu_widget(
        button::outlined(theme, "Open menu").on_press(Message::Demo(Event::Menu(!demo.menu))),
        vec![
            item("Cut")
                .leading_icon(symbol::edit(false))
                .shortcut("Ctrl+X")
                .on_select(Message::Demo(Event::Press)),
            item("Copy")
                .shortcut("Ctrl+C")
                .on_select(Message::Demo(Event::Press)),
            item("Paste").disabled(true),
            menu::divider(),
            item("Share").submenu(vec![
                item("Mail").on_select(Message::Demo(Event::Press)),
                item("Messages").on_select(Message::Demo(Event::Press)),
                item("More").submenu(vec![item("Other").on_select(Message::Demo(Event::Press))]),
            ]),
        ],
        demo.menu,
    )
    .on_dismiss(Message::Demo(Event::Menu(false)))
    .build(theme);
    let selects = row![
        select::filled(["Apple", "Banana", "Cherry"], demo.select, demo.select_open)
            .label("Filled select")
            .width(Length::Fixed(220.0))
            .on_toggle(Message::Demo(Event::SelectOpen(!demo.select_open)))
            .on_select(|i| Message::Demo(Event::SelectChoice(i)))
            .on_dismiss(Message::Demo(Event::SelectOpen(false)))
            .build(theme),
        select::outlined(["Apple", "Banana", "Cherry"], None::<usize>, false)
            .label("Disabled")
            .width(Length::Fixed(220.0))
            .build(theme),
    ]
    .spacing(16);
    let tips = row![
        tooltip::plain(
            theme,
            button::filled_tonal(theme, "Plain tooltip").on_press(Message::Demo(Event::Press)),
            "Supporting text"
        ),
        tooltip::rich(
            button::filled_tonal(theme, "Rich tooltip").on_press(Message::Demo(Event::Press)),
            "Supporting text that explains the element in more detail."
        )
        .subhead("Subhead")
        .action("Action", Message::Demo(Event::Press))
        .build(theme),
    ]
    .spacing(16);
    let snack = if demo.snackbar {
        Some(
            snackbar("Message archived")
                .action("Undo", Message::Demo(Event::Press))
                .on_dismiss(Message::Demo(Event::Snackbar(false)))
                .build(theme),
        )
    } else {
        None
    };
    let content: Element<'a, Message, Theme> = column![
        heading("Menu", theme),
        menu_button,
        heading("Select", theme),
        selects,
        heading("Tooltips", theme),
        tips,
        heading("Dialog", theme),
        button::filled(theme, "Open dialog").on_press(Message::Demo(Event::Dialog(true))),
        heading("Snackbar", theme),
        button::text(theme, "Show snackbar").on_press(Message::Demo(Event::Snackbar(true))),
    ]
    .extend(snack)
    .spacing(16)
    .into();
    modal(
        content,
        dialog()
            .icon(symbol::edit(false))
            .headline("Discard draft?")
            .supporting_text("The draft is deleted and cannot be restored.")
            .action("Cancel", Message::Demo(Event::Dialog(false)))
            .action("Discard", Message::Demo(Event::Dialog(false))),
        demo.dialog,
    )
    .on_dismiss(Message::Demo(Event::Dialog(false)))
    .build(theme)
}

fn destinations() -> Vec<Destination> {
    vec![
        Destination::symbol("Search", symbol::search),
        Destination::symbol("Date", symbol::calendar_today),
        Destination::symbol("Time", symbol::schedule).badge(Badge::Count("12".into())),
        Destination::symbol("Edit", symbol::edit).badge(Badge::Dot),
    ]
}

pub fn navigation<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let select = |i| Message::Demo(Event::Destination(i));
    let bars = column![
        heading("Navigation bar", theme),
        container(navigation_bar(
            theme,
            destinations(),
            demo.destination,
            select
        ))
        .width(Length::Fixed(400.0)),
        heading("Navigation rail", theme),
        row![
            container(navigation_rail(
                theme,
                destinations(),
                demo.destination,
                select
            ))
            .height(Length::Fixed(360.0)),
            container(
                navigation_rail(theme, destinations(), demo.destination, select)
                    .labels(Labels::Selected)
                    .header(
                        fab::fab(theme, symbol::edit(false)).on_press(Message::Demo(Event::Press))
                    )
            )
            .height(Length::Fixed(360.0)),
        ]
        .spacing(24),
    ]
    .spacing(16);
    let drawer = navigation_drawer(
        vec![
            section(vec![
                DrawerItem::new("Inbox").symbol(symbol::search).badge("24"),
                DrawerItem::new("Outbox").symbol(symbol::edit),
            ])
            .headline("Mail"),
            section(vec![DrawerItem::new("Trash").symbol(symbol::close)]),
        ],
        demo.destination.min(2),
        move |i| Message::Demo(Event::Destination(i)),
    )
    .shrink_height()
    .build(theme);
    let tabs = column![
        heading("Primary tabs", theme),
        tabs::primary(
            theme,
            vec![Tab::new("Flights"), Tab::new("Trips"), Tab::new("Explore")],
            demo.tab,
            |i| Message::Demo(Event::Tab(i))
        ),
        heading("Primary tabs with icons", theme),
        tabs::primary(
            theme,
            vec![
                Tab::new("Flights").icon(symbol::search(false)),
                Tab::new("Trips").icon(symbol::calendar_today(false)),
                Tab::new("Explore").icon(symbol::schedule(false)),
            ],
            demo.tab,
            |i| Message::Demo(Event::Tab(i))
        ),
        heading("Secondary tabs", theme),
        tabs::secondary(
            theme,
            vec![Tab::new("Video"), Tab::new("Photos"), Tab::new("Audio")],
            demo.tab,
            |i| Message::Demo(Event::Tab(i))
        ),
    ]
    .spacing(8);
    let app_bars = column![
        heading("Top app bars", theme),
        top_app_bar::small("Small")
            .navigation(menu_button(theme, Message::Demo(Event::Drawer(true))))
            .action(action_button(
                theme,
                symbol::search(false),
                Message::Demo(Event::Press)
            ))
            .build(theme),
        top_app_bar::center_aligned("Center-aligned")
            .navigation(menu_button(theme, Message::Demo(Event::Drawer(true))))
            .action(action_button(
                theme,
                symbol::more_vert(false),
                Message::Demo(Event::Press)
            ))
            .build(theme),
        top_app_bar::medium("Medium")
            .navigation(menu_button(theme, Message::Demo(Event::Drawer(true))))
            .action(action_button(
                theme,
                symbol::more_vert(false),
                Message::Demo(Event::Press)
            ))
            .build(theme),
        top_app_bar::large("Large")
            .navigation(menu_button(theme, Message::Demo(Event::Drawer(true))))
            .action(action_button(
                theme,
                symbol::more_vert(false),
                Message::Demo(Event::Press)
            ))
            .build(theme),
    ]
    .spacing(8);
    let search_views = column![
        heading("Search", theme),
        search(&demo.search)
            .placeholder("Search")
            .on_input(|t| Message::Demo(Event::Search(t)))
            .trailing(action_button(
                theme,
                symbol::close(false),
                Message::Demo(Event::Search(String::new()))
            ))
            .build(theme),
        search(&demo.search)
            .placeholder("Search")
            .on_input(|t| Message::Demo(Event::Search(t)))
            .results(column![
                list_item(theme, "Result one").leading(Leading::Icon(symbol::search(false))),
                list_item(theme, "Result two").leading(Leading::Icon(symbol::search(false))),
            ])
            .build(theme),
    ]
    .spacing(8);
    let sheet_buttons = column![
        heading("Sheets and drawer", theme),
        row![
            button::filled_tonal(theme, "Modal drawer")
                .on_press(Message::Demo(Event::Drawer(true))),
            button::filled_tonal(theme, "Modal side sheet")
                .on_press(Message::Demo(Event::Sheet(true))),
        ]
        .spacing(12),
        container(
            side_sheet("Standard sheet", text("Sheet content"))
                .action("Save", Message::Demo(Event::Press))
                .build(theme)
        )
        .height(Length::Fixed(220.0)),
    ]
    .spacing(8);
    let page: Element<'a, Message, Theme> = column![
        bars,
        row![
            drawer,
            column![tabs, search_views, sheet_buttons].spacing(16)
        ]
        .spacing(24),
        app_bars
    ]
    .spacing(24)
    .into();
    let with_drawer = modal_drawer(
        navigation_drawer(
            vec![
                section(vec![
                    DrawerItem::new("Inbox").symbol(symbol::search),
                    DrawerItem::new("Outbox").symbol(symbol::edit),
                ])
                .headline("Mail"),
            ],
            demo.destination.min(1),
            |i| Message::Demo(Event::Destination(i)),
        ),
        demo.drawer,
    )
    .on_dismiss(Message::Demo(Event::Drawer(false)))
    .build(theme, page);
    side_sheet("Modal side sheet", text("Sheet content"))
        .on_close(Message::Demo(Event::Sheet(false)))
        .action("Save", Message::Demo(Event::Sheet(false)))
        .modal(
            theme,
            with_drawer,
            demo.sheet,
            Some(Message::Demo(Event::Sheet(false))),
        )
}

pub fn pickers<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let today = Date::new(2025, 8, 12).unwrap();
    let chosen = demo.date.selected().map_or("No date".to_string(), |d| {
        format!("{}-{:02}-{:02}", d.year, d.month, d.day)
    });
    let time = format!("{:02}:{:02}", demo.time.hour(), demo.time.minute());
    let docked_anchor = button::outlined(theme, format!("Docked: {chosen}"))
        .on_press(Message::Demo(Event::DateDocked(!demo.date_docked)));
    let docked = date_picker::docked(&demo.date, today, |e| Message::Demo(Event::Date(e)))
        .on_cancel(Message::Demo(Event::DateDocked(false)))
        .on_confirm(Message::Demo(Event::DateDocked(false)))
        .below(
            theme,
            docked_anchor,
            demo.date_docked,
            Some(Message::Demo(Event::DateDocked(false))),
        );
    let page: Element<'a, Message, Theme> = column![
        heading("Date picker", theme),
        row![
            button::filled(theme, format!("Modal: {chosen}"))
                .on_press(Message::Demo(Event::DateDialog(true))),
            docked,
        ]
        .spacing(16),
        heading("Time picker", theme),
        row![
            button::filled(theme, format!("Modal: {time}"))
                .on_press(Message::Demo(Event::TimeDialog(true))),
            switch(theme, demo.twenty_four).on_toggle(|v| Message::Demo(Event::Pm(v))),
            text("24 hour clock"),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    ]
    .spacing(16)
    .into();
    let with_time = time_picker::time_picker(&demo.time, |e| Message::Demo(Event::Time(e)))
        .on_cancel(Message::Demo(Event::TimeDialog(false)))
        .on_confirm(Message::Demo(Event::TimeDialog(false)))
        .over(
            theme,
            page,
            demo.time_dialog,
            Some(Message::Demo(Event::TimeDialog(false))),
        );
    date_picker::modal(&demo.date, today, |e| Message::Demo(Event::Date(e)))
        .on_cancel(Message::Demo(Event::DateDialog(false)))
        .on_confirm(Message::Demo(Event::DateDialog(false)))
        .over(
            theme,
            with_time,
            demo.date_dialog,
            Some(Message::Demo(Event::DateDialog(false))),
        )
}

fn swatch<'a>(index: usize, theme: &Theme) -> Element<'a, Message, Theme> {
    let c = theme.colors;
    let (fill, ink) = [
        (c.primary_container, c.on_primary_container),
        (c.secondary_container, c.on_secondary_container),
        (c.tertiary_container, c.on_tertiary_container),
    ][index];
    container(
        text(format!("Panel {}", index + 1))
            .size(28)
            .style(move |_: &Theme| iced::widget::text::Style { color: Some(ink) }),
    )
    .center(Length::Fill)
    .style(move |_: &Theme| container::Style {
        background: Some(fill.into()),
        ..container::Style::default()
    })
    .into()
}

pub fn motion<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let c = theme.colors;
    let go = |pattern: usize, forward: bool| Message::Demo(Event::Swatch(pattern, forward));
    let current = swatch(demo.swatch, theme);
    let previous = demo.swatch_previous.map(|i| swatch(i, theme));
    let done = Message::Demo(Event::SwatchDone);
    let stage: Element<'a, Message, Theme> = match demo.pattern {
        0 => fade_through(theme, c.surface, current, previous)
            .on_finished(done)
            .into(),
        n => shared_axis(
            theme,
            c.surface,
            [Axis::X, Axis::Y, Axis::Z][n - 1],
            demo.forward,
            current,
            previous,
        )
        .on_finished(done)
        .into(),
    };
    let anchor = card::filled(
        theme,
        column![
            styled("Container transform", theme.typography.title_medium),
            text("Press the card"),
        ]
        .spacing(4),
    )
    .padding(16)
    .width(220)
    .on_press(Message::Demo(Event::Expand(true)));
    let detail: Element<'a, Message, Theme> = container(
        column![
            styled("Detail", theme.typography.headline_medium),
            text("The card grew into this page."),
            button::filled(theme, "Close").on_press(Message::Demo(Event::Expand(false))),
        ]
        .spacing(16),
    )
    .padding(48)
    .center_x(Length::Fill)
    .style(move |_: &Theme| container::Style {
        background: Some(c.surface_container_high.into()),
        ..container::Style::default()
    })
    .into();
    let transformed = container_transform(anchor, detail, demo.expanded)
        .color(c.surface_container_high)
        .shape(theme.shape.medium)
        .on_dismiss(Message::Demo(Event::Expand(false)))
        .build(theme);
    let faded = fade(
        theme,
        c.surface,
        card::elevated(theme, text("Fades and scales in")).padding(16),
        demo.fade,
    );
    column![
        heading("Fade through and shared axis", theme),
        container(stage)
            .width(Length::Fixed(360.0))
            .height(Length::Fixed(180.0)),
        row![
            button::filled_tonal(theme, "Fade through").on_press(go(0, true)),
            button::filled_tonal(theme, "Axis X").on_press(go(1, true)),
            button::filled_tonal(theme, "Axis X back").on_press(go(1, false)),
            button::filled_tonal(theme, "Axis Y").on_press(go(2, true)),
            button::filled_tonal(theme, "Axis Z").on_press(go(3, true)),
        ]
        .spacing(8),
        heading("Container transform", theme),
        transformed,
        heading("Fade", theme),
        row![
            switch(theme, demo.fade).on_toggle(|v| Message::Demo(Event::FadeToggle(v))),
            container(faded).height(Length::Fixed(64.0)),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    ]
    .spacing(16)
    .into()
}

pub fn adaptive_page<'a>(demo: &'a Demo, theme: &Theme) -> Element<'a, Message, Theme> {
    let widths = [("Compact", 400.0), ("Medium", 700.0), ("Expanded", 900.0)];
    let width = widths[demo.preview].1;
    let theme_owned = theme.clone();
    let destination = demo.destination;
    let mail = demo.mail;
    let app = adaptive(move |class, _| {
        let theme = &theme_owned;
        let destinations = vec![
            Destination::symbol("Mail", symbol::edit),
            Destination::symbol("Photos", symbol::calendar_today),
            Destination::symbol("Files", symbol::search),
        ];
        let page: Element<'static, Message, Theme> = match destination {
            0 => {
                let subjects = ["Quarterly report", "Lunch on Friday", "Trip photos"];
                let mut list = column![].spacing(0);
                for (i, subject) in subjects.iter().enumerate() {
                    list = list.push(
                        list_item(theme, *subject)
                            .supporting_text("Message preview text")
                            .leading(Leading::Avatar(subject[..1].to_string()))
                            .selected(mail == Some(i))
                            .on_press(Message::Demo(Event::Mail(Some(i)))),
                    );
                }
                let detail = mail.map(|i| {
                    let back: Element<'static, Message, Theme> = button::text(theme, "Back")
                        .on_press(Message::Demo(Event::Mail(None)))
                        .into();
                    container(
                        column![
                            back,
                            styled(subjects[i], theme.typography.headline_small),
                            text("The detail pane shows next to the list on wide windows."),
                        ]
                        .spacing(12),
                    )
                    .padding(16)
                    .into()
                });
                list_detail(class.width, false, list, detail)
            }
            1 => margins(
                class.width,
                feed(
                    class.width,
                    (1..=6).map(|i| {
                        card::filled(
                            theme,
                            container(text(format!("Photo {i}"))).center(Length::Fill),
                        )
                        .height(96)
                        .into()
                    }),
                ),
            ),
            _ => container(text("Files")).center(Length::Fill).into(),
        };
        scaffold(
            theme,
            class.width,
            destinations,
            destination,
            |i| Message::Demo(Event::Destination(i)),
            page,
        )
    });
    column![
        heading("Adaptive layout", theme),
        row(widths.iter().enumerate().map(|(i, (label, _))| {
            let b = if demo.preview == i {
                button::filled(theme, *label)
            } else {
                button::outlined(theme, *label)
            };
            b.on_press(Message::Demo(Event::Preview(i))).into()
        }))
        .spacing(8),
        container(
            container(app)
                .width(Length::Fixed(width))
                .height(Length::Fixed(520.0))
        )
        .padding(1)
        .style(|t: &Theme| container::Style {
            border: iced::Border {
                color: t.colors.outline_variant,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }),
    ]
    .spacing(16)
    .into()
}
