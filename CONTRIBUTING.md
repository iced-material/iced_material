# Contributing

## Requirements

- Rust 1.88 or newer. The code uses edition 2024.
- On Linux, a Wayland or X11 session to run the gallery. The tests do not need a window.
- To regenerate the color reference vectors only: a JDK with `javac` and a checkout of material-color-utils.

## Build and run

```
cargo build
cargo run -p gallery
cargo run -p material-iced --example minimal --features roboto
```

The gallery has one page per group of components. It is the quickest way to look at a change.

## Test

```
cargo test
```

The tests render widgets without a window on `tiny-skia`, send events to them and read pixels back. A test that needs a new kind of input or a measurement belongs in `material-iced/tests/common/mod.rs`, next to the helpers the other tests use.

- `ICED_MATERIAL_BACKEND=wgpu cargo test` runs the same tests on `wgpu`. `tests/renderers.rs` compares both renderers on one scene.
- `cargo test -p gallery` opens every page of the gallery, drives it with random input and opens every overlay. These tests take a few minutes.
- `cargo test -p gallery dump_screenshots -- --ignored` writes PNG files of several pages to `/tmp/shots`. Set `SHOTS` to use another directory.
- `cargo +1.88.0 check --workspace --all-targets` checks the minimum supported Rust version.

The color code is compared against vectors generated from the Java reference. To regenerate them:

```
tools/vectors/generate.sh <path to material-color-utils> material-iced/tests/data
```

The Material Symbols in `material-iced/icons` are fetched with `tools/icons/fetch.sh <directory> <symbol name>...`.

## Format and lint

```
cargo fmt
cargo clippy --all-targets -- -D warnings
```

Both must pass without changes or warnings.

## Changing or adding a widget

- Read dimensions, colors and motion from the theme, not from constants in the widget. Metrics of a widget are a struct in the widget module and a field of `theme::Components`.
- Offer a `Catalog` trait and a `Style` struct, as the existing widgets do, so that applications can restyle the widget.
- Make every interactive widget usable with the keyboard and show a focus indicator.
- Test geometry, the colors of each state, keyboard use and the messages the widget publishes.
- Every Rust source file starts with the line `// SPDX-License-Identifier: LGPL-3.0-only`.
- Public items have documentation. Widgets that are only used inside the crate stay private.

## Submitting changes

Keep a change to one subject, so that it can be reviewed and reverted on its own. In the description say what changed and how you checked it. Include the output of `cargo test` if a test was added or changed.

## License

By submitting a change you agree that it is licensed under the GNU Lesser General Public License, version 3 only, like the rest of the project.
