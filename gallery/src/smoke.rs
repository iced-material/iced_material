// SPDX-License-Identifier: LGPL-3.0-only

use std::time::Duration;

use iced::advanced::clipboard;
use iced::advanced::renderer::{Headless, Style};
use iced::time::Instant;
use iced::{Event, Font, Pixels, Point, Size, keyboard, mouse, window};
use iced_runtime::user_interface::{Cache, UserInterface};

use crate::components::Event as DemoEvent;
use crate::{Gallery, Message, Page};

struct Rig {
    renderer: iced::Renderer,
    cache: Option<Cache>,
    start: Instant,
    cursor: mouse::Cursor,
    size: Size,
    frame: u64,
    seed: u64,
    last: Vec<u8>,
}

const PAGES: [Page; 16] = [
    Page::Color,
    Page::Typography,
    Page::Shape,
    Page::Elevation,
    Page::Actions,
    Page::Chips,
    Page::Containment,
    Page::Selection,
    Page::TextFields,
    Page::Progress,
    Page::Lists,
    Page::Overlays,
    Page::Navigation,
    Page::Pickers,
    Page::Adaptive,
    Page::Motion,
];

impl Rig {
    fn new(size: Size) -> Rig {
        for bytes in iced_material::font::ROBOTO {
            iced::advanced::graphics::text::font_system()
                .write()
                .unwrap()
                .load_font(bytes.into());
        }
        let renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            Font::with_name("Roboto"),
            Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("tiny-skia renderer");
        Rig {
            renderer,
            cache: Some(Cache::default()),
            start: Instant::now(),
            cursor: mouse::Cursor::Unavailable,
            size,
            frame: 0,
            seed: 0x2545_f491_4f6c_dd1d,
            last: Vec::new(),
        }
    }

    fn random(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed % 10_000) as f32 / 10_000.0
    }

    fn pass(&mut self, gallery: &mut Gallery, mut events: Vec<Event>) {
        self.frame += 1;
        let now = self.start + Duration::from_millis(self.frame * 16);
        events.push(Event::Window(window::Event::RedrawRequested(now)));
        let theme = gallery.theme();
        let mut messages = Vec::new();
        let cache = {
            let mut ui = UserInterface::build(
                gallery.view(),
                self.size,
                self.cache.take().unwrap(),
                &mut self.renderer,
            );
            let _ = ui.update(
                &events,
                self.cursor,
                &mut self.renderer,
                &mut clipboard::Null,
                &mut messages,
            );
            ui.draw(
                &mut self.renderer,
                &theme,
                &Style {
                    text_color: theme.colors.on_surface,
                },
                self.cursor,
            );
            ui.into_cache()
        };
        self.cache = Some(cache);
        self.last = Headless::screenshot(
            &mut self.renderer,
            Size::new(self.size.width as u32, self.size.height as u32),
            1.0,
            theme.colors.surface,
        );
        for message in messages {
            gallery.update(message);
        }
    }

    fn settle(&mut self, gallery: &mut Gallery, frames: usize) {
        for _ in 0..frames {
            self.pass(gallery, Vec::new());
        }
    }

    fn click(&mut self, gallery: &mut Gallery, at: Point) {
        self.cursor = mouse::Cursor::Available(at);
        self.pass(
            gallery,
            vec![Event::Mouse(mouse::Event::CursorMoved { position: at })],
        );
        self.pass(
            gallery,
            vec![Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))],
        );
        self.pass(
            gallery,
            vec![Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            ))],
        );
    }

    fn key(&mut self, gallery: &mut Gallery, named: keyboard::key::Named, shift: bool) {
        let key = keyboard::Key::Named(named);
        let modifiers = if shift {
            keyboard::Modifiers::SHIFT
        } else {
            keyboard::Modifiers::empty()
        };
        let physical =
            keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
        self.pass(
            gallery,
            vec![Event::Keyboard(keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key.clone(),
                physical_key: physical,
                location: keyboard::Location::Standard,
                modifiers,
                text: None,
                repeat: false,
            })],
        );
        self.pass(
            gallery,
            vec![Event::Keyboard(keyboard::Event::KeyReleased {
                key: key.clone(),
                modified_key: key,
                physical_key: physical,
                location: keyboard::Location::Standard,
                modifiers,
            })],
        );
    }

    fn fuzz(&mut self, gallery: &mut Gallery, steps: usize) {
        use keyboard::key::Named;
        let keys = [
            Named::Tab,
            Named::Enter,
            Named::Space,
            Named::Escape,
            Named::ArrowLeft,
            Named::ArrowRight,
            Named::ArrowUp,
            Named::ArrowDown,
            Named::Home,
            Named::End,
            Named::PageUp,
            Named::PageDown,
        ];
        for step in 0..steps {
            let at = Point::new(
                320.0 + self.random() * (self.size.width - 320.0),
                self.random() * self.size.height,
            );
            match step % 4 {
                0 => self.click(gallery, at),
                1 => {
                    let index = (self.random() * keys.len() as f32) as usize % keys.len();
                    self.key(gallery, keys[index], step % 8 == 1);
                }
                2 => {
                    self.cursor = mouse::Cursor::Available(at);
                    let y = self.random() * 4.0 - 2.0;
                    self.pass(
                        gallery,
                        vec![
                            Event::Mouse(mouse::Event::CursorMoved { position: at }),
                            Event::Mouse(mouse::Event::WheelScrolled {
                                delta: mouse::ScrollDelta::Lines { x: 0.0, y },
                            }),
                        ],
                    );
                }
                _ => self.settle(gallery, 3),
            }
        }
    }
}

fn open(gallery: &mut Gallery, rig: &mut Rig, page: Page) {
    gallery.update(Message::Page(page));
    rig.settle(gallery, 20);
}

#[test]
fn every_page_renders_at_two_sizes() {
    for size in [
        Size::new(1280.0, 860.0),
        Size::new(700.0, 500.0),
        Size::new(1900.0, 1000.0),
    ] {
        let (mut gallery, _) = Gallery::new();
        let mut rig = Rig::new(size);
        for page in PAGES {
            open(&mut gallery, &mut rig, page);
        }
    }
}

#[test]
fn every_page_survives_random_input() {
    for page in PAGES {
        let (mut gallery, _) = Gallery::new();
        let mut rig = Rig::new(Size::new(1280.0, 860.0));
        open(&mut gallery, &mut rig, page);
        rig.fuzz(&mut gallery, 80);
    }
}

#[test]
fn overlays_open_and_close_on_every_page() {
    let events = [
        DemoEvent::Drawer(true),
        DemoEvent::Sheet(true),
        DemoEvent::DateDialog(true),
        DemoEvent::TimeDialog(true),
        DemoEvent::DateDocked(true),
        DemoEvent::Menu(true),
        DemoEvent::SelectOpen(true),
        DemoEvent::Dialog(true),
        DemoEvent::Expand(true),
    ];
    for page in PAGES {
        let (mut gallery, _) = Gallery::new();
        let mut rig = Rig::new(Size::new(1280.0, 860.0));
        open(&mut gallery, &mut rig, page);
        for event in &events {
            gallery.update(Message::Demo(event.clone()));
            rig.settle(&mut gallery, 10);
            rig.fuzz(&mut gallery, 12);
            rig.key(&mut gallery, keyboard::key::Named::Escape, false);
            rig.settle(&mut gallery, 20);
        }
    }
}

#[test]
fn stacked_modal_layers_do_not_panic() {
    use keyboard::key::Named;
    let cases: [(Page, &[DemoEvent]); 5] = [
        (
            Page::Navigation,
            &[DemoEvent::Drawer(true), DemoEvent::Sheet(true)],
        ),
        (
            Page::Navigation,
            &[DemoEvent::Sheet(true), DemoEvent::Drawer(true)],
        ),
        (
            Page::Pickers,
            &[DemoEvent::DateDialog(true), DemoEvent::TimeDialog(true)],
        ),
        (
            Page::Pickers,
            &[DemoEvent::TimeDialog(true), DemoEvent::DateDocked(true)],
        ),
        (
            Page::Overlays,
            &[
                DemoEvent::Menu(true),
                DemoEvent::Dialog(true),
                DemoEvent::SelectOpen(true),
            ],
        ),
    ];
    for (page, events) in cases {
        for size in [Size::new(1280.0, 860.0), Size::new(900.0, 600.0)] {
            let (mut gallery, _) = Gallery::new();
            let mut rig = Rig::new(size);
            open(&mut gallery, &mut rig, page);
            for event in events {
                gallery.update(Message::Demo(event.clone()));
                rig.settle(&mut gallery, 12);
            }
            rig.fuzz(&mut gallery, 40);
            for _ in 0..3 {
                rig.key(&mut gallery, Named::Escape, false);
                rig.settle(&mut gallery, 6);
            }
            rig.key(&mut gallery, Named::Tab, false);
            rig.key(&mut gallery, Named::Tab, true);
        }
    }
}

#[test]
fn tiny_and_odd_window_sizes_do_not_panic() {
    for size in [
        Size::new(321.0, 200.0),
        Size::new(360.0, 480.0),
        Size::new(500.0, 120.0),
        Size::new(1280.0, 150.0),
        Size::new(2400.0, 900.0),
    ] {
        let (mut gallery, _) = Gallery::new();
        let mut rig = Rig::new(size);
        for page in PAGES {
            gallery.update(Message::Page(page));
            rig.settle(&mut gallery, 6);
            for event in [
                DemoEvent::Drawer(true),
                DemoEvent::Sheet(true),
                DemoEvent::DateDialog(true),
                DemoEvent::TimeDialog(true),
                DemoEvent::Dialog(true),
                DemoEvent::Menu(true),
            ] {
                gallery.update(Message::Demo(event));
                rig.settle(&mut gallery, 4);
            }
            rig.fuzz(&mut gallery, 12);
            for _ in 0..4 {
                rig.key(&mut gallery, keyboard::key::Named::Escape, false);
            }
        }
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn write_png(path: &str, width: u32, height: u32, rgba: &[u8]) {
    let mut raw = Vec::new();
    for row in rgba.chunks(width as usize * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let mut chunks = raw.chunks(65_535).peekable();
    while let Some(chunk) = chunks.next() {
        z.push(u8::from(chunks.peek().is_none()));
        z.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(chunk.len() as u16)).to_le_bytes());
        z.extend_from_slice(chunk);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut chunk = |kind: &[u8; 4], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    };
    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(b"IHDR", &header);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    std::fs::write(path, out).unwrap();
}

#[test]
#[ignore]
fn dump_screenshots() {
    let dir = std::env::var("SHOTS").unwrap_or_else(|_| "/tmp/shots".into());
    std::fs::create_dir_all(&dir).unwrap();
    let size = Size::new(1280.0, 860.0);
    let shot = |rig: &mut Rig, gallery: &mut Gallery, name: &str| {
        rig.settle(gallery, 60);
        write_png(
            &format!("{dir}/{name}.png"),
            size.width as u32,
            size.height as u32,
            &rig.last,
        );
    };
    let (mut gallery, _) = Gallery::new();
    let mut rig = Rig::new(size);
    for (page, name) in [
        (Page::Selection, "selection"),
        (Page::Containment, "containment"),
        (Page::Navigation, "navigation"),
        (Page::Adaptive, "adaptive"),
        (Page::Pickers, "pickers"),
        (Page::Motion, "motion"),
        (Page::Overlays, "overlays"),
        (Page::TextFields, "textfields"),
    ] {
        open(&mut gallery, &mut rig, page);
        shot(&mut rig, &mut gallery, name);
    }
    let tall = Size::new(1280.0, 1500.0);
    let (mut gallery_tall, _) = Gallery::new();
    let mut rig_tall = Rig::new(tall);
    open(&mut gallery_tall, &mut rig_tall, Page::Navigation);
    rig_tall.settle(&mut gallery_tall, 60);
    write_png(
        &format!("{dir}/navigation-tall.png"),
        tall.width as u32,
        tall.height as u32,
        &rig_tall.last,
    );
    open(&mut gallery, &mut rig, Page::Pickers);
    gallery.update(Message::Demo(DemoEvent::TimeDialog(true)));
    shot(&mut rig, &mut gallery, "time-hours");
    gallery.update(Message::Demo(DemoEvent::Time(
        iced_material::widget::time_picker::Event::HourDone(10),
    )));
    gallery.update(Message::Demo(DemoEvent::Time(
        iced_material::widget::time_picker::Event::Minute(21),
    )));
    shot(&mut rig, &mut gallery, "time-minutes");
    gallery.update(Message::Demo(DemoEvent::Time(
        iced_material::widget::time_picker::Event::ToggleInput,
    )));
    shot(&mut rig, &mut gallery, "time-input");
    gallery.update(Message::Demo(DemoEvent::TimeDialog(false)));
    gallery.update(Message::Demo(DemoEvent::DateDialog(true)));
    shot(&mut rig, &mut gallery, "date-modal");
    gallery.update(Message::Demo(DemoEvent::Date(
        iced_material::widget::date_picker::Event::ToggleYears,
    )));
    shot(&mut rig, &mut gallery, "date-years");
    gallery.update(Message::Demo(DemoEvent::Date(
        iced_material::widget::date_picker::Event::ToggleYears,
    )));
    gallery.update(Message::Demo(DemoEvent::Date(
        iced_material::widget::date_picker::Event::ToggleInput,
    )));
    shot(&mut rig, &mut gallery, "date-input");
}
