// SPDX-License-Identifier: LGPL-3.0-only

mod components;
mod controls;
mod foundations;
#[cfg(test)]
mod smoke;

use iced::time::Instant;
use iced::widget::{column, container, row, scrollable};
use iced::{Color, Element, Font, Length, Subscription, Task, system, theme, window};
use material_iced::color::{ColorScheme, SchemeOptions, SpecVersion, Variant, matugen};
use material_iced::motion::Transition;
use material_iced::theme::ColorTransition;
use material_iced::widget::focus_scope;
use material_iced::widget::transition::fade_through;
use material_iced::{Theme, font};

fn main() -> iced::Result {
    let mut app = iced::application(Gallery::new, Gallery::update, Gallery::view)
        .title("material-iced gallery")
        .theme(Gallery::theme)
        .subscription(Gallery::subscription)
        .default_font(Font::with_name("Roboto"))
        .window_size((1280.0, 860.0));
    for bytes in font::ROBOTO {
        app = app.font(bytes);
    }
    app.run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Color,
    Typography,
    Shape,
    Elevation,
    Actions,
    Chips,
    Containment,
    Selection,
    TextFields,
    Progress,
    Lists,
    Overlays,
    Navigation,
    Pickers,
    Adaptive,
    Motion,
}

#[derive(Debug, Clone)]
pub enum Message {
    Seed(Color),
    SeedInput(String),
    Variant(Variant),
    Contrast(f64),
    Spec(SpecVersion),
    Mode(Mode),
    SystemMode(theme::Mode),
    MatugenPath(String),
    LoadMatugen,
    ClearMatugen,
    Page(Page),
    Frame(Instant),
    Demo(components::Event),
    PageDone,
}

pub struct Gallery {
    pub options: SchemeOptions,
    pub seed_input: String,
    pub mode: Mode,
    pub system_dark: bool,
    pub matugen_path: String,
    pub matugen: Option<(ColorScheme, ColorScheme)>,
    pub status: String,
    pub page: Page,
    previous: Option<Page>,
    demo: components::Demo,
    colors: ColorScheme,
    transition: Option<ColorTransition>,
    now: Instant,
}

impl Gallery {
    fn new() -> (Self, Task<Message>) {
        let options = SchemeOptions::default();
        let gallery = Gallery {
            options,
            seed_input: "#6750A4".to_string(),
            mode: Mode::System,
            system_dark: false,
            matugen_path: String::new(),
            matugen: None,
            status: String::new(),
            page: Page::Color,
            previous: None,
            demo: components::Demo::default(),
            colors: ColorScheme::new(options),
            transition: None,
            now: Instant::now(),
        };
        (gallery, system::theme().map(Message::SystemMode))
    }

    fn dark(&self) -> bool {
        match self.mode {
            Mode::System => self.system_dark,
            Mode::Light => false,
            Mode::Dark => true,
        }
    }

    fn target(&self) -> ColorScheme {
        match &self.matugen {
            Some((light, dark)) => {
                if self.dark() {
                    *dark
                } else {
                    *light
                }
            }
            None => ColorScheme::new(SchemeOptions {
                dark: self.dark(),
                ..self.options
            }),
        }
    }

    fn retarget(&mut self) {
        let current = self.theme().colors;
        let theme = Theme::light();
        let transition = Transition {
            duration: theme.motion.duration.long2,
            easing: theme.motion.easing.standard,
        };
        self.now = Instant::now();
        self.colors = self.target();
        self.transition = Some(ColorTransition::new(
            current,
            self.colors,
            self.now,
            transition,
        ));
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Seed(color) => {
                self.options.source = color;
                let [r, g, b, _] = color.into_rgba8();
                self.seed_input = format!("#{r:02X}{g:02X}{b:02X}");
                self.retarget();
            }
            Message::SeedInput(input) => {
                if let Ok(color) = input.parse::<Color>() {
                    self.options.source = color;
                    self.retarget();
                }
                self.seed_input = input;
            }
            Message::Variant(variant) => {
                self.options.variant = variant;
                self.retarget();
            }
            Message::Contrast(contrast) => {
                self.options.contrast = contrast;
                self.retarget();
            }
            Message::Spec(spec) => {
                self.options.spec = spec;
                self.retarget();
            }
            Message::Mode(mode) => {
                self.mode = mode;
                self.retarget();
            }
            Message::SystemMode(mode) => {
                self.system_dark = mode == theme::Mode::Dark;
                self.retarget();
            }
            Message::MatugenPath(path) => self.matugen_path = path,
            Message::LoadMatugen => {
                match std::fs::read_to_string(&self.matugen_path)
                    .map_err(|e| e.to_string())
                    .and_then(|json| matugen::import(&json).map_err(|e| e.to_string()))
                {
                    Ok(schemes) => {
                        self.matugen = Some(schemes);
                        self.status = "Loaded".to_string();
                        self.retarget();
                    }
                    Err(error) => self.status = error,
                }
            }
            Message::ClearMatugen => {
                self.matugen = None;
                self.status.clear();
                self.retarget();
            }
            Message::Page(page) => {
                if page != self.page {
                    self.previous = Some(self.page);
                    self.page = page;
                }
            }
            Message::PageDone => self.previous = None,
            Message::Demo(event) => self.demo.update(event),
            Message::Frame(now) => {
                self.now = now;
                if self.transition.as_ref().is_some_and(|t| !t.is_running(now)) {
                    self.transition = None;
                }
            }
        }
    }

    fn theme(&self) -> Theme {
        let colors = match &self.transition {
            Some(transition) => transition.colors(self.now),
            None => self.colors,
        };
        Theme::with_colors(colors, self.dark())
    }

    fn subscription(&self) -> Subscription<Message> {
        let system = system::theme_changes().map(Message::SystemMode);
        if self.transition.is_some() {
            Subscription::batch([system, window::frames().map(Message::Frame)])
        } else {
            system
        }
    }

    fn page_view(&self, page: Page, theme: &Theme) -> Element<'_, Message, Theme> {
        match page {
            Page::Color => foundations::color(theme),
            Page::Typography => foundations::typography(theme),
            Page::Shape => foundations::shape(theme),
            Page::Elevation => foundations::elevation(theme),
            Page::Actions => components::actions(&self.demo, theme),
            Page::Chips => components::chips(&self.demo, theme),
            Page::Containment => components::containment(theme),
            Page::Selection => components::selection(&self.demo, theme),
            Page::TextFields => components::text_fields(&self.demo, theme),
            Page::Progress => components::progress(&self.demo, theme),
            Page::Lists => components::lists(&self.demo, theme),
            Page::Overlays => components::overlays(&self.demo, theme),
            Page::Navigation => components::navigation(&self.demo, theme),
            Page::Pickers => components::pickers(&self.demo, theme),
            Page::Adaptive => components::adaptive_page(&self.demo, theme),
            Page::Motion => components::motion(&self.demo, theme),
        }
    }

    fn view(&self) -> Element<'_, Message, Theme> {
        let theme = self.theme();
        let page: Element<'_, Message, Theme> = fade_through(
            &theme,
            theme.colors.surface,
            self.page_view(self.page, &theme),
            self.previous.map(|p| self.page_view(p, &theme)),
        )
        .on_finished(Message::PageDone)
        .into();
        let content = row![
            container(scrollable(controls::view(self, &theme)).height(Length::Fill))
                .width(Length::Fixed(320.0))
                .padding(16)
                .style(move |t: &Theme| container::Style {
                    background: Some(t.colors.surface_container.into()),
                    ..container::Style::default()
                }),
            scrollable(container(page).padding(24).width(Length::Fill)).height(Length::Fill),
        ];
        focus_scope(column![content].height(Length::Fill)).into()
    }
}
