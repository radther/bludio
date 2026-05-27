//! Action button — a small pill-shaped button for device rows.
//!
//! Used by Bluetooth and Audio device rows for actions like Connect,
//! Disconnect, Mute, Default, etc.

use crate::ui::StyledExt;
use gpui::{ClickEvent, CursorStyle, Div, Hsla, SharedString, Stateful, div, prelude::*};

use super::super::theme::TextStyleSet;

/// Build a small action button with the given colors and click handler.
///
/// Text color should be [`ThemeColors::text_colored_button`] for buttons
/// with accent/danger backgrounds, or [`ThemeColors::text`] for buttons
/// on `element_background`.
pub fn action_btn(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    bg: Hsla,
    hover_bg: Hsla,
    text_color: Hsla,
    on_click: impl Fn() + 'static,
    text_styles: &TextStyleSet,
) -> Stateful<Div> {
    let id: SharedString = id.into();
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded_sm()
        .styled(text_styles.caption)
        .bg(bg)
        .text_color(text_color)
        .cursor(CursorStyle::PointingHand)
        .hover(move |el| el.bg(hover_bg))
        .child(label.into())
        .on_click(move |_: &ClickEvent, _window, _cx| {
            on_click();
        })
}
