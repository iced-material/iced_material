// SPDX-License-Identifier: LGPL-3.0-only

use iced::widget::{column, container, text};
use iced::{Alignment, Font, Length};
use iced_material::widget::{button, focus_scope};
use iced_material::{Element, Theme, font};

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}

struct Counter {
    theme: Theme,
    value: i32,
}

impl Counter {
    fn new() -> Counter {
        Counter {
            theme: Theme::light(),
            value: 0,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.value += 1,
            Message::Decrement => self.value -= 1,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            text(self.value.to_string()).size(48),
            button::filled(&self.theme, "Increment").on_press(Message::Increment),
            button::outlined(&self.theme, "Decrement").on_press(Message::Decrement),
        ]
        .spacing(16)
        .align_x(Alignment::Center);
        focus_scope(
            container(content)
                .center(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(theme.colors.surface.into()),
                    ..container::Style::default()
                }),
        )
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn main() -> iced::Result {
    let mut app = iced::application(Counter::new, Counter::update, Counter::view)
        .theme(Counter::theme)
        .default_font(Font::with_name("Roboto"));
    for bytes in font::ROBOTO {
        app = app.font(bytes);
    }
    app.run()
}
