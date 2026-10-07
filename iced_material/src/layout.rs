// SPDX-License-Identifier: LGPL-3.0-only

//! Window size classes, the layout grid and the canonical layouts.

use iced::widget::{Space, column, container, responsive, row};
use iced::{Length, Padding, Size};

use crate::Element;
use crate::theme::Theme;
use crate::widget::navigation::{Destination, navigation_bar, navigation_rail};
use crate::widget::navigation_drawer::{navigation_drawer, section};

/// Width class of a window. The breakpoints are those of `WindowSizeClass` in Jetpack Window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WidthClass {
    /// Less than 600 dp.
    Compact,
    /// 600 dp to 839 dp.
    Medium,
    /// 840 dp to 1199 dp.
    Expanded,
    /// 1200 dp to 1599 dp.
    Large,
    /// 1600 dp and more.
    ExtraLarge,
}

/// Height class of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeightClass {
    /// Less than 480 dp.
    Compact,
    /// 480 dp to 899 dp.
    Medium,
    /// 900 dp and more.
    Expanded,
}

impl WidthClass {
    /// The class of a window `width` dp wide.
    pub fn of(width: f32) -> WidthClass {
        match width {
            w if w >= 1600.0 => WidthClass::ExtraLarge,
            w if w >= 1200.0 => WidthClass::Large,
            w if w >= 840.0 => WidthClass::Expanded,
            w if w >= 600.0 => WidthClass::Medium,
            _ => WidthClass::Compact,
        }
    }

    /// The layout grid of this class.
    pub fn grid(self) -> Grid {
        Grid::of(self)
    }
}

impl HeightClass {
    /// The class of a window `height` dp high.
    pub fn of(height: f32) -> HeightClass {
        match height {
            h if h >= 900.0 => HeightClass::Expanded,
            h if h >= 480.0 => HeightClass::Medium,
            _ => HeightClass::Compact,
        }
    }
}

/// The width and height class of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowClass {
    /// Width class.
    pub width: WidthClass,
    /// Height class.
    pub height: HeightClass,
}

impl WindowClass {
    /// The classes of a window of `size`.
    pub fn of(size: Size) -> WindowClass {
        WindowClass {
            width: WidthClass::of(size.width),
            height: HeightClass::of(size.height),
        }
    }
}

/// Columns, margins and gutters of a window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    /// Number of columns.
    pub columns: usize,
    /// Space between the window edge and the first and last column.
    pub margin: f32,
    /// Space between columns.
    pub gutter: f32,
}

impl Grid {
    /// The grid of a width class.
    pub fn of(class: WidthClass) -> Grid {
        match class {
            WidthClass::Compact => Grid {
                columns: 4,
                margin: 16.0,
                gutter: 16.0,
            },
            WidthClass::Medium => Grid {
                columns: 8,
                margin: 24.0,
                gutter: 24.0,
            },
            _ => Grid {
                columns: 12,
                margin: 24.0,
                gutter: 24.0,
            },
        }
    }

    /// Width of one column in a window `total` dp wide.
    pub fn column_width(&self, total: f32) -> f32 {
        let columns = self.columns as f32;
        (total - self.margin * 2.0 - self.gutter * (columns - 1.0)) / columns
    }

    /// Width of `span` adjacent columns and the gutters between them.
    pub fn span(&self, total: f32, span: usize) -> f32 {
        let span = span.clamp(1, self.columns) as f32;
        self.column_width(total) * span + self.gutter * (span - 1.0)
    }
}

/// Builds a view from the class and size of the space it is given.
pub fn adaptive<'a, Message: 'a>(
    view: impl Fn(WindowClass, Size) -> Element<'a, Message> + 'a,
) -> Element<'a, Message> {
    responsive(move |size| view(WindowClass::of(size), size)).into()
}

/// Space around content so it follows the margins of the grid of `class`.
pub fn margins<'a, Message: 'a>(
    class: WidthClass,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content)
        .padding(Padding::from([0.0, class.grid().margin]))
        .into()
}

/// Width of the fixed pane of a two pane layout.
pub fn pane_width(class: WidthClass) -> f32 {
    if class >= WidthClass::ExtraLarge {
        412.0
    } else {
        360.0
    }
}

/// Space between two panes.
pub fn pane_spacing(class: WidthClass) -> f32 {
    if class >= WidthClass::Expanded {
        24.0
    } else {
        0.0
    }
}

fn two_panes<'a, Message: 'a>(
    class: WidthClass,
    two_on_medium: bool,
    fixed_first: bool,
    fixed: Element<'a, Message>,
    flexible: Option<Element<'a, Message>>,
    fixed_is_primary: bool,
) -> Element<'a, Message> {
    let both = class >= WidthClass::Expanded || (two_on_medium && class == WidthClass::Medium);
    match (both, flexible) {
        (true, Some(flexible)) => {
            let fixed = container(fixed).width(Length::Fixed(pane_width(class)));
            let flexible = container(flexible).width(Length::Fill);
            let spacing = pane_spacing(class).max(if two_on_medium { 24.0 } else { 0.0 });
            if fixed_first {
                row![fixed, Space::new().width(Length::Fixed(spacing)), flexible].into()
            } else {
                row![flexible, Space::new().width(Length::Fixed(spacing)), fixed].into()
            }
        }
        (false, Some(flexible)) => {
            if fixed_is_primary {
                fixed
            } else {
                flexible
            }
        }
        (_, None) => fixed,
    }
}

/// The list-detail layout.
///
/// Windows from the Expanded class show the list in a fixed pane next to the
/// detail. Narrower windows show one pane: the detail when there is one, else
/// the list. With `two_on_medium` a Medium window shows both panes too.
pub fn list_detail<'a, Message: 'a>(
    class: WidthClass,
    two_on_medium: bool,
    list: impl Into<Element<'a, Message>>,
    detail: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let list = list.into();
    match detail {
        Some(detail)
            if class >= WidthClass::Expanded || (two_on_medium && class == WidthClass::Medium) =>
        {
            two_panes(class, two_on_medium, true, list, Some(detail), true)
        }
        Some(detail) => detail,
        None => list,
    }
}

/// The supporting pane layout.
///
/// Windows from the Expanded class show the supporting pane in a fixed pane
/// after the main content. Narrower windows show the main content, or the
/// supporting content alone when `show_supporting` is true.
pub fn supporting_pane<'a, Message: 'a>(
    class: WidthClass,
    two_on_medium: bool,
    show_supporting: bool,
    main: impl Into<Element<'a, Message>>,
    supporting: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let (main, supporting) = (main.into(), supporting.into());
    let both = class >= WidthClass::Expanded || (two_on_medium && class == WidthClass::Medium);
    if both {
        two_panes(class, two_on_medium, false, supporting, Some(main), false)
    } else if show_supporting {
        supporting
    } else {
        main
    }
}

/// Number of columns of the feed layout in a window of `class`.
pub fn feed_columns(class: WidthClass) -> usize {
    match class {
        WidthClass::Compact => 1,
        WidthClass::Medium => 2,
        _ => 3,
    }
}

/// The feed layout: items in rows of equal columns, separated by the gutter of the grid.
pub fn feed<'a, Message: 'a>(
    class: WidthClass,
    items: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let columns = feed_columns(class);
    let gutter = class.grid().gutter;
    let mut rows = column![].spacing(gutter);
    let mut items = items.into_iter().peekable();
    while items.peek().is_some() {
        let mut line = row![].spacing(gutter);
        for _ in 0..columns {
            line = match items.next() {
                Some(item) => line.push(container(item).width(Length::FillPortion(1))),
                None => line.push(
                    Space::new()
                        .width(Length::FillPortion(1))
                        .height(Length::Shrink),
                ),
            };
        }
        rows = rows.push(line);
    }
    rows.into()
}

/// A page with navigation that follows the window class.
///
/// A Compact window gets a navigation bar below the content, a Medium window a
/// navigation rail beside it, and larger windows a standard navigation drawer.
pub fn scaffold<'a, Message: Clone + 'static>(
    theme: &Theme,
    class: WidthClass,
    destinations: Vec<Destination>,
    selected: usize,
    on_select: impl Fn(usize) -> Message + Clone + 'static,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let content = container(content).width(Length::Fill).height(Length::Fill);
    match class {
        WidthClass::Compact => column![
            content,
            navigation_bar(theme, destinations, selected, on_select)
        ]
        .into(),
        WidthClass::Medium => row![
            navigation_rail(theme, destinations, selected, on_select),
            content
        ]
        .into(),
        _ => {
            let items = destinations.iter().map(Destination::drawer_item).collect();
            row![
                navigation_drawer(vec![section(items)], selected, on_select).build(theme),
                content
            ]
            .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints_follow_window_size_class() {
        for (width, class) in [
            (0.0, WidthClass::Compact),
            (599.9, WidthClass::Compact),
            (600.0, WidthClass::Medium),
            (839.9, WidthClass::Medium),
            (840.0, WidthClass::Expanded),
            (1199.9, WidthClass::Expanded),
            (1200.0, WidthClass::Large),
            (1599.9, WidthClass::Large),
            (1600.0, WidthClass::ExtraLarge),
        ] {
            assert_eq!(WidthClass::of(width), class, "{width}");
        }
        assert_eq!(HeightClass::of(479.0), HeightClass::Compact);
        assert_eq!(HeightClass::of(480.0), HeightClass::Medium);
        assert_eq!(HeightClass::of(900.0), HeightClass::Expanded);
    }

    #[test]
    fn grid_columns_and_spans_add_up() {
        let grid = WidthClass::Compact.grid();
        assert_eq!(grid.columns, 4);
        let total = 360.0;
        let column = grid.column_width(total);
        assert!((column * 4.0 + grid.gutter * 3.0 + grid.margin * 2.0 - total).abs() < 1e-3);
        assert!((grid.span(total, 4) - (total - 32.0)).abs() < 1e-3);
        assert!((grid.span(total, 1) - column).abs() < 1e-3);
        assert_eq!(WidthClass::Medium.grid().columns, 8);
        assert_eq!(WidthClass::Expanded.grid().columns, 12);
    }

    #[test]
    fn pane_sizes_follow_the_adaptive_directive() {
        assert_eq!(pane_width(WidthClass::Expanded), 360.0);
        assert_eq!(pane_width(WidthClass::Large), 360.0);
        assert_eq!(pane_width(WidthClass::ExtraLarge), 412.0);
        assert_eq!(pane_spacing(WidthClass::Medium), 0.0);
        assert_eq!(pane_spacing(WidthClass::Expanded), 24.0);
        assert_eq!(feed_columns(WidthClass::Compact), 1);
        assert_eq!(feed_columns(WidthClass::Medium), 2);
        assert_eq!(feed_columns(WidthClass::ExtraLarge), 3);
    }
}
