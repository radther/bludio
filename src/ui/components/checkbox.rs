//! Custom checkbox — a square on/off control with a check icon.
//!
//! A rounded square box that shows a check icon when checked.
//! When checked, the box fills with the audio accent color.
//! When unchecked, the box is empty with a border.

use gpui::{CursorStyle, Hsla, Stateful, div, prelude::*, px};

use crate::ui::theme::ThemeColors;

// ── Checkbox builder ───────────────────────────────────────────────────────

/// Render a square checkbox with a check icon.
///
/// When checked, the box fills with the accent color and shows the check icon.
/// When unchecked, the box is empty with a subtle border.
///
/// # Example
/// ```ignore
/// checkbox("my-checkbox", true, &colors)
///     .on_click(|_, _, _| {})
/// ```
pub fn checkbox(
    id: impl Into<gpui::ElementId>,
    is_checked: bool,
    colors: &ThemeColors,
) -> Stateful<gpui::Div> {
    let box_bg = if is_checked {
        colors.audio_accent
    } else {
        colors.element_background
    };
    let box_border = if is_checked {
        colors.audio_accent
    } else {
        colors.border
    };
    let icon_color = colors.text_colored_button;

    div()
        .id(id.into())
        .cursor(CursorStyle::PointingHand)
        .child(
            div()
                .w(px(20.0))
                .h(px(20.0))
                .rounded_sm()
                .bg(box_bg)
                .border_1()
                .border_color(box_border)
                .flex()
                .items_center()
                .justify_center()
                .when(is_checked, |checked| {
                    checked.child(
                        crate::ui::common::icons::check()
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(icon_color),
                    )
                }),
        )
}

/// Render a checkbox using a custom color instead of the default audio accent.
#[allow(dead_code)]
pub fn checkbox_with_color(
    id: impl Into<gpui::ElementId>,
    is_checked: bool,
    active_color: Hsla,
    colors: &ThemeColors,
) -> Stateful<gpui::Div> {
    let box_bg = if is_checked {
        active_color
    } else {
        colors.element_background
    };
    let box_border = if is_checked {
        active_color
    } else {
        colors.border
    };
    let icon_color = colors.text_colored_button;

    div()
        .id(id.into())
        .cursor(CursorStyle::PointingHand)
        .child(
            div()
                .w(px(20.0))
                .h(px(20.0))
                .rounded_sm()
                .bg(box_bg)
                .border_1()
                .border_color(box_border)
                .flex()
                .items_center()
                .justify_center()
                .when(is_checked, |checked| {
                    checked.child(
                        crate::ui::common::icons::check()
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(icon_color),
                    )
                }),
        )
}
