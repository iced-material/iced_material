# Iced Material

![License: LGPL-3.0-only](https://img.shields.io/badge/license-LGPL--3.0--only-blue)
![Rust 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange)
![Iced 0.14](https://img.shields.io/badge/iced-0.14-lightgrey)

A Material Design 3 widget and theming library for the Iced GUI toolkit. 

> [!NOTE]
> **Repository Mirrors**
>
> - **git.nuros.org** ([iced_material/iced_material](https://git.nuros.org/iced_material/iced_material)): primary, self-hosted Forgejo instance, accounts restricted to the core team.
> - **GitHub** ([iced-material/iced_material](https://github.com/iced-material/iced_material)): mirror for external contributors. Issues and Pull Requests opened here are welcome and are reviewed and processed by the core team.

## Dependencies

- Rust 1.88 or newer, edition 2024.
- Iced 0.14.0.

## Build

```
cargo build
cargo test
cargo run -p gallery
cargo run -p material-iced --example minimal --features roboto
```

`cargo test` renders widgets headless on `tiny-skia`. Setting `ICED_MATERIAL_BACKEND=wgpu` runs the same tests on `wgpu`.

## Usage

Add the crate and Iced to `Cargo.toml`. The `roboto` feature embeds the Roboto fonts that the type scale uses.

```toml
[dependencies]
iced = "0.14"
material-iced = { path = "../material-iced", features = ["roboto"] }
```

Every widget takes the theme when it is built. Wrap the root of a view in `focus_scope` to get Tab and Shift+Tab traversal.

```rust
use iced::widget::{column, container, text};
use iced::{Alignment, Font, Length};
use material_iced::widget::{button, focus_scope};
use material_iced::{Element, Theme, font};

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
```

The same program is `material-iced/examples/minimal.rs`.

A theme is built from a source color:

```rust
use iced::Color;
use material_iced::Theme;
use material_iced::color::{SchemeOptions, Variant};

let theme = Theme::new(SchemeOptions {
    source: Color::from_rgb8(0x1B, 0x6D, 0x00),
    variant: Variant::TonalSpot,
    dark: true,
    ..SchemeOptions::default()
});
```

A scheme can also be loaded from Matugen JSON, see `docs/matugen.md`. `Theme::iced_palette` converts a theme to an `iced::Theme` palette for applications that also use stock Iced widgets. The conversion keeps six colors. Stock widgets styled with it do not follow the Material component specifications.

## Versions and platforms

The minimum supported Rust version is 1.88, the one Iced 0.14 declares. The crate targets Iced 0.14.0 only.

Development and tests run on Linux with Wayland and X11. The crate is not built or run on Windows or macOS in this repository. Mobile and web targets are out of scope.

Iced 0.14 has no screen reader support, so the widgets have none.

## Acknowledgments

- Material Design 3 by Google, and the `material-web`, Material Components for Android and Jetpack Compose Material 3 sources, which are the reference for dimensions, colors and motion.
- Iced, the Rust GUI toolkit this library is built on.
- material-color-utils by Google. The `color` module is a Rust port of it. The port was checked value by value against the Java reference.
- Matugen, whose JSON output the scheme file format follows.
- Material Symbols by Google (Apache License 2.0) and Roboto (SIL Open Font License 1.1), which are bundled.

## Documentation and contributing

- API documentation: `cargo doc --open`.
- [docs/matugen.md](docs/matugen.md): the color scheme file format.
- [CONTRIBUTING.md](CONTRIBUTING.md): how to build, test and submit changes.

## License

GNU Lesser General Public License, version 3 only. The license texts are in `COPYING.LESSER` and `COPYING`. The bundled icons are under the Apache License 2.0 (`material-iced/icons/LICENSE`) and the fonts under the SIL Open Font License (`material-iced/fonts/OFL.txt`).
