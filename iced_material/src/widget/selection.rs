// SPDX-License-Identifier: LGPL-3.0-only

use iced::advanced::Shell;
use iced::border::Radius;
use iced::mouse::Cursor;
use iced::time::Instant;
use iced::{Color, Event, Point, Rectangle, Renderer, Size, window};

use crate::draw::surface;
use crate::interaction::{Activation, Tokens};
use crate::motion::{Transition, Tween};
use crate::theme::Theme;
use crate::widget::pressable;

pub(crate) struct State {
    pub press: pressable::State,
    pub selection: Tween,
    selected: bool,
}

impl State {
    pub fn new(selected: bool) -> State {
        State {
            press: pressable::State::default(),
            selection: Tween::new(if selected { 1.0 } else { 0.0 }),
            selected,
        }
    }

    pub fn now(&self) -> Option<Instant> {
        self.press.now()
    }

    pub fn progress(&self) -> f32 {
        self.selection.value(self.now())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update<Message>(
        &mut self,
        event: &Event,
        selected: bool,
        transition: Transition,
        hit: Rectangle,
        cursor: Cursor,
        enabled: bool,
        tokens: &Tokens,
        shell: &mut Shell<'_, Message>,
    ) -> Option<Activation> {
        if selected != self.selected {
            self.selected = selected;
            self.selection
                .go(if selected { 1.0 } else { 0.0 }, transition, self.now());
            shell.request_redraw();
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && self.selection.tick(*now)
        {
            shell.request_redraw();
        }
        self.press
            .update(event, hit, cursor, enabled, false, tokens, shell)
    }
}

pub(crate) fn centered(bounds: Rectangle, size: Size) -> Rectangle {
    Rectangle::new(
        Point::new(
            bounds.center_x() - size.width / 2.0,
            bounds.center_y() - size.height / 2.0,
        ),
        size,
    )
}

pub(crate) fn draw_state_layer(
    renderer: &mut Renderer,
    theme: &Theme,
    state: &State,
    layer: Rectangle,
    hover: Color,
    pressed: Color,
) {
    let radius = Radius::from(layer.width / 2.0);
    draw_state_layer_rect(renderer, theme, state, layer, radius, hover, pressed);
}

pub(crate) fn draw_state_layer_rect(
    renderer: &mut Renderer,
    theme: &Theme,
    state: &State,
    layer: Rectangle,
    radius: Radius,
    hover: Color,
    pressed: Color,
) {
    let now = state.now();
    surface::state_layer(
        renderer,
        layer,
        radius,
        hover,
        state.press.interaction.hover.value(now),
    );
    state
        .press
        .interaction
        .ripple
        .draw(renderer, layer, radius, pressed, theme, now);
}
