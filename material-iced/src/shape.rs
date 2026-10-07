// SPDX-License-Identifier: LGPL-3.0-only

//! Material shape scale.

use iced::Size;
use iced::border::Radius;

/// Size of one corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    /// Radius in logical pixels.
    Fixed(f32),
    /// Half of the shorter side of the shape.
    Full,
}

impl Corner {
    fn resolve(self, size: Size) -> f32 {
        match self {
            Corner::Fixed(radius) => radius,
            Corner::Full => size.width.min(size.height) / 2.0,
        }
    }
}

/// Corner sizes of a shape in the order top left, top right, bottom right, bottom left.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    pub top_left: Corner,
    pub top_right: Corner,
    pub bottom_right: Corner,
    pub bottom_left: Corner,
}

impl Shape {
    /// A shape with the same size on every corner.
    pub const fn all(corner: Corner) -> Shape {
        Shape {
            top_left: corner,
            top_right: corner,
            bottom_right: corner,
            bottom_left: corner,
        }
    }

    /// A shape with only the top corners rounded.
    pub const fn top(corner: Corner) -> Shape {
        Shape {
            top_left: corner,
            top_right: corner,
            bottom_right: Corner::Fixed(0.0),
            bottom_left: Corner::Fixed(0.0),
        }
    }

    /// A shape with only the start corners rounded, for left to right layouts.
    pub const fn start(corner: Corner) -> Shape {
        Shape {
            top_left: corner,
            top_right: Corner::Fixed(0.0),
            bottom_right: Corner::Fixed(0.0),
            bottom_left: corner,
        }
    }

    /// A shape with only the end corners rounded, for left to right layouts.
    pub const fn end(corner: Corner) -> Shape {
        Shape {
            top_left: Corner::Fixed(0.0),
            top_right: corner,
            bottom_right: corner,
            bottom_left: Corner::Fixed(0.0),
        }
    }

    /// Resolves the corner radii for a shape of the given size.
    pub fn radius(&self, size: Size) -> Radius {
        Radius {
            top_left: self.top_left.resolve(size),
            top_right: self.top_right.resolve(size),
            bottom_right: self.bottom_right.resolve(size),
            bottom_left: self.bottom_left.resolve(size),
        }
    }

    /// Interpolates the resolved radii of two shapes.
    pub fn lerp(&self, other: &Shape, amount: f32, size: Size) -> Radius {
        let a = self.radius(size);
        let b = other.radius(size);
        let mix = |x: f32, y: f32| x + (y - x) * amount;
        Radius {
            top_left: mix(a.top_left, b.top_left),
            top_right: mix(a.top_right, b.top_right),
            bottom_right: mix(a.bottom_right, b.bottom_right),
            bottom_left: mix(a.bottom_left, b.bottom_left),
        }
    }

    /// Grows every corner by `offset`, used for outlines drawn outside a shape.
    pub fn outset(&self, offset: f32, size: Size) -> Radius {
        let r = self.radius(size);
        Radius {
            top_left: r.top_left + offset,
            top_right: r.top_right + offset,
            bottom_right: r.bottom_right + offset,
            bottom_left: r.bottom_left + offset,
        }
    }
}

/// The Material corner shape tokens.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeScale {
    pub none: Shape,
    pub extra_small: Shape,
    pub extra_small_top: Shape,
    pub small: Shape,
    pub medium: Shape,
    pub large: Shape,
    pub large_top: Shape,
    pub large_start: Shape,
    pub large_end: Shape,
    pub extra_large: Shape,
    pub extra_large_top: Shape,
    pub full: Shape,
}

impl Default for ShapeScale {
    fn default() -> Self {
        ShapeScale {
            none: Shape::all(Corner::Fixed(0.0)),
            extra_small: Shape::all(Corner::Fixed(4.0)),
            extra_small_top: Shape::top(Corner::Fixed(4.0)),
            small: Shape::all(Corner::Fixed(8.0)),
            medium: Shape::all(Corner::Fixed(12.0)),
            large: Shape::all(Corner::Fixed(16.0)),
            large_top: Shape::top(Corner::Fixed(16.0)),
            large_start: Shape::start(Corner::Fixed(16.0)),
            large_end: Shape::end(Corner::Fixed(16.0)),
            extra_large: Shape::all(Corner::Fixed(28.0)),
            extra_large_top: Shape::top(Corner::Fixed(28.0)),
            full: Shape::all(Corner::Full),
        }
    }
}
