// SPDX-License-Identifier: LGPL-3.0-only

use std::any::Any;

use iced::Rectangle;
use iced::advanced::widget::operation::Focusable;
use iced::advanced::widget::{Id, Operation};

pub(crate) struct Probe(pub bool);

impl Operation for Probe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.0 |= state.is_focused();
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, _state: &mut dyn Any) {}
}
