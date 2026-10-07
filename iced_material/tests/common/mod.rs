// SPDX-License-Identifier: LGPL-3.0-only

#![allow(dead_code)]

use std::sync::Once;
use std::time::Duration;

use iced::advanced::clipboard;
use iced::advanced::renderer::{Headless, Style};
use iced::advanced::widget::{self, Operation, operation::Focusable};
use iced::time::Instant;
use iced::{Color, Event, Font, Pixels, Point, Rectangle, Size, keyboard, mouse, window};
use material_iced::{Element, Theme};
use iced_runtime::user_interface::{Cache, UserInterface};

static FONTS: Once = Once::new();

fn load_fonts() {
    FONTS.call_once(|| {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/");
        for file in ["Roboto-Regular.ttf", "Roboto-Medium.ttf", "Roboto-Bold.ttf"] {
            let bytes = std::fs::read(format!("{dir}{file}")).unwrap();
            iced::advanced::graphics::text::font_system()
                .write()
                .unwrap()
                .load_font(bytes.into());
        }
    });
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }

    pub fn at(&self, point: Point) -> [u8; 4] {
        self.pixel(
            (point.x * self.scale).floor() as u32,
            (point.y * self.scale).floor() as u32,
        )
    }
}

pub fn rgb(color: Color) -> [u8; 4] {
    color.into_rgba8()
}

pub fn distance(a: [u8; 4], b: [u8; 4]) -> u32 {
    a.iter()
        .zip(b.iter())
        .take(3)
        .map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs())
        .max()
        .unwrap()
}

struct Collect(Vec<Rectangle>);

impl Operation for Collect {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        _state: &mut dyn Focusable,
    ) {
        self.0.push(bounds);
    }
}

pub fn key(named: keyboard::key::Named, shift: bool) -> Vec<Event> {
    let modifiers = if shift {
        keyboard::Modifiers::SHIFT
    } else {
        keyboard::Modifiers::empty()
    };
    let key = keyboard::Key::Named(named);
    vec![
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }),
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers,
        }),
    ]
}

pub fn type_text(text: &str) -> Vec<Event> {
    let key = keyboard::Key::Character(text.into());
    vec![Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: Some(text.into()),
        repeat: false,
    })]
}

pub fn ink(image: &Image, area: Rectangle, background: [u8; 4]) -> Option<Rectangle> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in area.y as u32..(area.y + area.height) as u32 {
        for x in area.x as u32..(area.x + area.width) as u32 {
            if distance(image.pixel(x, y), background) > 24 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (x0 != u32::MAX).then(|| {
        Rectangle::new(
            Point::new(x0 as f32, y0 as f32),
            Size::new((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32),
        )
    })
}

pub fn solid(image: &Image, area: Rectangle, color: [u8; 4]) -> Option<Rectangle> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in area.y as u32..(area.y + area.height) as u32 {
        for x in area.x as u32..(area.x + area.width) as u32 {
            if distance(image.pixel(x, y), color) == 0 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (x0 != u32::MAX).then(|| {
        Rectangle::new(
            Point::new(x0 as f32, y0 as f32),
            Size::new((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32),
        )
    })
}

pub fn run(image: &Image, from: Point, dx: i32, dy: i32) -> i32 {
    let color = image.at(from);
    let (mut x, mut y, mut n) = (from.x as i32, from.y as i32, 0);
    loop {
        x += dx;
        y += dy;
        if x < 0 || y < 0 || x >= image.width as i32 || y >= image.height as i32 {
            return n;
        }
        if image.pixel(x as u32, y as u32) != color {
            return n;
        }
        n += 1;
    }
}

pub struct Harness {
    pub renderer: iced::Renderer,
    pub size: Size,
    pub cache: Option<Cache>,
    pub start: Instant,
    pub cursor: mouse::Cursor,
}

impl Harness {
    pub fn new(size: Size) -> Harness {
        let backend = std::env::var("ICED_MATERIAL_BACKEND").unwrap_or("tiny-skia".to_string());
        Harness::with_backend(size, &backend)
    }

    pub fn with_backend(size: Size, backend: &str) -> Harness {
        load_fonts();
        let renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            Font::with_name("Roboto"),
            Pixels(14.0),
            Some(backend),
        ))
        .unwrap_or_else(|| panic!("{backend} renderer"));
        Harness {
            renderer,
            size,
            cache: Some(Cache::default()),
            start: Instant::now(),
            cursor: mouse::Cursor::Unavailable,
        }
    }

    pub fn update<'a, Message>(
        &mut self,
        view: Element<'a, Message>,
        events: &[Event],
    ) -> Vec<Message> {
        let mut ui = UserInterface::build(
            view,
            self.size,
            self.cache.take().unwrap(),
            &mut self.renderer,
        );
        let mut messages = Vec::new();
        let _ = ui.update(
            events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        self.cache = Some(ui.into_cache());
        messages
    }

    pub fn focusables<'a, Message>(&mut self, view: Element<'a, Message>) -> Vec<Rectangle> {
        let mut ui = UserInterface::build(
            view,
            self.size,
            self.cache.take().unwrap(),
            &mut self.renderer,
        );
        let mut collect = Collect(Vec::new());
        ui.operate(&self.renderer, &mut collect);
        self.cache = Some(ui.into_cache());
        collect.0
    }

    pub fn frame<'a, Message>(&mut self, view: Element<'a, Message>, at: Duration) -> Vec<Message> {
        let now = self.start + at;
        self.update(view, &[Event::Window(window::Event::RedrawRequested(now))])
    }

    pub fn click<'a, Message>(
        &mut self,
        view: impl Fn() -> Element<'a, Message>,
        position: Point,
    ) -> Vec<Message> {
        let mut messages = self.move_cursor(view(), position);
        messages.extend(self.update(
            view(),
            &[Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))],
        ));
        messages.extend(self.update(
            view(),
            &[Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            ))],
        ));
        messages
    }

    pub fn press<'a, Message>(
        &mut self,
        view: impl Fn() -> Element<'a, Message>,
        position: Point,
    ) -> Vec<Message> {
        let mut messages = self.move_cursor(view(), position);
        messages.extend(self.update(
            view(),
            &[Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))],
        ));
        messages
    }

    pub fn release<'a, Message>(&mut self, view: Element<'a, Message>) -> Vec<Message> {
        self.update(
            view,
            &[Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            ))],
        )
    }

    pub fn scroll<'a, Message>(
        &mut self,
        view: Element<'a, Message>,
        position: Point,
        y: f32,
    ) -> Vec<Message> {
        self.cursor = mouse::Cursor::Available(position);
        self.update(
            view,
            &[Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y },
            })],
        )
    }

    pub fn move_cursor<'a, Message>(
        &mut self,
        view: Element<'a, Message>,
        position: Point,
    ) -> Vec<Message> {
        self.cursor = mouse::Cursor::Available(position);
        self.update(
            view,
            &[Event::Mouse(mouse::Event::CursorMoved { position })],
        )
    }

    pub fn screenshot<'a, Message>(
        &mut self,
        view: Element<'a, Message>,
        theme: &Theme,
        scale: f32,
    ) -> Image {
        let mut ui = UserInterface::build(
            view,
            self.size,
            self.cache.take().unwrap(),
            &mut self.renderer,
        );
        let mut ignored = Vec::new();
        let _ = ui.update(
            &[],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut ignored,
        );
        ui.draw(
            &mut self.renderer,
            theme,
            &Style {
                text_color: theme.colors.on_surface,
            },
            self.cursor,
        );
        self.cache = Some(ui.into_cache());
        let physical = Size::new(
            (self.size.width * scale).round() as u32,
            (self.size.height * scale).round() as u32,
        );
        let rgba = self
            .renderer
            .screenshot(physical, scale, theme.colors.surface);
        Image {
            width: physical.width,
            height: physical.height,
            scale,
            rgba,
        }
    }
}
